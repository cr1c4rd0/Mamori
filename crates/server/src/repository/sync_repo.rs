//! Persistencia de la sincronización en PostgreSQL.
//!
//! Especificación: `specs/esquemas_bd/servidor_postgres.md`.

use crate::error::ApiError;
use chrono::{DateTime, Utc};
use mamori_shared::sync::{
    ChangeOp, ChangeRecord, Collection, DataSnapshot, PackageRecord, ParkingSpotRecord, PullRequest,
    PullResponse, PushRequest, PushResponse, ResidentRecord, SequencedChange, SyncRecord, TowerRecord,
    UnitRecord, VisitRecord,
};
use sqlx::{Connection, PgConnection, PgPool, Row};
use uuid::Uuid;

/// Llave del bloqueo consultivo que serializa los Push (RN-SRV-01).
const PUSH_LOCK_KEY: i64 = 0x4D414D4F5249; // "MAMORI"
/// Máximo de cambios devueltos por un Pull (RN-SRV-03).
pub const PULL_LIMIT: i64 = 5_000;

const SEED_JSON: &str = include_str!("../../../../specs/mocks_dummy_data/seed_estado_local.json");

fn enum_text<T: serde::Serialize>(v: &T) -> Result<String, ApiError> {
    match serde_json::to_value(v)? {
        serde_json::Value::String(s) => Ok(s),
        other => Ok(other.to_string()),
    }
}

fn text_enum<T: serde::de::DeserializeOwned>(s: String) -> Result<T, ApiError> {
    Ok(serde_json::from_value(serde_json::Value::String(s))?)
}

/// Registra los cambios de una terminal (idempotente por `change_id`) y los materializa.
pub async fn push(pool: &PgPool, req: &PushRequest) -> Result<PushResponse, ApiError> {
    if let Some(bad) = req.changes.iter().find(|c| c.origin_node != req.node_id) {
        return Err(ApiError::BadRequest(format!(
            "el cambio {} pertenece al nodo {}, no a {}",
            bad.change_id, bad.origin_node, req.node_id
        )));
    }

    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)").bind(PUSH_LOCK_KEY).execute(&mut *tx).await?;
    sqlx::query(
        "INSERT INTO sync_nodes (node_id) VALUES ($1)
         ON CONFLICT (node_id) DO UPDATE SET last_seen_at = now()",
    )
    .bind(req.node_id)
    .execute(&mut *tx)
    .await?;

    let mut inserted = 0usize;
    for change in &req.changes {
        let seq: Option<i64> = sqlx::query_scalar(
            "INSERT INTO change_log (change_id, origin_node, collection, record_id, op, payload, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (change_id) DO NOTHING
             RETURNING seq",
        )
        .bind(change.change_id)
        .bind(change.origin_node)
        .bind(enum_text(&change.collection)?)
        .bind(&change.record_id)
        .bind(enum_text(&change.op)?)
        .bind(&change.payload)
        .bind(change.created_at)
        .fetch_optional(&mut *tx)
        .await?;

        if seq.is_some() {
            inserted += 1;
            materialize_or_note(&mut tx, change).await?;
        }
    }

    let server_seq: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(seq), 0) FROM change_log")
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;

    tracing::info!(
        "Push de {}: {} cambios ({} nuevos), seq servidor = {}",
        req.node_id,
        req.changes.len(),
        inserted,
        server_seq
    );
    Ok(PushResponse {
        accepted: req.changes.iter().map(|c| c.change_id).collect(),
        server_seq: server_seq as u64,
    })
}

/// Devuelve los cambios de otros nodos desde el cursor (RN-SYNC-06, RN-SRV-02/03).
pub async fn pull(pool: &PgPool, req: &PullRequest) -> Result<PullResponse, ApiError> {
    let rows = sqlx::query(
        "SELECT seq, change_id, origin_node, collection, record_id, op, payload, created_at
         FROM change_log WHERE seq > $1 ORDER BY seq LIMIT $2",
    )
    .bind(req.since_seq as i64)
    .bind(PULL_LIMIT)
    .fetch_all(pool)
    .await?;

    let mut cursor = req.since_seq;
    let mut changes = Vec::new();
    for row in rows {
        let seq: i64 = row.try_get("seq")?;
        cursor = seq as u64;
        let origin_node: Uuid = row.try_get("origin_node")?;
        if origin_node == req.node_id {
            continue;
        }
        changes.push(SequencedChange {
            seq: seq as u64,
            change: ChangeRecord {
                change_id: row.try_get("change_id")?,
                origin_node,
                collection: text_enum(row.try_get("collection")?)?,
                record_id: row.try_get("record_id")?,
                op: text_enum(row.try_get("op")?)?,
                payload: row.try_get("payload")?,
                created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
            },
        });
    }

    sqlx::query(
        "INSERT INTO sync_nodes (node_id) VALUES ($1)
         ON CONFLICT (node_id) DO UPDATE SET last_seen_at = now()",
    )
    .bind(req.node_id)
    .execute(pool)
    .await?;

    Ok(PullResponse { changes, server_seq: cursor })
}

