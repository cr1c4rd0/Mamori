//! Motor de sincronización de la terminal (D-23, D-24, D-26).
//!
//! Reglas: `specs/reglas_negocio/RN_sincronizacion_local_first.md`.

use crate::store::{LocalStore, StoreError};
use crate::transport::{ServerError, SyncServer};
use crate::AppWindow;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use mamori_shared::sync::{PullRequest, PushRequest};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::Notify;

/// Ventanas de sincronización fuera de hora pico, en hora local (RN-SYNC-01).
pub const SYNC_HOURS: [u32; 4] = [3, 10, 15, 22];

/// Cada cuánto se revisa el reloj para detectar una ventana.
const CLOCK_CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Tiempo mínimo que se muestra "SINCRONIZANDO…" en el encabezado.
const MIN_SYNCING_VISIBLE: Duration = Duration::from_millis(1000);

pub type SharedStore = Arc<Mutex<LocalStore>>;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Server(#[from] ServerError),
    #[error("error del almacenamiento local: {0}")]
    Local(#[from] StoreError),
}

/// Bloquea el almacenamiento; si otro hilo falló con el candado tomado, se sigue
/// usando el último estado (los archivos en disco son la fuente de verdad).
pub fn lock(store: &SharedStore) -> MutexGuard<'_, LocalStore> {
    store.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// `true` si entre `prev` (excluido) y `now` (incluido) pasó alguna ventana de sincronización.
pub fn window_crossed(prev: NaiveDateTime, now: NaiveDateTime) -> bool {
    if now <= prev {
        return false;
    }
    let mut day = prev.date();
    while day <= now.date() {
        for hour in SYNC_HOURS {
            if let Some(slot) = day.and_hms_opt(hour, 0, 0) {
                if slot > prev && slot <= now {
                    return true;
                }
            }
        }
        match day.succ_opt() {
            Some(next) => day = next,
            None => break,
        }
    }
    false
}

/// Estado que muestra el indicador del encabezado.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncView {
    Syncing,
    Idle { last_success: Option<DateTime<Utc>> },
    Failed { last_success: Option<DateTime<Utc>> },
}

impl SyncView {
    pub fn from_store(store: &LocalStore) -> Self {
        let last_success = store.meta.last_success_at;
        if store.meta.last_error.is_some() {
            SyncView::Failed { last_success }
        } else {
            SyncView::Idle { last_success }
        }
    }

    /// `(estado, título, detalle)` para la UI. Estados: `syncing`, `ok`, `never`, `error`.
    pub fn labels(&self) -> (&'static str, String, String) {
        let fmt = |t: &Option<DateTime<Utc>>| match t {
            Some(t) => t.with_timezone(&Local).format("%d/%m/%Y %H:%M").to_string(),
            None => "Nunca".to_string(),
        };
        match self {
            SyncView::Syncing => ("syncing", "SINCRONIZANDO…".into(), "Enviando y recibiendo datos".into()),
            SyncView::Idle { last_success: None } => ("never", "ÚLTIMA SINCRONIZACIÓN".into(), "Nunca".into()),
            SyncView::Idle { last_success } => ("ok", "ÚLTIMA SINCRONIZACIÓN".into(), fmt(last_success)),
            SyncView::Failed { last_success } => ("error", "SIN CONEXIÓN".into(), format!("Última: {}", fmt(last_success))),
        }
    }
}

/// Una sincronización completa: Push de la cola y luego Pull (RN-SYNC-02).
/// Devuelve cuántos cambios de otros nodos se aplicaron.
pub async fn run_sync(store: &SharedStore, server: &SyncServer) -> Result<usize, SyncError> {
    let (node_id, outbox, since_seq) = {
        let s = lock(store);
        (s.node_id(), s.outbox.clone(), s.meta.server_seq)
    };

    let accepted = if outbox.is_empty() {
        Vec::new()
    } else {
        server.push(PushRequest { node_id, changes: outbox }).await?.accepted
    };

    let pull = server.pull(PullRequest { node_id, since_seq }).await?;

    // Lo enviado se retira de la cola DESPUÉS de aplicar lo recibido: así un cambio
    // remoto del mismo registro sigue chocando con la versión local y gana la
    // terminal (RN-SYNC-04). Si algo falla antes, la cola se reenvía en la próxima
    // ventana sin duplicarse en el servidor (RN-SYNC-05).
    let mut s = lock(store);
    let applied = s.apply_remote(&pull.changes, pull.server_seq)?;
    s.acknowledge(&accepted)?;
    Ok(applied)
}

fn publish(ui: &slint::Weak<AppWindow>, store: &SharedStore, view: SyncView, refresh_data: bool) {
    let ui = ui.clone();
    let store = store.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            crate::ui_bind::set_sync_indicator(&ui, &view);
            if refresh_data {
                crate::ui_bind::refresh_all(&ui, &lock(&store));
            }
        }
    });
}

