use egui::Ui;

use crate::ipc::types::Snapshot;

pub fn show_now_playing(ui: &mut Ui, snapshot: &Option<Snapshot>) {
    ui.heading("Now playing");
    if snapshot.is_none() {
        ui.label("No state yet");
        return;
    }
    let s = snapshot.as_ref().unwrap();
    let track = s.logical_track.as_ref().or(s.track.as_ref());
    if let Some(t) = track {
        let title = t.title.as_deref().unwrap_or("Unknown title");
        let artist = t.artist.as_deref().unwrap_or("Unknown artist");
        ui.label(egui::RichText::new(title).size(18.0).strong());
        ui.label(artist);
    } else {
        ui.label("Nothing loaded");
    }
    let state = s.state.as_deref().unwrap_or("unknown");
    ui.label(format!("State: {}", state));
    if let (Some(pos), Some(dur)) = (s.position, s.duration) {
        if dur > 0.0 {
            ui.add(
                egui::ProgressBar::new((pos / dur).clamp(0.0, 1.0) as f32)
                    .text(format!("{:.0}s / {:.0}s", pos, dur)),
            );
        } else {
            ui.label(format!("Position: {:.0}s", pos));
        }
    } else if s.seekable == Some(false) {
        ui.label("● Live stream");
    }
    if let Some(err) = &s.stream_error {
        ui.colored_label(egui::Color32::LIGHT_RED, err.clone());
    }
}
