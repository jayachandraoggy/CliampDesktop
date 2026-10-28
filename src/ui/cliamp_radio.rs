use egui::Ui;

use crate::radio_api::channels::Channel;
use crate::radio_api::statistics::Statistics;

pub fn show_cliamp_radio(
    ui: &mut Ui,
    channels: &[Channel],
    stats: &Option<Statistics>,
    loading: bool,
    refresh: &mut bool,
    play_channel: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        ui.heading("Cliamp Radio");
        if ui.button("Refresh").clicked() {
            *refresh = true;
        }
        if loading {
            ui.spinner();
        }
    });

    egui::ScrollArea::vertical().show(ui, |ui| {
        for ch in channels {
            let listeners = stats
                .as_ref()
                .and_then(|s| s.stations.as_ref())
                .and_then(|m| m.get(&ch.id))
                .and_then(|st| st.listeners)
                .map(|n| format!("● {} listening", n))
                .unwrap_or_else(|| {
                    if ch.tracks > 0 {
                        format!("{} tracks", ch.tracks)
                    } else {
                        "live".to_string()
                    }
                });

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&ch.name).strong());
                        if let Some(g) = &ch.genre {
                            ui.label(g);
                        }
                        if let Some(d) = &ch.description {
                            ui.label(egui::RichText::new(d).small().weak());
                        }
                        ui.label(listeners);
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Play").clicked() {
                            *play_channel = Some(ch.id.clone());
                        }
                    });
                });
            });
            ui.add_space(4.0);
        }
        if channels.is_empty() && !loading {
            ui.label("No channels loaded.");
        }
    });
}