async fn sync_once(store: &SharedStore, server: &SyncServer, ui: &slint::Weak<AppWindow>, reason: &str) {
    publish(ui, store, SyncView::Syncing, false);
    tracing::info!("Sincronización iniciada ({}) contra {}", reason, server.describe());

    let started = std::time::Instant::now();
    let result = run_sync(store, server).await;
    // Contra un servidor local tarda milisegundos: se mantiene el estado visible para
    // que el vigilante perciba que la sincronización ocurrió.
    if let Some(rest) = MIN_SYNCING_VISIBLE.checked_sub(started.elapsed()) {
        tokio::time::sleep(rest).await;
    }
    let error = match &result {
        Ok(applied) => {
            tracing::info!("Sincronización exitosa: {} cambios recibidos aplicados", applied);
            None
        }
        Err(e) => {
            tracing::warn!("Sincronización fallida: {}", e);
            Some(e.to_string())
        }
    };

    let view = {
        let mut s = lock(store);
        if let Err(e) = s.mark_attempt(Utc::now(), error) {
            tracing::error!("No se pudo guardar el estado de sincronización: {}", e);
        }
        SyncView::from_store(&s)
    };
    let refresh = matches!(result, Ok(n) if n > 0);
    publish(ui, store, view, refresh);
}

/// Permite pedir una sincronización manual (botón `SINCRONIZAR` / `F9`, D-28).
#[derive(Clone)]
pub struct SyncHandle {
    trigger: Arc<Notify>,
    busy: Arc<AtomicBool>,
}

impl SyncHandle {
    /// Pide una sincronización inmediata. Devuelve `false` si ya hay una en curso (RN-SYNC-N07).
    pub fn request_now(&self) -> bool {
        if self.busy.load(Ordering::SeqCst) {
            return false;
        }
        self.trigger.notify_one();
        true
    }
}

