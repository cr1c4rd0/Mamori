slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando Mamori Terminal Windows Nativa (Slint)...");

    let ui = AppWindow::new()?;

    // Callbacks nativos para la interfaz de garita
    ui.on_quick_action({
        let ui_handle = ui.as_weak();
        move |action| {
            if let Some(ui) = ui_handle.upgrade() {
                tracing::info!("Acción rápida disparada: {}", action);
                ui.set_active_module(action.clone());
                ui.set_status_text(format!("Módulo cargado: {}", action).into());
            }
        }
    });

    // Iniciar el bucle de eventos nativo de Windows
    ui.run()?;

    Ok(())
}
