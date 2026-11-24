//! Almacenamiento local de la terminal en archivos JSON (D-22).
//!
//! Especificación: `specs/esquemas_bd/almacenamiento_local_poc.md`.
//! - `estado_local.json`: estado completo (`DataSnapshot`).
//! - `cola_envio.json`: cambios pendientes de confirmación del servidor (Outbox).
//! - `sync_meta.json`: identidad del nodo y estado de la sincronización.

use chrono::{DateTime, Utc};
use mamori_shared::sync::{ChangeRecord, DataSnapshot, SequencedChange, SyncRecord};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Datos iniciales del POC, embebidos en el ejecutable.
const SEED_JSON: &str = include_str!("../../../specs/mocks_dummy_data/seed_estado_local.json");

const STATE_FILE: &str = "estado_local.json";
const OUTBOX_FILE: &str = "cola_envio.json";
const META_FILE: &str = "sync_meta.json";

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("error de disco en {path}: {source}")]
    Io { path: PathBuf, source: std::io::Error },
    #[error("archivo con formato inválido {path}: {source}")]
    Json { path: PathBuf, source: serde_json::Error },
    #[error("no se pudo serializar el registro: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Identidad del nodo y estado de la última sincronización.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMeta {
    pub node_id: Uuid,
    /// Cursor: último `seq` del servidor ya recibido.
    pub server_seq: u64,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    /// Mensaje del último intento fallido; `None` si el último intento fue exitoso.
    pub last_error: Option<String>,
}

pub struct LocalStore {
    dir: PathBuf,
    pub data: DataSnapshot,
    pub outbox: Vec<ChangeRecord>,
    pub meta: SyncMeta,
}

/// Escritura atómica: archivo temporal + fsync + reemplazo (RN-SYNC-N04).
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), StoreError> {
    let io_err = |source| StoreError::Io { path: path.to_path_buf(), source };
    let bytes = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp).map_err(io_err)?;
        file.write_all(&bytes).map_err(io_err)?;
        file.sync_all().map_err(io_err)?;
    }
    fs::rename(&tmp, path).map_err(io_err)
}

/// Lee un JSON; devuelve `None` si el archivo no existe.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, StoreError> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|source| StoreError::Json { path: path.to_path_buf(), source }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(StoreError::Io { path: path.to_path_buf(), source }),
    }
}

impl LocalStore {
    /// Abre (o crea con los datos iniciales) el almacenamiento de la terminal en `dir`.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let dir = dir.into();
        fs::create_dir_all(&dir).map_err(|source| StoreError::Io { path: dir.clone(), source })?;

        let data = match read_json::<DataSnapshot>(&dir.join(STATE_FILE))? {
            Some(mut data) => {
                // Archivos anteriores a D-30 no tienen torres: se completan con las torres
                // iniciales (mismos ids en todas las terminales y en el servidor).
                if data.towers.is_empty() {
                    let seed: DataSnapshot = serde_json::from_str(SEED_JSON)?;
                    data.towers = seed.towers;
                    write_json_atomic(&dir.join(STATE_FILE), &data)?;
                    tracing::info!("Estado local actualizado: torres iniciales agregadas");
                }
                data
            }
            None => {
                let seed: DataSnapshot = serde_json::from_str(SEED_JSON)?;
                write_json_atomic(&dir.join(STATE_FILE), &seed)?;
                tracing::info!("Estado local creado con datos iniciales en {}", dir.display());
                seed
            }
        };
        let outbox = read_json(&dir.join(OUTBOX_FILE))?.unwrap_or_default();
        let meta = match read_json::<SyncMeta>(&dir.join(META_FILE))? {
            Some(meta) => meta,
            None => {
                let meta = SyncMeta {
                    node_id: Uuid::new_v4(),
                    server_seq: 0,
                    last_success_at: None,
                    last_attempt_at: None,
                    last_error: None,
                };
                write_json_atomic(&dir.join(META_FILE), &meta)?;
                meta
            }
        };
        Ok(Self { dir, data, outbox, meta })
    }

    pub fn node_id(&self) -> Uuid {
        self.meta.node_id
    }

    /// Guarda un registro creado o modificado en la terminal: lo aplica al estado,
    /// lo deja en la cola de envío y persiste ambos archivos (REQ-SYNC-02).
    pub fn record<T: SyncRecord>(&mut self, record: &T) -> Result<(), StoreError> {
        let change = ChangeRecord::upsert(self.meta.node_id, record)?;
        self.data.apply(&change)?;
        // RN-SYNC-03: en la cola solo queda la última versión de cada registro, y queda
        // en la POSICIÓN de la primera: así un inmueble creado antes que sus
        // parqueaderos sigue llegando antes al servidor aunque se edite después
        // (llave foránea parking_spots → units).
        match self.outbox.iter_mut().find(|c| c.key() == change.key()) {
            Some(pending) => *pending = change,
            None => self.outbox.push(change),
        }
        self.save_data()?;
        self.save_outbox()
    }

    /// Retira de la cola los cambios confirmados por el servidor (REQ-SYNC-05, RN-SYNC-N02).
    pub fn acknowledge(&mut self, accepted: &[Uuid]) -> Result<(), StoreError> {
        let accepted: HashSet<&Uuid> = accepted.iter().collect();
        let before = self.outbox.len();
        self.outbox.retain(|c| !accepted.contains(&c.change_id));
        if self.outbox.len() != before {
            self.save_outbox()?;
        }
        Ok(())
    }

    /// Aplica cambios de otros nodos. Gana la terminal: se ignoran los que chocan
    /// con un cambio local pendiente (RN-SYNC-04). Devuelve cuántos se aplicaron.
    pub fn apply_remote(&mut self, changes: &[SequencedChange], server_seq: u64) -> Result<usize, StoreError> {
        let mut applied = 0;
        for sc in changes {
            let pending_local = self.outbox.iter().any(|c| c.key() == sc.change.key());
            if pending_local {
                tracing::info!(
                    "Cambio remoto ignorado (gana la terminal): {:?}/{}",
                    sc.change.collection,
                    sc.change.record_id
                );
                continue;
            }
            self.data.apply(&sc.change)?;
            applied += 1;
        }
        if applied > 0 {
            self.save_data()?;
        }
        self.meta.server_seq = self.meta.server_seq.max(server_seq);
        self.save_meta()?;
        Ok(applied)
    }

    pub fn mark_attempt(&mut self, at: DateTime<Utc>, error: Option<String>) -> Result<(), StoreError> {
        self.meta.last_attempt_at = Some(at);
        if error.is_none() {
            self.meta.last_success_at = Some(at);
        }
        self.meta.last_error = error;
        self.save_meta()
    }

    fn save_data(&self) -> Result<(), StoreError> {
        write_json_atomic(&self.dir.join(STATE_FILE), &self.data)
    }

    fn save_outbox(&self) -> Result<(), StoreError> {
        write_json_atomic(&self.dir.join(OUTBOX_FILE), &self.outbox)
    }

    fn save_meta(&self) -> Result<(), StoreError> {
        write_json_atomic(&self.dir.join(META_FILE), &self.meta)
    }
}

