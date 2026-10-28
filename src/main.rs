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

    let icon = eframe::icon_data::from_png_bytes(include_bytes!(
        "../assets/icons/hicolor/64x64/apps/hertz.png"
    ))
    .expect("valid app icon PNG");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("Hertz")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Hertz",
        options,
        Box::new(|cc| Ok(Box::new(app::CliampApp::new(cc)))),
    )
}
