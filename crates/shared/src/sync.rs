//! Contrato de sincronización Local-First (D-22 a D-25).
//!
//! Especificación: `specs/contratos_api/protocolo_sincronizacion.md` y
//! `specs/esquemas_bd/almacenamiento_local_poc.md`. Lo usan la terminal
//! (`client-desktop`), el servidor simulado y, más adelante, `crates/server`.

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// 1. REGISTROS PERSISTIDOS (estado completo de cada entidad en la terminal)
// ============================================================================

/// Inmueble tal como lo opera la terminal de portería.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitRecord {
    pub id: String,
    pub tower: String,
    pub unit_number: String,
    pub unit_type: String,
    pub intercom: Option<String>,
    pub mobile: Option<String>,
    pub landline: Option<String>,
    /// Códigos de parqueaderos privados asignados (ej. `["P-101"]`).
    pub parking: Vec<String>,
}

/// Celda de parqueadero de la copropiedad. La clave es `code`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParkingSpotRecord {
    pub code: String,
    /// `Carro` o `Moto`.
    pub spot_type: String,
    /// `None` = disponible.
    pub assigned_unit_id: Option<String>,
}

impl ParkingSpotRecord {
    /// Texto mostrado en los selectores: `P-104 (Carro)`.
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.code, self.spot_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResidentRecord {
    pub id: String,
    pub full_name: String,
    pub document: Option<String>,
    /// Etiquetas de inmuebles asociados (ej. `Torre 1 - Apto 102`).
    pub units: Vec<String>,
    pub role: String,
    pub mobile: Option<String>,
    pub email: Option<String>,
}

/// Visita registrada en portería. Activa mientras `exit_at` sea `None` (RN-SYNC-N06).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisitRecord {
    pub id: String,
    pub visitor_name: String,
    pub document: String,
    pub destination: String,
    pub access_type: String,
    pub entry_at: DateTime<Utc>,
    pub exit_at: Option<DateTime<Utc>>,
}

/// Paquete recibido en portería. Pendiente mientras `delivered_at` sea `None`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageRecord {
    pub id: String,
    pub destination: String,
    pub recipient: String,
    pub carrier: String,
    pub description: String,
    pub received_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

/// Registro que puede sincronizarse: sabe a qué colección pertenece y cuál es su clave.
pub trait SyncRecord: Serialize + DeserializeOwned + Clone {
    const COLLECTION: Collection;
    fn record_id(&self) -> &str;
}

macro_rules! impl_sync_record {
    ($ty:ty, $col:expr, $key:ident) => {
        impl SyncRecord for $ty {
            const COLLECTION: Collection = $col;
            fn record_id(&self) -> &str {
                &self.$key
            }
        }
    };
}

impl_sync_record!(UnitRecord, Collection::Units, id);
impl_sync_record!(ParkingSpotRecord, Collection::ParkingSpots, code);
impl_sync_record!(ResidentRecord, Collection::Residents, id);
impl_sync_record!(VisitRecord, Collection::Visits, id);
impl_sync_record!(PackageRecord, Collection::Packages, id);

/// Estado completo de un nodo (terminal o servidor central).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DataSnapshot {
    pub units: Vec<UnitRecord>,
    pub parking_spots: Vec<ParkingSpotRecord>,
    pub residents: Vec<ResidentRecord>,
    pub visits: Vec<VisitRecord>,
    pub packages: Vec<PackageRecord>,
}

fn apply_to<T: SyncRecord>(
    list: &mut Vec<T>,
    change: &ChangeRecord,
) -> Result<(), serde_json::Error> {
    match change.op {
        ChangeOp::Delete => list.retain(|r| r.record_id() != change.record_id),
        ChangeOp::Upsert => {
            let payload = change.payload.clone().unwrap_or(serde_json::Value::Null);
            let record: T = serde_json::from_value(payload)?;
            match list.iter_mut().find(|r| r.record_id() == change.record_id) {
                Some(existing) => *existing = record,
                None => list.push(record),
            }
        }
    }
    Ok(())
}