#[cfg(test)]
pub(crate) fn temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mamori-test-{}-{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use mamori_shared::sync::ResidentRecord;

    fn resident(id: &str, name: &str) -> ResidentRecord {
        ResidentRecord {
            id: id.into(),
            full_name: name.into(),
            document: None,
            units: vec![],
            role: "Propietario".into(),
            mobile: None,
            email: None,
        }
    }

    #[test]
    fn primera_apertura_carga_datos_iniciales_sin_cola() {
        let store = LocalStore::open(temp_dir("seed")).unwrap();
        assert_eq!(store.data.units.len(), 6);
        assert_eq!(store.data.visits.len(), 4);
        assert!(store.outbox.is_empty());
    }

    #[test]
    fn archivo_antiguo_sin_torres_recibe_las_torres_iniciales() {
        let dir = temp_dir("legacy");
        fs::write(
            dir.join(STATE_FILE),
            r#"{"units":[],"parking_spots":[],"residents":[],"visits":[],"packages":[]}"#,
        )
        .unwrap();
        let store = LocalStore::open(&dir).unwrap();
        assert_eq!(store.data.towers.len(), 4);
        assert!(store.outbox.is_empty(), "las torres iniciales no se envían al servidor");
    }

    #[test]
    fn los_cambios_sobreviven_a_reabrir() {
        let dir = temp_dir("persist");
        let node = {
            let mut store = LocalStore::open(&dir).unwrap();
            store.record(&resident("r9", "Nuevo")).unwrap();
            store.node_id()
        };
        let store = LocalStore::open(&dir).unwrap();
        assert_eq!(store.node_id(), node);
        assert!(store.data.residents.iter().any(|r| r.id == "r9"));
        assert_eq!(store.outbox.len(), 1);
    }

    #[test]
    fn la_cola_guarda_solo_la_ultima_version() {
        let mut store = LocalStore::open(temp_dir("compact")).unwrap();
        store.record(&resident("r9", "A")).unwrap();
        store.record(&resident("r9", "B")).unwrap();
        assert_eq!(store.outbox.len(), 1);
        assert_eq!(store.outbox[0].payload.as_ref().unwrap()["full_name"], "B");
    }

    #[test]
    fn editar_un_pendiente_conserva_su_posicion_en_la_cola() {
        let mut store = LocalStore::open(temp_dir("order")).unwrap();
        store.record(&resident("r8", "Primero")).unwrap();
        store.record(&resident("r9", "Segundo")).unwrap();
        store.record(&resident("r8", "Primero editado")).unwrap();
        let ids: Vec<&str> = store.outbox.iter().map(|c| c.record_id.as_str()).collect();
        assert_eq!(ids, vec!["r8", "r9"]);
        assert_eq!(store.outbox[0].payload.as_ref().unwrap()["full_name"], "Primero editado");
    }

    #[test]
    fn confirmacion_retira_solo_lo_confirmado() {
        let mut store = LocalStore::open(temp_dir("ack")).unwrap();
        store.record(&resident("r8", "A")).unwrap();
        store.record(&resident("r9", "B")).unwrap();
        let first = store.outbox[0].change_id;
        store.acknowledge(&[first]).unwrap();
        assert_eq!(store.outbox.len(), 1);
        assert_eq!(store.outbox[0].record_id, "r9");
    }

    #[test]
    fn gana_la_terminal_ante_conflicto() {
        let mut store = LocalStore::open(temp_dir("conflict")).unwrap();
        store.record(&resident("r1", "Versión local")).unwrap();
        let other_node = Uuid::new_v4();
        let remote = vec![
            SequencedChange { seq: 1, change: ChangeRecord::upsert(other_node, &resident("r1", "Versión remota")).unwrap() },
            SequencedChange { seq: 2, change: ChangeRecord::upsert(other_node, &resident("r2", "Otro")).unwrap() },
        ];
        let applied = store.apply_remote(&remote, 2).unwrap();
        assert_eq!(applied, 1);
        let r1 = store.data.residents.iter().find(|r| r.id == "r1").unwrap();
        assert_eq!(r1.full_name, "Versión local");
        assert_eq!(store.meta.server_seq, 2);
    }
}