/// Materializa en un SAVEPOINT; si falla, el cambio queda en el log con el motivo (RN-SRV-N01).
async fn materialize_or_note(conn: &mut PgConnection, change: &ChangeRecord) -> Result<(), ApiError> {
    let mut sp = conn.begin().await?;
    match materialize(&mut sp, change).await {
        Ok(()) => sp.commit().await?,
        Err(e) => {
            sp.rollback().await?;
            tracing::warn!("Cambio {} guardado sin materializar: {}", change.change_id, e);
            sqlx::query("UPDATE change_log SET apply_error = $1 WHERE change_id = $2")
                .bind(e.to_string())
                .bind(change.change_id)
                .execute(&mut *conn)
                .await?;
        }
    }
    Ok(())
}

fn payload<T: SyncRecord>(change: &ChangeRecord) -> Result<T, ApiError> {
    let value = change
        .payload
        .clone()
        .ok_or_else(|| ApiError::BadRequest(format!("upsert sin payload: {}", change.change_id)))?;
    Ok(serde_json::from_value(value)?)
}

async fn materialize(conn: &mut PgConnection, change: &ChangeRecord) -> Result<(), ApiError> {
    let by = change.origin_node;
    let at = change.created_at;

    if change.op == ChangeOp::Delete {
        let sql = match change.collection {
            Collection::Towers => "DELETE FROM towers WHERE id = $1",
            Collection::Units => "DELETE FROM units WHERE id = $1",
            Collection::ParkingSpots => "DELETE FROM parking_spots WHERE code = $1",
            Collection::Residents => "DELETE FROM residents WHERE id = $1",
            Collection::Visits => "DELETE FROM visits WHERE id = $1",
            Collection::Packages => "DELETE FROM packages WHERE id = $1",
        };
        sqlx::query(sql).bind(&change.record_id).execute(&mut *conn).await?;
        return Ok(());
    }

    match change.collection {
        Collection::Towers => upsert_tower(conn, &payload(change)?, at, Some(by)).await,
        Collection::Units => upsert_unit(conn, &payload(change)?, at, Some(by)).await,
        Collection::ParkingSpots => upsert_parking(conn, &payload(change)?, at, Some(by)).await,
        Collection::Residents => upsert_resident(conn, &payload(change)?, at, Some(by)).await,
        Collection::Visits => upsert_visit(conn, &payload(change)?, at, Some(by)).await,
        Collection::Packages => upsert_package(conn, &payload(change)?, at, Some(by)).await,
    }
}

async fn upsert_tower(c: &mut PgConnection, t: &TowerRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO towers (id, name, is_active, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (id) DO UPDATE SET name=$2, is_active=$3, updated_at=$4, updated_by=$5",
    )
    .bind(&t.id).bind(&t.name).bind(t.is_active).bind(at).bind(by)
    .execute(c)
    .await?;
    Ok(())
}

async fn upsert_unit(c: &mut PgConnection, u: &UnitRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO units (id, tower, unit_number, unit_type, intercom, mobile, landline, is_active, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
         ON CONFLICT (id) DO UPDATE SET tower=$2, unit_number=$3, unit_type=$4, intercom=$5,
             mobile=$6, landline=$7, is_active=$8, updated_at=$9, updated_by=$10",
    )
    .bind(&u.id).bind(&u.tower).bind(&u.unit_number).bind(&u.unit_type)
    .bind(&u.intercom).bind(&u.mobile).bind(&u.landline).bind(u.is_active).bind(at).bind(by)
    .execute(c)
    .await?;
    Ok(())
}

