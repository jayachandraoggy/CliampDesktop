use egui::{RichText, Ui};

use crate::cliamp::spawn::ConnectionPhase;

pub fn connection_banner(ui: &mut Ui, phase: &ConnectionPhase) {
    match phase {
        ConnectionPhase::NoBinary => {
            ui.colored_label(
                egui::Color32::from_rgb(220, 120, 80),
                "cliamp is not installed. Install from https://www.cliamp.stream/",
            );
            ui.hyperlink_to("cliamp.stream", "https://www.cliamp.stream/");
        }
        ConnectionPhase::Connecting => {
            ui.spinner();
            ui.label("Connecting to cliamp…");
        }
        ConnectionPhase::Failed(msg) => {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("Connection failed: {}", msg));
        }
        ConnectionPhase::Connected => {}
    }
}

pub fn error_toast(ui: &mut Ui, message: &Option<String>) {
    if let Some(msg) = message {
        ui.colored_label(
            egui::Color32::from_rgb(255, 140, 140),
            RichText::new(msg).strong(),
        );
    }
}
