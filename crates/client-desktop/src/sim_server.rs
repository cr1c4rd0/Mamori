//! Servidor central simulado como nodo de un sistema distribuido (D-25).
//!
//! Contrato: `specs/contratos_api/protocolo_sincronizacion.md`. Varias terminales
//! (procesos) comparten la misma carpeta; un bloqueo de archivo del sistema
//! operativo serializa los accesos. Se reemplazará por endpoints Axum reales en
//! `crates/server` con los mismos DTOs de `mamori_shared::sync`.

use crate::store::{read_json, write_json_atomic};
pub use crate::transport::ServerError;
use mamori_shared::sync::{
    DataSnapshot, PullRequest, PullResponse, PushRequest, PushResponse, SequencedChange,
};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

const LOG_FILE: &str = "log_cambios.jsonl";
const CENTRAL_FILE: &str = "estado_central.json";
const LOCK_FILE: &str = ".lock";
/// Si este archivo existe en la carpeta del servidor, se simula caída de conexión.
pub const OFFLINE_FLAG_FILE: &str = "SIN_CONEXION";

fn storage<E: std::fmt::Display>(e: E) -> ServerError {
    ServerError::Failed(e.to_string())
}

#[derive(Clone)]
pub struct SimServer {
    dir: PathBuf,
    latency: Duration,
}

impl SimServer {
    pub fn new(dir: impl Into<PathBuf>, latency: Duration) -> Self {
        Self { dir: dir.into(), latency }
    }

    fn check_reachable(&self) -> Result<(), ServerError> {
        let env_offline = std::env::var("MAMORI_SIM_OFFLINE").is_ok_and(|v| v == "1");
        if env_offline || self.dir.join(OFFLINE_FLAG_FILE).exists() {
            return Err(ServerError::Unreachable);
        }
        Ok(())
    }

    pub async fn push(&self, req: PushRequest) -> Result<PushResponse, ServerError> {
        tokio::time::sleep(self.latency).await;
        self.check_reachable()?;
        let dir = self.dir.clone();
        tokio::task::spawn_blocking(move || push_blocking(&dir, req))
            .await
            .map_err(storage)?
    }

    pub async fn pull(&self, req: PullRequest) -> Result<PullResponse, ServerError> {
        tokio::time::sleep(self.latency).await;
        self.check_reachable()?;
        let dir = self.dir.clone();
        tokio::task::spawn_blocking(move || pull_blocking(&dir, req))
            .await
            .map_err(storage)?
    }
}

/// Ejecuta `f` con bloqueo exclusivo sobre la carpeta del servidor.
fn with_lock<R>(dir: &Path, f: impl FnOnce() -> Result<R, ServerError>) -> Result<R, ServerError> {
    fs::create_dir_all(dir).map_err(storage)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(LOCK_FILE))
        .map_err(storage)?;
    lock.lock().map_err(storage)?;
    f() // el bloqueo se libera al cerrar `lock`
}

fn read_log(dir: &Path) -> Result<Vec<SequencedChange>, ServerError> {
    let text = match fs::read_to_string(dir.join(LOG_FILE)) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(storage(e)),
    };
    let mut log = Vec::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        match serde_json::from_str(line) {
            Ok(entry) => log.push(entry),
            Err(e) => tracing::warn!("Línea {} del log del servidor ilegible, se omite: {}", n + 1, e),
        }
    }
    Ok(log)
}

fn push_blocking(dir: &Path, req: PushRequest) -> Result<PushResponse, ServerError> {
    with_lock(dir, || {
        let log = read_log(dir)?;
        let known: HashSet<_> = log.iter().map(|e| e.change.change_id).collect();
        let mut next_seq = log.last().map_or(1, |e| e.seq + 1);

        let mut new_entries = Vec::new();
        for change in &req.changes {
            if !known.contains(&change.change_id) {
                new_entries.push(SequencedChange { seq: next_seq, change: change.clone() });
                next_seq += 1;
            }
        }

        if !new_entries.is_empty() {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join(LOG_FILE))
                .map_err(storage)?;
            for entry in &new_entries {
                let line = serde_json::to_string(entry).map_err(storage)?;
                writeln!(file, "{line}").map_err(storage)?;
            }
            file.sync_all().map_err(storage)?;

            // Estado central materializado: lo que verán otros roles.
            let central_path = dir.join(CENTRAL_FILE);
            let mut central: DataSnapshot = read_json(&central_path).map_err(storage)?.unwrap_or_default();
            for entry in &new_entries {
                central.apply(&entry.change).map_err(storage)?;
            }
            write_json_atomic(&central_path, &central).map_err(storage)?;
        }

        tracing::info!(
            "Servidor simulado: {} cambios recibidos de {} ({} nuevos)",
            req.changes.len(),
            req.node_id,
            new_entries.len()
        );
        Ok(PushResponse {
            accepted: req.changes.iter().map(|c| c.change_id).collect(),
            server_seq: next_seq - 1,
        })
    })
}

fn pull_blocking(dir: &Path, req: PullRequest) -> Result<PullResponse, ServerError> {
    with_lock(dir, || {
        let log = read_log(dir)?;
        let server_seq = log.last().map_or(0, |e| e.seq);
        let changes = log
            .into_iter()
            .filter(|e| e.seq > req.since_seq && e.change.origin_node != req.node_id)
            .collect();
        Ok(PullResponse { changes, server_seq })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::temp_dir;
    use mamori_shared::sync::{ChangeRecord, ResidentRecord};
    use uuid::Uuid;

    fn change(node: Uuid, id: &str) -> ChangeRecord {
        let r = ResidentRecord {
            id: id.into(),
            full_name: id.into(),
            document: None,
            units: vec![],
            role: "Propietario".into(),
            mobile: None,
            email: None,
        };
        ChangeRecord::upsert(node, &r).unwrap()
    }

    #[tokio::test]
    async fn push_es_idempotente_y_pull_excluye_cambios_propios() {
        let server = SimServer::new(temp_dir("server"), Duration::ZERO);
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let c1 = change(a, "r1");

        let r1 = server.push(PushRequest { node_id: a, changes: vec![c1.clone()] }).await.unwrap();
        let r2 = server.push(PushRequest { node_id: a, changes: vec![c1.clone()] }).await.unwrap();
        assert_eq!(r1.server_seq, 1);
        assert_eq!(r2.server_seq, 1, "un change_id repetido no se duplica");
        assert_eq!(r2.accepted, vec![c1.change_id]);

        let own = server.pull(PullRequest { node_id: a, since_seq: 0 }).await.unwrap();
        assert!(own.changes.is_empty());
        let other = server.pull(PullRequest { node_id: b, since_seq: 0 }).await.unwrap();
        assert_eq!(other.changes.len(), 1);
        let none_new = server.pull(PullRequest { node_id: b, since_seq: 1 }).await.unwrap();
        assert!(none_new.changes.is_empty());
    }

    #[tokio::test]
    async fn sin_conexion_responde_no_disponible() {
        let dir = temp_dir("offline");
        fs::write(dir.join(OFFLINE_FLAG_FILE), "").unwrap();
        let server = SimServer::new(&dir, Duration::ZERO);
        let res = server.pull(PullRequest { node_id: Uuid::new_v4(), since_seq: 0 }).await;
        assert!(matches!(res, Err(ServerError::Unreachable)));
    }
}