async fn upsert_parking(c: &mut PgConnection, p: &ParkingSpotRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO parking_spots (code, spot_type, assigned_unit_id, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (code) DO UPDATE SET spot_type=$2, assigned_unit_id=$3, updated_at=$4, updated_by=$5",
    )
    .bind(&p.code).bind(&p.spot_type).bind(&p.assigned_unit_id).bind(at).bind(by)
    .execute(c)
    .await?;
    Ok(())
}

async fn upsert_resident(c: &mut PgConnection, r: &ResidentRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO residents (id, full_name, document, role, mobile, email, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         ON CONFLICT (id) DO UPDATE SET full_name=$2, document=$3, role=$4, mobile=$5, email=$6,
             updated_at=$7, updated_by=$8",
    )
    .bind(&r.id).bind(&r.full_name).bind(&r.document).bind(&r.role)
    .bind(&r.mobile).bind(&r.email).bind(at).bind(by)
    .execute(&mut *c)
    .await?;
    sqlx::query("DELETE FROM resident_units WHERE resident_id = $1").bind(&r.id).execute(&mut *c).await?;
    for label in &r.units {
        sqlx::query("INSERT INTO resident_units (resident_id, unit_label) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&r.id)
            .bind(label)
            .execute(&mut *c)
            .await?;
    }
    Ok(())
}

async fn upsert_visit(c: &mut PgConnection, v: &VisitRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO visits (id, visitor_name, document, destination, access_type, entry_at, exit_at, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         ON CONFLICT (id) DO UPDATE SET visitor_name=$2, document=$3, destination=$4, access_type=$5,
             entry_at=$6, exit_at=$7, updated_at=$8, updated_by=$9",
    )
    .bind(&v.id).bind(&v.visitor_name).bind(&v.document).bind(&v.destination)
    .bind(&v.access_type).bind(v.entry_at).bind(v.exit_at).bind(at).bind(by)
    .execute(c)
    .await?;
    Ok(())
}

async fn upsert_package(c: &mut PgConnection, p: &PackageRecord, at: DateTime<Utc>, by: Option<Uuid>) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO packages (id, destination, recipient, carrier, description, received_at, delivered_at, updated_at, updated_by)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         ON CONFLICT (id) DO UPDATE SET destination=$2, recipient=$3, carrier=$4, description=$5,
             received_at=$6, delivered_at=$7, updated_at=$8, updated_by=$9",
    )
    .bind(&p.id).bind(&p.destination).bind(&p.recipient).bind(&p.carrier)
    .bind(&p.description).bind(p.received_at).bind(p.delivered_at).bind(at).bind(by)
    .execute(c)
    .await?;
    Ok(())
}

/// Garantiza las torres iniciales (mismos ids que en las terminales). No pisa torres
/// existentes, así que respeta renombres e inactivaciones ya sincronizados.
pub async fn ensure_seed_towers(pool: &PgPool) -> Result<u64, ApiError> {
    let seed: DataSnapshot = serde_json::from_str(SEED_JSON)?;
    let mut inserted = 0;
    for t in &seed.towers {
        inserted += sqlx::query("INSERT INTO towers (id, name, is_active) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
            .bind(&t.id)
            .bind(&t.name)
            .bind(t.is_active)
            .execute(pool)
            .await?
            .rows_affected();
    }
    Ok(inserted)
}

/// Carga los datos iniciales si la base está vacía (RN-SRV-04).
pub async fn seed_if_empty(pool: &PgPool) -> Result<bool, ApiError> {
    let units: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM units").fetch_one(pool).await?;
    let log: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM change_log").fetch_one(pool).await?;
    if units > 0 || log > 0 {
        return Ok(false);
    }
    let seed: DataSnapshot = serde_json::from_str(SEED_JSON)?;
    let at = Utc::now();
    let mut tx = pool.begin().await?;
    for u in &seed.units {
        upsert_unit(&mut tx, u, at, None).await?;
    }
    for p in &seed.parking_spots {
        upsert_parking(&mut tx, p, at, None).await?;
    }
    for r in &seed.residents {
        upsert_resident(&mut tx, r, at, None).await?;
    }
    for v in &seed.visits {
        upsert_visit(&mut tx, v, at, None).await?;
    }
    for p in &seed.packages {
        upsert_package(&mut tx, p, at, None).await?;
    }
    tx.commit().await?;
    Ok(true)
}
