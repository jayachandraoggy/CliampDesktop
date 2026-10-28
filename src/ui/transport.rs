use egui::Ui;

use crate::ipc::types::Snapshot;

pub struct TransportActions {
    pub toggle: bool,
    pub next: bool,
    pub prev: bool,
    pub volume: Option<f64>,
}

pub fn show_transport(ui: &mut Ui, snapshot: &Option<Snapshot>) -> TransportActions {
    let mut actions = TransportActions {
        toggle: false,
        next: false,
        prev: false,
        volume: None,
    };

    ui.horizontal(|ui| {
        if ui.button("⏮ Prev").clicked() {
            actions.prev = true;
        }
        let playing = snapshot
            .as_ref()
            .and_then(|s| s.state.as_deref())
            .map(|s| s == "playing")
            .unwrap_or(false);
        let label = if playing { "⏸ Pause" } else { "▶ Play" };
        if ui.button(label).clicked() {
            actions.toggle = true;
        }
        if ui.button("⏭ Next").clicked() {
            actions.next = true;
        }
    });

    let vol = snapshot
        .as_ref()
        .and_then(|s| s.volume)
        .unwrap_or(0.0);
    let mut slider = vol as f32;
    ui.horizontal(|ui| {
        ui.label("Volume (dB)");
        if ui
            .add(egui::Slider::new(&mut slider, -30.0..=12.0).suffix(" dB"))
            .changed()
        {
            actions.volume = Some(slider as f64);
        }
    });

    actions
}
