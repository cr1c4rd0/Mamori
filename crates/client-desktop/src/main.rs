slint::include_modules!();

#[allow(unused_imports)]
use mamori_shared::models::*;
use slint::{Model, VecModel};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

/// Filtra en tiempo real los parqueaderos del catálogo maestro:
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
        .filter(|spot| {
            if query_lower.is_empty() {
                true
            } else {
                spot.to_lowercase().contains(&query_lower)
            }
        })
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Mamori Terminal Windows Nativa (Slint)...");

    let ui = AppWindow::new()?;

    // Modelo reactivo para la tabla de inmuebles en la vista principal
    let initial_units = vec![
        UnitItem {
            id: "u1".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 101".into(),
            unit_type: "Apartamento".into(),
            intercom: "Ext. 101".into(),
            mobile: "300 123 4567".into(),
            landline: "(604) 444 0001".into(),
            parking: "P-101".into(),
        },
        UnitItem {
            id: "u2".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 102".into(),
            unit_type: "Apartamento".into(),
            intercom: "Sin citófono".into(),
            mobile: "315 987 6543".into(),
            landline: "—".into(),
            parking: "P-102, P-102B".into(),
        },
        UnitItem {
            id: "u3".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 103".into(),
            unit_type: "Apartamento".into(),
            intercom: "Ext. 103".into(),
            mobile: "311 555 8899".into(),
            landline: "—".into(),
            parking: "Sin celda".into(),
        },
        UnitItem {
            id: "u4".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 201".into(),
            unit_type: "Apartamento".into(),
            intercom: "Ext. 201".into(),
            mobile: "310 444 3322".into(),
            landline: "(604) 444 0004".into(),
            parking: "P-201".into(),
        },
        UnitItem {
            id: "u5".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 202".into(),
            unit_type: "Apartamento".into(),
            intercom: "Ext. 202".into(),
            mobile: "318 777 6655".into(),
            landline: "—".into(),
            parking: "P-202".into(),
        },
        UnitItem {
            id: "u6".into(),
            tower: "Torre 1".into(),
            unit_number: "Apto 203".into(),
            unit_type: "Apartamento".into(),
            intercom: "Sin citófono".into(),
            mobile: "301 222 1100".into(),
            landline: "—".into(),
            parking: "P-203".into(),
        },
    ];
    let units_model = Rc::new(VecModel::from(initial_units));
    ui.set_units(units_model.clone().into());

    // Catálogo maestro de parqueaderos disponibles en la copropiedad (no asignados a otros predios)
    let master_parking_catalog = Rc::new(RefCell::new(vec![
        "P-104 (Carro)".to_string(),
        "P-105 (Carro)".to_string(),
        "P-106 (Carro)".to_string(),
        "P-107 (Carro)".to_string(),
        "M-01 (Moto)".to_string(),
        "M-02 (Moto)".to_string(),
        "M-03 (Moto)".to_string(),
        "M-04 (Moto)".to_string(),
    ]));

    // Modelo reactivo para las filas dinámicas de parqueaderos asignados al inmueble en edición
    let selected_parking_model = Rc::new(VecModel::default());
    ui.set_selected_parking_list(selected_parking_model.clone().into());

    // Callbacks de navegación entre módulos
    ui.on_quick_action({
        let ui_handle = ui.as_weak();
        move |action| {
            if let Some(ui) = ui_handle.upgrade() {
                tracing::info!("Módulo seleccionado: {}", action);
                ui.set_active_module(action.clone());
                ui.set_status_text(format!("Módulo activo: {}", action).into());
            }
        }
    });

    // Callback para seleccionar torre
    ui.on_select_tower({
        let ui_handle = ui.as_weak();
        move |tower_name| {
            if let Some(ui) = ui_handle.upgrade() {
                tracing::info!("Torre seleccionada: {}", tower_name);
                ui.set_status_text(format!("Visualizando predios de: {}", tower_name).into());
            }
        }
    });

    // Callback: Abrir Side Drawer de Nuevo Inmueble
    let model_open = selected_parking_model.clone();
    ui.on_open_drawer({
        let ui_handle = ui.as_weak();
        let model = model_open.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                model.set_vec(Vec::new());
                ui.set_form_unit_number("".into());
                ui.set_form_intercom("".into());
                ui.set_form_mobile("".into());
                ui.set_form_landline("".into());
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(true);
            }
        }
    });

    // Callback: Cancelar / Cerrar Side Drawer
    let model_cancel = selected_parking_model.clone();
    ui.on_cancel_drawer({
        let ui_handle = ui.as_weak();
        let model = model_cancel.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                model.set_vec(Vec::new());
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(false);
            }
        }
    });

    // Callback: Abrir sub-panel inferior con lista de parqueaderos disponibles y buscador
    let model_picker = selected_parking_model.clone();
    let master_picker = master_parking_catalog.clone();
    ui.on_open_parking_picker({
        let ui_handle = ui.as_weak();
        let model = model_picker.clone();
        let master = master_picker.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_parking_search_query("".into());
                filter_and_update_picker(&ui, &master.borrow(), &model, "");
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
    let model_search = selected_parking_model.clone();
    let master_search = master_parking_catalog.clone();
    ui.on_search_parking({
        let ui_handle = ui.as_weak();
        let model = model_search.clone();
        let master = master_search.clone();
        move |query| {
            if let Some(ui) = ui_handle.upgrade() {
                filter_and_update_picker(&ui, &master.borrow(), &model, query.as_str());
            }
        }
    });

    // Callback: Asignar parqueadero seleccionado desde la lista y cerrar el panel de abajo
    let model_pick = selected_parking_model.clone();
    ui.on_pick_parking({
        let ui_handle = ui.as_weak();
        let model = model_pick.clone();
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
    let selected_model_remove = selected_parking_model.clone();
    ui.on_remove_parking_selection({
        let ui_handle = ui.as_weak();
        let model = selected_model_remove.clone();
        move |idx| {
            if idx >= 0 && (idx as usize) < model.row_count() {
                let mut current: Vec<slint::SharedString> = Vec::new();
                for i in 0..model.row_count() {
                    if i != (idx as usize) {
                        if let Some(item) = model.row_data(i) {
                            current.push(item);
                        }
                    }
                }
                model.set_vec(current);
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Parqueadero removido de la asignación.".into());
                }
            }
        }
    });

    // Callback: Crear un nuevo parqueadero en la copropiedad y asignarlo directamente
    let selected_model_create = selected_parking_model.clone();
    let master_catalog_create = master_parking_catalog.clone();
    ui.on_create_new_parking({
        let ui_handle = ui.as_weak();
        let sel_model = selected_model_create.clone();
        let master = master_catalog_create.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let code = ui.get_form_parking_code();
                let spot_type = ui.get_form_parking_type();
                if !code.trim().is_empty() {
                    let formatted = format!("{} ({})", code.trim(), spot_type);
                    let shared_str = slint::SharedString::from(formatted.clone());

                    // Agregar al catálogo maestro si no existía
                    if !master.borrow().contains(&formatted) {
                        master.borrow_mut().push(formatted.clone());
                    }

                    // Auto-asignar a la lista del inmueble
                    let already_added = (0..sel_model.row_count()).any(|i| sel_model.row_data(i) == Some(shared_str.clone()));
                    if !already_added {
                        sel_model.push(shared_str);
                    }

                    ui.set_status_text(format!("Parqueadero {} creado y asignado al inmueble.", code).into());
                    ui.set_form_parking_code("".into());
                    ui.set_show_parking_modal(false);
                }
            }
        }
    });

    // Callback: Guardar nuevo inmueble desde la terminal
    let selected_model_save = selected_parking_model.clone();
    let units_model_save = units_model.clone();
    let master_catalog_save = master_parking_catalog.clone();
    ui.on_save_unit({
        let ui_handle = ui.as_weak();
        let model = selected_model_save.clone();
        let units_vec = units_model_save.clone();
        let master = master_catalog_save.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let unit_number = ui.get_form_unit_number();
                let mobile = ui.get_form_mobile();
                let intercom = ui.get_form_intercom();
                let landline = ui.get_form_landline();
                let selected_tower = ui.get_selected_tower();

                if unit_number.trim().is_empty() {
                    ui.set_status_text("Error: Ingrese un número de inmueble antes de guardar.".into());
                    return;
                }

                // Recopilar todos los parqueaderos seleccionados en las filas dinámicas
                let mut parkings = Vec::new();
                for i in 0..model.row_count() {
                    if let Some(item) = model.row_data(i) {
                        parkings.push(item.to_string());
                    }
                }
                let parking_summary = if parkings.is_empty() {
                    "Sin celda".to_string()
                } else {
                    parkings.join(", ")
                };

                tracing::info!(
                    "Guardando Inmueble: {} en {} | Cel: {} | Citófono: {} | Parqueaderos: {}",
                    unit_number,
                    selected_tower,
                    mobile,
                    intercom,
                    parking_summary
                );

                // 1. Agregar nuevo inmueble al modelo reactivo de la tabla principal
                let new_unit = UnitItem {
                    id: format!("u-{}", units_vec.row_count() + 1).into(),
                    tower: selected_tower,
                    unit_number: unit_number.clone(),
                    unit_type: "Apartamento".into(),
                    intercom: if intercom.trim().is_empty() { "Sin citófono".into() } else { intercom },
                    mobile: if mobile.trim().is_empty() { "—".into() } else { mobile },
                    landline: if landline.trim().is_empty() { "—".into() } else { landline },
                    parking: parking_summary.clone().into(),
                };
                units_vec.push(new_unit);

                // 2. Retirar de forma PERMANENTE los parqueaderos asignados del catálogo maestro
                {
                    let mut master_guard = master.borrow_mut();
                    master_guard.retain(|spot| !parkings.contains(spot));
                }

                // 3. Limpiar formulario y cerrar panel
                model.set_vec(Vec::new());

                ui.set_status_text(format!("Inmueble {} guardado exitosamente con [{}].", unit_number, parking_summary).into());
                ui.set_form_unit_number("".into());
                ui.set_form_intercom("".into());
                ui.set_form_mobile("".into());
                ui.set_form_landline("".into());
                ui.set_show_parking_picker(false);
                ui.set_show_drawer(false);
            }
        }
    });

    // Iniciar bucle de eventos nativo de Windows (GPU / Win32)
    ui.run()?;

    Ok(())
}
