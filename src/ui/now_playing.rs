use egui::{Color32, Frame, RichText, Ui, Vec2};

use crate::ipc::types::Snapshot;

fn format_clock(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn state_badge(ui: &mut Ui, state: &str) {
    let (label, color) = match state {
        "playing" => ("Playing", Color32::from_rgb(80, 180, 110)),
        "paused" => ("Paused", ui.visuals().weak_text_color()),
        "buffering" | "loading" => ("Buffering", Color32::from_rgb(220, 170, 70)),
        _ => (state, ui.visuals().weak_text_color()),
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new("●").color(color));
        ui.label(RichText::new(label).small().color(color));
    });
}

pub fn show_now_playing(ui: &mut Ui, snapshot: &Option<Snapshot>) {
    ui.heading("Now playing");
    ui.add_space(4.0);

    Frame::group(ui.style())
        .inner_margin(14.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(180.0);

            let Some(s) = snapshot else {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.label(RichText::new("♪").size(36.0).weak());
                    ui.add_space(8.0);
                    ui.label(RichText::new("Waiting for player…").weak());
                });
                return;
            };

            let track = s.logical_track.as_ref().or(s.track.as_ref());
            ui.horizontal(|ui| {
                let icon_size = Vec2::new(64.0, 64.0);
                Frame::new()
                    .fill(ui.visuals().widgets.inactive.bg_fill)
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        ui.set_min_size(icon_size);
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("♪").size(28.0).weak());
                        });
                    });

                ui.add_space(12.0);
                ui.vertical(|ui| {
                    if let Some(t) = track {
                        let title = t.title.as_deref().unwrap_or("Unknown title");
                        let artist = t.artist.as_deref().unwrap_or("Unknown artist");
                        ui.label(RichText::new(title).size(17.0).strong());
                        ui.label(artist);
                        if let Some(album) = t.album.as_deref().filter(|a| !a.is_empty()) {
                            ui.label(RichText::new(album).small().weak());
                        }
                    } else {
                        ui.label(RichText::new("Nothing loaded").size(17.0).strong());
                        ui.label(RichText::new("Pick a station from the list").weak());
                    }

                    ui.add_space(8.0);
                    if let Some(state) = s.state.as_deref() {
                        state_badge(ui, state);
                    }
                });
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);

            let live = s.seekable == Some(false);
            if live {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(220, 80, 80)));
                    ui.label(RichText::new("Live stream").strong());
                });
                if let Some(pos) = s.position.filter(|p| *p > 0.0) {
                    ui.label(
                        RichText::new(format!("On air for {}", format_clock(pos)))
                            .small()
                            .weak(),
                    );
                }
            } else if let (Some(pos), Some(dur)) = (s.position, s.duration) {
                if dur > 0.0 {
                    let frac = (pos / dur).clamp(0.0, 1.0) as f32;
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .text(format!("{} / {}", format_clock(pos), format_clock(dur))),
                    );
                } else {
                    ui.label(format!("Position {}", format_clock(pos)));
                }
            }

            if let Some(err) = &s.stream_error {
                ui.add_space(6.0);
                ui.colored_label(Color32::LIGHT_RED, err.clone());
            }
        });
}
