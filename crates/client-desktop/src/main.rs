slint::include_modules!();

mod sim_server;
mod store;
mod sync;
mod transport;
mod ui_bind;
mod units_crud;

use chrono::Utc;
use mamori_shared::sync::{ParkingSpotRecord, ResidentRecord, SyncRecord};
use sim_server::SimServer;
use transport::{HttpServer, SyncServer};
use slint::{Model, VecModel};
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use store::LocalStore;
use sync::{lock, SharedStore, SyncView};
use ui_bind::non_empty;

/// Latencia simulada de red del servidor simulado (para ver el estado "Sincronizando…").
const SIM_NETWORK_LATENCY: Duration = Duration::from_millis(1200);
/// Servidor real local por defecto (D-29).
const DEFAULT_SERVER_URL: &str = "http://127.0.0.1:8080";

/// Servidor de sincronización: real (por defecto) o simulado con `MAMORI_SERVER=sim`.
fn sync_server() -> Result<SyncServer, transport::ServerError> {
    if std::env::var("MAMORI_SERVER").is_ok_and(|v| v.eq_ignore_ascii_case("sim")) {
        let dir = data_dir("MAMORI_SERVER_DIR", "servidor_simulado");
        return Ok(SyncServer::Sim(SimServer::new(dir, SIM_NETWORK_LATENCY)));
    }
    let url = std::env::var("MAMORI_SERVER_URL").unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string());
    Ok(SyncServer::Http(HttpServer::new(&url)?))
}

/// Muestra una alerta flotante en la parte inferior de la pantalla con cierre automático (4s).
fn show_bottom_alert(ui: &AppWindow, kind: &str, message: &str) {
    ui.set_alert_kind(kind.into());
    ui.set_alert_message(message.into());
    ui.set_show_alert(true);

    let ui_handle = ui.as_weak();
    let expected_msg = message.to_string();
    slint::Timer::single_shot(Duration::from_secs(4), move || {
        if let Some(ui) = ui_handle.upgrade() {
            if ui.get_alert_message().as_str() == expected_msg {
                ui.set_show_alert(false);
            }
        }
    });
}

/// Guarda un registro en el almacenamiento local (disco + cola de envío) y refresca la UI.
/// Devuelve `false` y avisa al vigilante si no se pudo escribir en disco.
fn persist<T: SyncRecord>(ui: &AppWindow, store: &SharedStore, records: &[T]) -> bool {
    let mut s = lock(store);
    for record in records {
        if let Err(e) = s.record(record) {
            tracing::error!("No se pudo guardar en disco: {}", e);
            drop(s);
            show_bottom_alert(ui, "error", "No se pudo guardar en el disco de la terminal. Avise al administrador.");
            return false;
        }
    }
    ui_bind::refresh_all(ui, &s);
    true
}

/// Guarda todos los registros de una operación del CRUD de Unidades & Predios.
/// Orden: torres → inmuebles → parqueaderos → residentes (las llaves foráneas del
/// servidor reciben primero el inmueble y luego sus parqueaderos).
fn persist_plan(ui: &AppWindow, store: &SharedStore, plan: &units_crud::Plan) -> bool {
    let mut s = lock(store);
    let mut save = || -> Result<(), store::StoreError> {
        plan.towers.iter().try_for_each(|r| s.record(r))?;
        plan.units.iter().try_for_each(|r| s.record(r))?;
        plan.parking_spots.iter().try_for_each(|r| s.record(r))?;
        plan.residents.iter().try_for_each(|r| s.record(r))
    };
    if let Err(e) = save() {
        tracing::error!("No se pudo guardar en disco: {}", e);
        drop(s);
        show_bottom_alert(ui, "error", "No se pudo guardar en el disco de la terminal. Avise al administrador.");
        return false;
    }
    ui_bind::refresh_all(ui, &s);
    true
}

