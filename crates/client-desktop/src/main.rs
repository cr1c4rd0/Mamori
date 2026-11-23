slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Mamori Terminal Windows Nativa (Slint)...");

    let ui = AppWindow::new()?;

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

    // Callback para guardar nuevo inmueble desde la terminal
    ui.on_save_unit({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let unit_number = ui.get_form_unit_number();
                let mobile = ui.get_form_mobile();
                let intercom = ui.get_form_intercom();

                tracing::info!(
                    "Guardando Inmueble: {} | Cel: {} | Citófono: {}",
                    unit_number,
                    mobile,
                    intercom
                );

                ui.set_status_text(format!("Inmueble {} guardado exitosamente.", unit_number).into());
                ui.set_show_form(false);
            }
        }
    });

    // Iniciar bucle de eventos nativo de Windows (GPU / Win32)
    ui.run()?;

    Ok(())
}
