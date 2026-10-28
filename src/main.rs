mod app;
mod cliamp;
mod ipc;
mod radio_api;
mod ui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("Cliamp Desktop"),
        ..Default::default()
    };

    eframe::run_native(
        "Cliamp Desktop",
        options,
        Box::new(|cc| Ok(Box::new(app::CliampApp::new(cc)))),
    )
}