/// Lee el formulario del panel de inmueble.
fn unit_form(ui: &AppWindow, parking: &VecModel<slint::SharedString>) -> units_crud::UnitForm {
    units_crud::UnitForm {
        unit_number: ui.get_form_unit_number().to_string(),
        unit_type: ui.get_form_unit_type().to_string(),
        intercom: non_empty(&ui.get_form_intercom()),
        mobile: non_empty(&ui.get_form_mobile()),
        landline: non_empty(&ui.get_form_landline()),
        parking: parking.iter().map(|p| parking_code(&p).to_string()).collect(),
    }
}

/// Carpeta de datos: variable de entorno o `%APPDATA%\Mamori\<sub>`.
fn data_dir(env_var: &str, sub: &str) -> PathBuf {
    if let Ok(dir) = std::env::var(env_var) {
        return PathBuf::from(dir);
    }
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Mamori").join(sub)
}

/// Código de una opción `P-104 (Carro)` → `P-104`.
fn parking_code(display: &str) -> &str {
    display.split(" (").next().unwrap_or(display).trim()
}

/// Filtra en tiempo real los parqueaderos disponibles:
/// 1. Excluye cualquier parqueadero ya seleccionado para el inmueble en edición.
/// 2. Aplica el filtro de búsqueda por código o tipo (ej: "105", "Carro", "Moto").
fn filter_and_update_picker(
    ui: &AppWindow,
    master_catalog: &[String],
    selected_model: &VecModel<slint::SharedString>,
    query: &str,
) {
    let selected_set: HashSet<String> = (0..selected_model.row_count())
        .filter_map(|i| selected_model.row_data(i).map(|s| s.to_string()))
        .collect();

    let query_lower = query.trim().to_lowercase();

    let items: Vec<ParkingPickerItem> = master_catalog
        .iter()
        .filter(|spot| !selected_set.contains(*spot))
        .filter(|spot| query_lower.is_empty() || spot.to_lowercase().contains(&query_lower))
        .map(|spot| {
            let (code, spot_type) = if let Some(idx) = spot.find(" (") {
                let code = &spot[..idx];
                let spot_type = spot[idx + 2..].trim_end_matches(')');
                (code.to_string(), spot_type.to_string())
            } else {
                (spot.clone(), "General".to_string())
            };

            ParkingPickerItem {
                code: code.into(),
                spot_type: spot_type.into(),
                display_name: spot.clone().into(),
            }
        })
        .collect();

    let picker_model = Rc::new(VecModel::from(items));
    ui.set_filtered_parking_options(picker_model.into());
}