impl DataSnapshot {
    /// Aplica un cambio (upsert del registro completo o borrado) al estado.
    pub fn apply(&mut self, change: &ChangeRecord) -> Result<(), serde_json::Error> {
        match change.collection {
            Collection::Units => apply_to(&mut self.units, change),
            Collection::ParkingSpots => apply_to(&mut self.parking_spots, change),
            Collection::Residents => apply_to(&mut self.residents, change),
            Collection::Visits => apply_to(&mut self.visits, change),
            Collection::Packages => apply_to(&mut self.packages, change),
        }
    }
}

// ============================================================================
// 2. CAMBIOS Y PROTOCOLO PUSH / PULL
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Collection {
    Units,
    ParkingSpots,
    Residents,
    Visits,
    Packages,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    Upsert,
    Delete,
}

/// Un cambio de un registro completo (RN-SYNC-03).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeRecord {
    /// Único por cambio; permite que el servidor sea idempotente (RN-SYNC-05).
    pub change_id: Uuid,
    pub origin_node: Uuid,
    pub collection: Collection,
    pub record_id: String,
    pub op: ChangeOp,
    /// Registro completo serializado; `None` si `op = Delete`.
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl ChangeRecord {
    /// Crea un cambio `Upsert` con el estado completo del registro.
    pub fn upsert<T: SyncRecord>(origin_node: Uuid, record: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            change_id: Uuid::new_v4(),
            origin_node,
            collection: T::COLLECTION,
            record_id: record.record_id().to_string(),
            op: ChangeOp::Upsert,
            payload: Some(serde_json::to_value(record)?),
            created_at: Utc::now(),
        })
    }

    /// Clave `(colección, id)` usada para compactar la cola y detectar conflictos.
    pub fn key(&self) -> (Collection, &str) {
        (self.collection, self.record_id.as_str())
    }
}

/// Cambio con el número de secuencia asignado por el servidor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SequencedChange {
    pub seq: u64,
    pub change: ChangeRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub node_id: Uuid,
    pub changes: Vec<ChangeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub accepted: Vec<Uuid>,
    pub server_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub node_id: Uuid,
    pub since_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub changes: Vec<SequencedChange>,
    pub server_seq: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resident(id: &str, name: &str) -> ResidentRecord {
        ResidentRecord {
            id: id.into(),
            full_name: name.into(),
            document: None,
            units: vec!["Torre 1 - Apto 101".into()],
            role: "Propietario".into(),
            mobile: None,
            email: None,
        }
    }

    #[test]
    fn upsert_inserta_y_luego_reemplaza() {
        let node = Uuid::new_v4();
        let mut snap = DataSnapshot::default();
        snap.apply(&ChangeRecord::upsert(node, &resident("r1", "Ana")).unwrap()).unwrap();
        snap.apply(&ChangeRecord::upsert(node, &resident("r1", "Ana María")).unwrap()).unwrap();
        assert_eq!(snap.residents.len(), 1);
        assert_eq!(snap.residents[0].full_name, "Ana María");
    }

    #[test]
    fn delete_retira_el_registro() {
        let node = Uuid::new_v4();
        let mut snap = DataSnapshot::default();
        snap.apply(&ChangeRecord::upsert(node, &resident("r1", "Ana")).unwrap()).unwrap();
        let mut del = ChangeRecord::upsert(node, &resident("r1", "Ana")).unwrap();
        del.op = ChangeOp::Delete;
        del.payload = None;
        snap.apply(&del).unwrap();
        assert!(snap.residents.is_empty());
    }

    #[test]
    fn parqueadero_usa_codigo_como_clave() {
        let spot = ParkingSpotRecord { code: "P-104".into(), spot_type: "Carro".into(), assigned_unit_id: None };
        let change = ChangeRecord::upsert(Uuid::new_v4(), &spot).unwrap();
        assert_eq!(change.key(), (Collection::ParkingSpots, "P-104"));
        assert_eq!(spot.display_name(), "P-104 (Carro)");
    }
}