/// Lanza el planificador en segundo plano. Con `MAMORI_SYNC_INTERVAL_SECS` (solo
/// desarrollo) sincroniza cada N segundos en lugar de usar las ventanas fijas.
pub fn spawn_scheduler(store: SharedStore, server: SyncServer, ui: slint::Weak<AppWindow>) -> SyncHandle {
    let handle = SyncHandle { trigger: Arc::new(Notify::new()), busy: Arc::new(AtomicBool::new(false)) };
    let SyncHandle { trigger, busy } = handle.clone();

    let demo_interval = std::env::var("MAMORI_SYNC_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&secs| secs > 0)
        .map(Duration::from_secs);

    match demo_interval {
        Some(d) => tracing::warn!("Modo demo: sincronización cada {:?} (no usar en producción)", d),
        None => tracing::info!("Sincronización programada a las {:?} h (hora local)", SYNC_HOURS),
    }

    tokio::spawn(async move {
        let mut prev = Local::now().naive_local();
        loop {
            let reason = tokio::select! {
                _ = trigger.notified() => Some("manual"),
                _ = tokio::time::sleep(demo_interval.unwrap_or(CLOCK_CHECK_INTERVAL)) => {
                    if demo_interval.is_some() {
                        Some("modo demo")
                    } else {
                        let now = Local::now().naive_local();
                        let crossed = window_crossed(prev, now);
                        prev = now;
                        crossed.then_some("ventana programada")
                    }
                }
            };
            if let Some(reason) = reason {
                busy.store(true, Ordering::SeqCst);
                sync_once(&store, &server, &ui, reason).await;
                busy.store(false, Ordering::SeqCst);
            }
        }
    });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::temp_dir;
    use chrono::NaiveDate;
    use mamori_shared::sync::ResidentRecord;

    fn at(d: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn detecta_las_ventanas_fijas() {
        assert!(window_crossed(at(24, 9, 59), at(24, 10, 0)));
        assert!(window_crossed(at(24, 14, 59), at(24, 15, 1)));
        assert!(window_crossed(at(24, 21, 59), at(24, 22, 0)));
        assert!(window_crossed(at(24, 2, 59), at(24, 3, 0)));
        assert!(!window_crossed(at(24, 10, 1), at(24, 14, 59)));
        assert!(!window_crossed(at(24, 10, 0), at(24, 10, 0)));
    }

    #[test]
    fn detecta_ventana_al_cruzar_medianoche() {
        assert!(window_crossed(at(24, 23, 30), at(25, 3, 0)));
        assert!(!window_crossed(at(24, 23, 30), at(25, 2, 59)));
    }

    #[test]
    fn etiquetas_del_indicador() {
        assert_eq!(SyncView::Syncing.labels().0, "syncing");
        assert_eq!(SyncView::Idle { last_success: None }.labels().2, "Nunca");
        let (state, title, _) = SyncView::Failed { last_success: None }.labels();
        assert_eq!((state, title.as_str()), ("error", "SIN CONEXIÓN"));
    }

    #[tokio::test]
    async fn dos_terminales_se_sincronizan_y_gana_la_terminal() {
        let server = SyncServer::Sim(crate::sim_server::SimServer::new(temp_dir("srv"), Duration::ZERO));
        let a: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("a")).unwrap()));
        let b: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("b")).unwrap()));

        let mut r1 = lock(&a).data.residents[0].clone();
        r1.full_name = "Editado en A".into();
        lock(&a).record(&r1).unwrap();
        let nuevo = ResidentRecord { id: "nuevo-b".into(), ..r1.clone() };
        lock(&b).record(&ResidentRecord { full_name: "Editado en B".into(), ..r1.clone() }).unwrap();
        lock(&b).record(&nuevo).unwrap();

        // B sincroniza primero; luego A, que tiene pendiente su versión de r1.
        run_sync(&b, &server).await.unwrap();
        run_sync(&a, &server).await.unwrap();
        let a_state = lock(&a);
        let a_r1 = a_state.data.residents.iter().find(|r| r.id == r1.id).unwrap();
        assert_eq!(a_r1.full_name, "Editado en A", "gana la terminal");
        assert!(a_state.data.residents.iter().any(|r| r.id == "nuevo-b"), "llega lo nuevo de B");
        assert!(a_state.outbox.is_empty());
        drop(a_state);

        // B recibe la versión de A en su siguiente sincronización.
        run_sync(&b, &server).await.unwrap();
        let b_state = lock(&b);
        let b_r1 = b_state.data.residents.iter().find(|r| r.id == r1.id).unwrap();
        assert_eq!(b_r1.full_name, "Editado en A");
    }

    #[tokio::test]
    async fn fallo_de_conexion_conserva_la_cola() {
        let dir = temp_dir("srv-off");
        std::fs::write(dir.join(crate::sim_server::OFFLINE_FLAG_FILE), "").unwrap();
        let server = SyncServer::Sim(crate::sim_server::SimServer::new(&dir, Duration::ZERO));
        let a: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("a-off")).unwrap()));
        let r = lock(&a).data.residents[0].clone();
        lock(&a).record(&r).unwrap();
        assert!(run_sync(&a, &server).await.is_err());
        assert_eq!(lock(&a).outbox.len(), 1);
    }

    #[tokio::test]
    async fn servidor_http_apagado_es_sin_conexion() {
        // Puerto 9 (discard): nadie escucha ahí en un equipo normal.
        let server = SyncServer::Http(crate::transport::HttpServer::new("http://127.0.0.1:9").unwrap());
        let a: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("a-http-off")).unwrap()));
        let err = run_sync(&a, &server).await.unwrap_err();
        assert!(matches!(err, SyncError::Server(ServerError::Unreachable)), "{err}");
    }

    /// Escenario de dos terminales contra el servidor real (Axum + PostgreSQL).
    /// Requiere el servidor corriendo: `cargo test -p mamori-client -- --ignored`
    #[tokio::test]
    #[ignore = "requiere mamori-server corriendo en MAMORI_SERVER_URL o 127.0.0.1:8080"]
    async fn dos_terminales_contra_servidor_real() {
        let url = std::env::var("MAMORI_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let server = SyncServer::Http(crate::transport::HttpServer::new(&url).unwrap());
        let a: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("a-real")).unwrap()));
        let b: SharedStore = Arc::new(Mutex::new(LocalStore::open(temp_dir("b-real")).unwrap()));
        // Ponerse al día con lo que ya haya en el servidor antes de probar.
        run_sync(&a, &server).await.unwrap();
        run_sync(&b, &server).await.unwrap();

        let id = format!("test-{}", uuid::Uuid::new_v4());
        let base = ResidentRecord {
            id: id.clone(),
            full_name: "Creado en B".into(),
            document: None,
            units: vec!["Torre 1 - Apto 101".into()],
            role: "Propietario".into(),
            mobile: None,
            email: None,
        };
        lock(&b).record(&base).unwrap();
        run_sync(&b, &server).await.unwrap();
        run_sync(&a, &server).await.unwrap();
        assert!(lock(&a).data.residents.iter().any(|r| r.id == id), "A recibe lo creado por B");

        // Conflicto: ambos editan; B sincroniza primero, A después → gana A (terminal con pendiente).
        lock(&a).record(&ResidentRecord { full_name: "Editado en A".into(), ..base.clone() }).unwrap();
        lock(&b).record(&ResidentRecord { full_name: "Editado en B".into(), ..base.clone() }).unwrap();
        run_sync(&b, &server).await.unwrap();
        run_sync(&a, &server).await.unwrap();
        run_sync(&b, &server).await.unwrap();
        let name_in = |s: &SharedStore| lock(s).data.residents.iter().find(|r| r.id == id).unwrap().full_name.clone();
        assert_eq!(name_in(&a), "Editado en A");
        assert_eq!(name_in(&b), "Editado en A");
        assert!(lock(&a).outbox.is_empty() && lock(&b).outbox.is_empty());

        // Limpieza: borrar el residente de prueba por el propio protocolo, para que las
        // terminales reales no lo reciban.
        let node = lock(&a).node_id();
        let mut del = mamori_shared::sync::ChangeRecord::upsert(node, &base).unwrap();
        del.op = mamori_shared::sync::ChangeOp::Delete;
        del.payload = None;
        server.push(PushRequest { node_id: node, changes: vec![del] }).await.unwrap();
    }
}