/// Filtra en tiempo real los inmuebles registrados para asociarlos a un residente,
/// excluyendo los inmuebles que ya fueron seleccionados en el formulario actual.
fn filter_and_update_unit_picker(
    ui: &AppWindow,
    units_model: &VecModel<UnitItem>,
    selected_units_model: &VecModel<slint::SharedString>,
    query: &str,
) {
    let selected_set: HashSet<String> = (0..selected_units_model.row_count())
        .filter_map(|i| selected_units_model.row_data(i).map(|s| s.to_string()))
        .collect();

    let query_lower = query.trim().to_lowercase();
    let mut items = Vec::new();

    for i in 0..units_model.row_count() {
        if let Some(u) = units_model.row_data(i) {
            let display = format!("{} - {}", u.tower, u.unit_number);
            if selected_set.contains(&display) {
                continue;
            }
            if !query_lower.is_empty()
                && !display.to_lowercase().contains(&query_lower)
                && !u.unit_type.to_lowercase().contains(&query_lower)
            {
                continue;
            }
            items.push(UnitPickerItem {
                tower: u.tower,
                unit_number: u.unit_number,
                unit_type: u.unit_type,
                display_name: display.into(),
            });
        }
    }

    let picker_model = Rc::new(VecModel::from(items));
    ui.set_filtered_unit_options(picker_model.into());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Mamori Terminal Windows Nativa (Slint)...");

    // Almacenamiento local (D-22) y servidor de sincronización (D-29)
    let terminal_dir = data_dir("MAMORI_DATA_DIR", "terminal");
    let server = sync_server()?;
    let store: SharedStore = Arc::new(Mutex::new(LocalStore::open(&terminal_dir)?));
    tracing::info!(
        "Datos locales: {} | Sincroniza con: {} | Nodo: {}",
        terminal_dir.display(),
        server.describe(),
        lock(&store).node_id()
    );

    let ui = AppWindow::new()?;

    // Modelos reactivos; su contenido siempre se pinta desde el almacenamiento local.
    let units_model = Rc::new(VecModel::<UnitItem>::default());
    ui.set_units(units_model.clone().into());
    ui.set_tower_units(Rc::new(VecModel::<UnitItem>::default()).into());
    ui.set_towers(Rc::new(VecModel::<TowerItem>::default()).into());
    let active_visits_model = Rc::new(VecModel::<ActiveVisitItem>::default());
    ui.set_active_visits(active_visits_model.clone().into());
    let pending_packages_model = Rc::new(VecModel::<PendingPackageItem>::default());
    ui.set_pending_packages(pending_packages_model.clone().into());
    let residents_model = Rc::new(VecModel::<ResidentItem>::default());
    ui.set_residents(residents_model.clone().into());
    {
        let s = lock(&store);
        ui_bind::refresh_all(&ui, &s);
        ui_bind::set_sync_indicator(&ui, &SyncView::from_store(&s));
    }

    // Modelo reactivo para las filas dinámicas de parqueaderos asignados al inmueble en edición
    let selected_parking_model = Rc::new(VecModel::default());
    ui.set_selected_parking_list(selected_parking_model.clone().into());

    // Callback para cerrar manualmente la alerta inferior
    ui.on_dismiss_alert({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_alert(false);
            }
        }
    });

    // Callbacks de Autenticación (Login / Cerrar Sesión)
    ui.on_attempt_login({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let username = ui.get_login_username().trim().to_lowercase();
                let password = ui.get_login_password();

                let auth_result = match (username.as_str(), password.as_str()) {
                    ("porteria", "1234") => Some(("Cristian Ricardo Rojas", "Portería")),
                    ("admin", "1234") => Some(("Cristian Ricardo Rojas", "Administrador")),
                    _ => None,
                };

                if let Some((full_name, role)) = auth_result {
                    tracing::info!("Sesión iniciada: {} ({})", full_name, role);
                    ui.set_current_user_name(full_name.into());
                    ui.set_current_user_role(role.into());
                    ui.set_login_password("".into());
                    ui.set_login_error("".into());
                    ui.set_show_alert(false);
                    ui.set_active_module("home".into());
                    ui.set_current_view_title("DASHBOARD".into());
                    ui.set_status_text("DASHBOARD".into());
                    ui.set_is_authenticated(true);
                } else {
                    tracing::warn!("Intento de acceso fallido para usuario: {}", username);
                    ui.set_login_error("".into());
                    ui.set_login_password("".into());
                    show_bottom_alert(&ui, "error", "Usuario o contraseña incorrectos. Verifique e intente nuevamente.");
                }
            }
        }
    });

    ui.on_logout({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                tracing::info!("Sesión cerrada por el usuario.");
                ui.set_show_drawer(false);
                ui.set_show_parking_modal(false);
                ui.set_show_parking_picker(false);
                ui.set_show_resident_drawer(false);
                ui.set_login_username("".into());
                ui.set_login_password("".into());
                ui.set_login_error("".into());
                ui.set_active_module("home".into());
                ui.set_current_view_title("DASHBOARD".into());
                ui.set_is_authenticated(false);
            }
        }
    });

    // Callbacks de navegación entre módulos
    ui.on_quick_action({
        let ui_handle = ui.as_weak();
        move |action| {
            if let Some(ui) = ui_handle.upgrade() {
                let view_title = match action.as_str() {
                    "home" => "DASHBOARD",
                    "visita" => "VISITAS",
                    "vehiculo" => "VEHÍCULOS",
                    "paquete" => "PAQUETERÍA",
                    "residentes" => "RESIDENTES",
                    "minuta" => "MINUTA & NOVEDADES",
                    "unidades" => "UNIDADES & PREDIOS",
                    "config" => "CONFIGURACIÓN",
                    other => other,
                };
                tracing::info!("Vista seleccionada: {} ({})", view_title, action);
                ui.set_active_module(action.clone());
                ui.set_current_view_title(view_title.into());
                ui.set_status_text(view_title.into());
            }
        }
    });

    // Callback para seleccionar torre
    ui.on_select_tower({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        move |tower_name| {
            if let Some(ui) = ui_handle.upgrade() {
                tracing::info!("Torre seleccionada: {}", tower_name);
                ui.set_selected_tower(tower_name.clone());
                ui_bind::refresh_all(&ui, &lock(&store));
                ui.set_status_text(format!("Visualizando predios de: {}", tower_name).into());
            }
        }
    });

    // ===== CRUD de Torres / Bloques (D-30) =====
    ui.on_new_tower({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_editing_tower_id("".into());
                ui.set_form_tower_name("".into());
                ui.set_confirm_delete_tower(false);
                ui.set_show_tower_drawer(true);
            }
        }
    });

    ui.on_edit_tower({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let selected = ui.get_selected_tower();
            let tower = lock(&store).data.towers.iter().find(|t| t.is_active && t.name == selected.as_str()).cloned();
            if let Some(t) = tower {
                ui.set_editing_tower_id(t.id.into());
                ui.set_form_tower_name(t.name.into());
                ui.set_confirm_delete_tower(false);
                ui.set_show_tower_drawer(true);
            }
        }
    });

    ui.on_cancel_tower_drawer({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_confirm_delete_tower(false);
                ui.set_show_tower_drawer(false);
            }
        }
    });

    ui.on_save_tower({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let id = ui.get_editing_tower_id();
            let name = ui.get_form_tower_name();
            let result = {
                let s = lock(&store);
                if id.is_empty() {
                    units_crud::create_tower(&s.data, &name)
                } else {
                    units_crud::rename_tower(&s.data, &id, &name)
                }
            };
            match result {
                Err(e) => show_bottom_alert(&ui, "error", &e.to_string()),
                Ok(plan) => {
                    let new_name = plan.towers.first().map(|t| t.name.clone());
                    if persist_plan(&ui, &store, &plan) {
                        if let Some(n) = new_name {
                            ui.set_selected_tower(n.clone().into());
                            ui_bind::refresh_all(&ui, &lock(&store));
                            let verb = if id.is_empty() { "creada" } else { "actualizada" };
                            tracing::info!("Torre {}: {}", verb, n);
                            ui.set_status_text(format!("{} {}.", n, verb).into());
                        }
                        ui.set_show_tower_drawer(false);
                    }
                }
            }
        }
    });

    ui.on_delete_tower({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let id = ui.get_editing_tower_id();
            let result = units_crud::delete_tower(&lock(&store).data, &id, &ui.get_current_user_role());
            match result {
                Err(e) => {
                    ui.set_confirm_delete_tower(false);
                    show_bottom_alert(&ui, "error", &e.to_string());
                }
                Ok(plan) => {
                    let name = plan.towers.first().map(|t| t.name.clone()).unwrap_or_default();
                    if persist_plan(&ui, &store, &plan) {
                        tracing::info!("Torre eliminada (inactivada): {}", name);
                        ui.set_status_text(format!("{} eliminada.", name).into());
                        ui.set_show_tower_drawer(false);
                    }
                }
            }
        }
    });

    // Callback Dashboard Home: Registrar salida de visita activa (se conserva con hora de salida)
    ui.on_register_visit_exit({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        let model = active_visits_model.clone();
        move |idx| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let Some(id) = ui_bind::row_id(&*model, idx, |v: &ActiveVisitItem| v.id.to_string()) else {
                return;
            };
            let visit = lock(&store).data.visits.iter().find(|v| v.id == id).cloned();
            if let Some(mut visit) = visit {
                visit.exit_at = Some(Utc::now());
                if persist(&ui, &store, &[visit.clone()]) {
                    tracing::info!("Salida registrada: {} ({})", visit.visitor_name, visit.destination);
                    ui.set_status_text(
                        format!("Salida registrada: {} ({})", visit.visitor_name, visit.destination).into(),
                    );
                }
            }
        }
    });

    // Callback Dashboard Home: Marcar paquete como entregado (se conserva con hora de entrega)
    ui.on_deliver_package({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        let model = pending_packages_model.clone();
        move |idx| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let Some(id) = ui_bind::row_id(&*model, idx, |p: &PendingPackageItem| p.id.to_string()) else {
                return;
            };
            let package = lock(&store).data.packages.iter().find(|p| p.id == id).cloned();
            if let Some(mut pkg) = package {
                pkg.delivered_at = Some(Utc::now());
                if persist(&ui, &store, &[pkg.clone()]) {
                    tracing::info!("Paquete entregado: {} -> {}", pkg.carrier, pkg.destination);
                    ui.set_status_text(
                        format!("Paquete de {} entregado a {} ({}).", pkg.carrier, pkg.destination, pkg.recipient).into(),
                    );
                }
            }
        }
    });

    // Callbacks Módulo Residentes: Abrir/Cancelar Drawer, Sub-panel de Inmuebles, Buscar y Guardar
    let selected_resident_units_model = Rc::new(VecModel::<slint::SharedString>::default());
    ui.set_selected_resident_units(selected_resident_units_model.clone().into());

    ui.on_open_resident_drawer({
        let ui_handle = ui.as_weak();
        let sel_units = selected_resident_units_model.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                sel_units.set_vec(Vec::new());
                ui.set_form_resident_doc("".into());
                ui.set_form_resident_name("".into());
                ui.set_form_resident_unit("".into());
                ui.set_form_resident_role("Propietario".into());
                ui.set_form_resident_mobile("".into());
                ui.set_form_resident_email("".into());
                ui.set_show_unit_picker(false);
                ui.set_show_resident_drawer(true);
            }
        }
    });

    ui.on_cancel_resident_drawer({
        let ui_handle = ui.as_weak();
        let sel_units = selected_resident_units_model.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                sel_units.set_vec(Vec::new());
                ui.set_show_unit_picker(false);
                ui.set_show_resident_drawer(false);
            }
        }
    });

    ui.on_open_unit_picker({
        let ui_handle = ui.as_weak();
        let units_vec = units_model.clone();
        let sel_units = selected_resident_units_model.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_unit_search_query("".into());
                filter_and_update_unit_picker(&ui, &units_vec, &sel_units, "");
                ui.set_show_unit_picker(true);
            }
        }
    });

    ui.on_close_unit_picker({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_unit_picker(false);
            }
        }
    });

    ui.on_search_units({
        let ui_handle = ui.as_weak();
        let units_vec = units_model.clone();
        let sel_units = selected_resident_units_model.clone();
        move |query| {
            if let Some(ui) = ui_handle.upgrade() {
                filter_and_update_unit_picker(&ui, &units_vec, &sel_units, query.as_str());
            }
        }
    });

    ui.on_pick_unit({
        let ui_handle = ui.as_weak();
        let sel_units = selected_resident_units_model.clone();
        move |val| {
            let already = (0..sel_units.row_count()).any(|i| sel_units.row_data(i) == Some(val.clone()));
            if !already {
                sel_units.push(val.clone());
            }
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_unit_picker(false);
                ui.set_status_text(format!("Inmueble {} asociado al residente.", val).into());
            }
        }
    });

    ui.on_remove_resident_unit({
        let ui_handle = ui.as_weak();
        let sel_units = selected_resident_units_model.clone();
        move |idx| {
            if idx >= 0 && (idx as usize) < sel_units.row_count() {
                sel_units.remove(idx as usize);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Inmueble removido de la asociación.".into());
                }
            }
        }
    });

    ui.on_search_residents({
        let store = store.clone();
        let model = residents_model.clone();
        move |query| {
            model.set_vec(ui_bind::filtered_residents(&lock(&store), query.as_str()));
        }
    });

    ui.on_save_resident({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        let sel_units = selected_resident_units_model.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let name = ui.get_form_resident_name();
            let assigned_units: Vec<String> = sel_units.iter().map(|u| u.to_string()).collect();

            if name.trim().is_empty() || assigned_units.is_empty() {
                show_bottom_alert(&ui, "error", "Ingrese nombre completo y asigne al menos un inmueble.");
                return;
            }

            let resident = ResidentRecord {
                id: uuid::Uuid::new_v4().to_string(), // RN-SYNC-07
                full_name: name.trim().to_string(),
                document: non_empty(&ui.get_form_resident_doc()),
                units: assigned_units,
                role: ui.get_form_resident_role().to_string(),
                mobile: non_empty(&ui.get_form_resident_mobile()),
                email: non_empty(&ui.get_form_resident_email()),
            };
            let unit_summary = resident.units.join(", ");

            ui.set_resident_search_query("".into());
            if persist(&ui, &store, &[resident.clone()]) {
                sel_units.set_vec(Vec::new());
                tracing::info!("Residente registrado: {} en {}", resident.full_name, unit_summary);
                ui.set_status_text(format!("Residente {} registrado en {}.", resident.full_name, unit_summary).into());
                ui.set_show_unit_picker(false);
                ui.set_show_resident_drawer(false);
            }
        }
    });

    // Callback: Abrir Side Drawer de Nuevo Inmueble
    ui.on_open_drawer({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                if ui.get_selected_tower().is_empty() {
                    show_bottom_alert(&ui, "error", "Cree primero una torre o bloque.");
                    return;
                }
                model.set_vec(Vec::new());
                ui.set_editing_unit_id("".into());
                ui.set_editing_unit_tower(ui.get_selected_tower());
                ui.set_form_unit_number("".into());
                ui.set_form_unit_type("Apartamento".into());
                ui.set_form_intercom("".into());
                ui.set_form_mobile("".into());
                ui.set_form_landline("".into());
                ui.set_confirm_delete_unit(false);
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(true);
            }
        }
    });

    // Callback: Ficha del Inmueble al hacer clic en una fila de la tabla (D-34, REQ-UN-12)
    ui.on_open_unit_detail({
        let ui_handle = ui.as_weak();
        let store = store.clone();
        move |id| {
            let Some(ui) = ui_handle.upgrade() else { return };
            if ui_bind::fill_unit_detail(&ui, &lock(&store), &id) {
                ui.set_confirm_delete_unit(false);
                ui.set_show_unit_detail(true);
                let unit = ui.get_detail_unit();
                ui.set_status_text(format!("Ficha de {} - {}", unit.tower, unit.unit_number).into());
            }
        }
    });

    // Callback: Abrir el panel en modo edición con los datos del inmueble (REQ-UN-03)
    ui.on_edit_unit({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        let store = store.clone();
        move |id| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let s = lock(&store);
            let Some(unit) = s.data.units.iter().find(|u| u.is_active && u.id == id.as_str()) else {
                return;
            };
            model.set_vec(ui_bind::unit_parking_display(&s, unit));
            ui.set_editing_unit_id(unit.id.as_str().into());
            ui.set_editing_unit_tower(unit.tower.as_str().into());
            ui.set_form_unit_number(unit.unit_number.as_str().into());
            ui.set_form_unit_type(unit.unit_type.as_str().into());
            ui.set_form_intercom(unit.intercom.clone().unwrap_or_default().into());
            ui.set_form_mobile(unit.mobile.clone().unwrap_or_default().into());
            ui.set_form_landline(unit.landline.clone().unwrap_or_default().into());
            ui.set_confirm_delete_unit(false);
            ui.set_show_parking_picker(false);
            ui.set_show_drawer(true);
        }
    });

    // Callback: Eliminar (inactivar) el inmueble en edición (RN-UN-07..09)
    ui.on_delete_unit({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let id = ui.get_editing_unit_id();
            let result = units_crud::delete_unit(&lock(&store).data, &id, &ui.get_current_user_role());
            match result {
                Err(e) => {
                    ui.set_confirm_delete_unit(false);
                    show_bottom_alert(&ui, "error", &e.to_string());
                }
                Ok(plan) => {
                    let label = plan.units.first().map(|u| u.label()).unwrap_or_default();
                    if persist_plan(&ui, &store, &plan) {
                        tracing::info!("Inmueble eliminado (inactivado): {}", label);
                        model.set_vec(Vec::new());
                        ui.set_status_text(format!("Inmueble {} eliminado.", label).into());
                        ui.set_show_parking_picker(false);
                        ui.set_show_drawer(false);
                    }
                }
            }
        }
    });

    // Callback: Cancelar / Cerrar Side Drawer
    ui.on_cancel_drawer({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                model.set_vec(Vec::new());
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(false);
            }
        }
    });

    // Callback: Abrir sub-panel inferior con lista de parqueaderos disponibles y buscador
    ui.on_open_parking_picker({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        let store = store.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_parking_search_query("".into());
                let available = ui_bind::available_parking(&lock(&store), &ui.get_editing_unit_id());
                filter_and_update_picker(&ui, &available, &model, "");
                ui.set_show_parking_picker(true);
                ui.set_status_text("Seleccione un parqueadero del panel inferior o use el buscador.".into());
            }
        }
    });

    // Callback: Cerrar sub-panel inferior
    ui.on_close_parking_picker({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_parking_picker(false);
            }
        }
    });

    // Callback: Filtrado en tiempo real desde el buscador
    ui.on_search_parking({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        let store = store.clone();
        move |query| {
            if let Some(ui) = ui_handle.upgrade() {
                let available = ui_bind::available_parking(&lock(&store), &ui.get_editing_unit_id());
                filter_and_update_picker(&ui, &available, &model, query.as_str());
            }
        }
    });

    // Callback: Asignar parqueadero seleccionado desde la lista y cerrar el panel de abajo
    ui.on_pick_parking({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        move |val| {
            let already_added = (0..model.row_count()).any(|i| model.row_data(i) == Some(val.clone()));
            if !already_added {
                tracing::info!("Parqueadero asignado desde panel inferior: {}", val);
                model.push(val.clone());
            }
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_parking_picker(false);
                ui.set_status_text(format!("Parqueadero {} asignado al inmueble.", val).into());
            }
        }
    });

    // Callback: Eliminar una fila de parqueadero agregada
    ui.on_remove_parking_selection({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        move |idx| {
            if idx >= 0 && (idx as usize) < model.row_count() {
                model.remove(idx as usize);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Parqueadero removido de la asignación.".into());
                }
            }
        }
    });

    // Callback: Crear un nuevo parqueadero en la copropiedad y asignarlo directamente
    ui.on_create_new_parking({
        let ui_handle = ui.as_weak();
        let sel_model = selected_parking_model.clone();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let code = ui.get_form_parking_code().trim().to_string();
            if code.is_empty() {
                show_bottom_alert(&ui, "error", "Ingrese un código o identificador para el parqueadero.");
                return;
            }
            let exists = lock(&store).data.parking_spots.iter().any(|p| p.code.eq_ignore_ascii_case(&code));
            if exists {
                show_bottom_alert(&ui, "error", &format!("El parqueadero {} ya existe en la copropiedad.", code));
                return;
            }

            let spot = ParkingSpotRecord {
                code: code.clone(),
                spot_type: ui.get_form_parking_type().to_string(),
                assigned_unit_id: None,
            };
            if !persist(&ui, &store, &[spot.clone()]) {
                return;
            }

            // Si se abrió desde el panel de Nuevo Inmueble, auto-asignar al predio actual
            if ui.get_show_drawer() {
                sel_model.push(spot.display_name().into());
                ui.set_status_text(format!("Parqueadero {} creado y asignado al inmueble.", code).into());
            } else {
                ui.set_status_text(format!("Parqueadero {} creado en el catálogo disponible.", code).into());
            }
            ui.set_form_parking_code("".into());
            ui.set_show_parking_modal(false);
        }
    });

    // Callback: Guardar inmueble (crear o editar según `editing-unit-id`)
    ui.on_save_unit({
        let ui_handle = ui.as_weak();
        let model = selected_parking_model.clone();
        let store = store.clone();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            let form = unit_form(&ui, &model);
            let editing_id = ui.get_editing_unit_id();
            let result = {
                let s = lock(&store);
                if editing_id.is_empty() {
                    units_crud::create_unit(&s.data, &ui.get_editing_unit_tower(), &form)
                } else {
                    units_crud::update_unit(&s.data, &editing_id, &form)
                }
            };
            let plan = match result {
                Ok(plan) => plan,
                Err(e) => return show_bottom_alert(&ui, "error", &e.to_string()),
            };
            let Some(unit) = plan.units.first().cloned() else { return };
            let parking_summary = if unit.parking.is_empty() { "Sin celda".to_string() } else { unit.parking.join(", ") };
            if persist_plan(&ui, &store, &plan) {
                let verb = if editing_id.is_empty() { "registrado" } else { "actualizado" };
                tracing::info!("Inmueble {}: {} | Parqueaderos: {}", verb, unit.label(), parking_summary);
                model.set_vec(Vec::new());
                ui.set_status_text(format!("Inmueble {} {} con [{}].", unit.label(), verb, parking_summary).into());
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(false);
            }
        }
    });

    // Sincronización en segundo plano en las ventanas 03/10/15/22 h (D-23)
    let sync_handle = sync::spawn_scheduler(store.clone(), server, ui.as_weak());

    // Sincronización manual: botón SINCRONIZAR o tecla F9 (D-28)
    ui.on_sync_now({
        let ui_handle = ui.as_weak();
        move || {
            let Some(ui) = ui_handle.upgrade() else { return };
            if sync_handle.request_now() {
                tracing::info!("Sincronización manual solicitada por el vigilante");
                ui.set_status_text("Sincronización manual iniciada.".into());
            } else {
                ui.set_status_text("Ya hay una sincronización en curso.".into());
            }
        }
    });

    // Ajustar la ventana al tamaño del monitor (maximizada, no pantalla completa).
    // En Windows, Slint (backend winit) no aplica `set_maximized(true)` hecho antes
    // de que exista la ventana nativa, y luego ignora nuevas llamadas porque cree
    // que ya está maximizada. Por eso se maximiza con el bucle de eventos ya en
    // marcha y se alterna false→true para forzar que la orden llegue a Windows.
    ui.show()?;
    {
        let ui_weak = ui.as_weak();
        slint::Timer::single_shot(Duration::from_millis(50), move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.window().set_maximized(false);
                ui.window().set_maximized(true);
            }
        });
    }

    // Iniciar bucle de eventos nativo de Windows (GPU / Win32)
    ui.run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parking_code;

    #[test]
    fn extrae_codigo_de_parqueadero() {
        assert_eq!(parking_code("P-104 (Carro)"), "P-104");
        assert_eq!(parking_code("M-01"), "M-01");
    }
}
