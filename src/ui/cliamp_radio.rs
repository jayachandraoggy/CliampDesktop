use egui::{RichText, Ui};

use crate::radio_api::channels::Channel;
use crate::radio_api::statistics::Statistics;
use crate::ui::list_row::{meta_label, selectable_row};

pub fn show_cliamp_radio(
    ui: &mut Ui,
    channels: &[Channel],
    stats: &Option<Statistics>,
    loading: bool,
    selected_id: &mut Option<String>,
    refresh: &mut bool,
    play_channel: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        ui.heading("Cliamp Radio");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if loading {
                ui.spinner();
            }
            if ui.button("Refresh").clicked() {
                *refresh = true;
            }
        });
    });
    ui.add_space(4.0);
    ui.label(
        RichText::new("Select a channel, then press Play — or use ▶ on the row.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for ch in channels {
                let row_key = format!("cliamp:{}", ch.id);
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
                            "Live".to_string()
                        }
                    });

                let ch_id = ch.id.clone();
                let ch_name = ch.name.clone();
                let genre = ch.genre.clone();
                let description = ch.description.clone();
                let listeners_text = listeners.clone();
                let is_live = ch.tracks == 0;

                let action = selectable_row(ui, &row_key, selected_id, 76.0, |ui| {
                    let mut activate = false;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 52.0);
                            ui.label(RichText::new(&ch_name).strong());
                            if let Some(g) = &genre {
                                ui.label(RichText::new(g).small());
                            }
                            if let Some(d) = &description {
                                ui.label(RichText::new(d).small().weak());
                            }
                            let _ = meta_label(ui, &listeners_text, is_live);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("▶").size(16.0))
                                        .min_size(egui::vec2(36.0, 36.0)),
                                )
                                .clicked()
                            {
                                activate = true;
                            }
                        });
                    });
                    activate
                });

                if action.selected {
                    *selected_id = Some(row_key);
                }
                if action.activate {
                    *selected_id = Some(format!("cliamp:{}", ch_id));
                    *play_channel = Some(ch_id);
                }
                ui.add_space(4.0);
            }
            if channels.is_empty() && !loading {
                ui.label("No channels loaded.");
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can_play = selected_id
            .as_ref()
            .and_then(|k| k.strip_prefix("cliamp:"))
            .is_some();
        ui.add_enabled_ui(can_play, |ui| {
            if ui.button("Play selected").clicked() {
                if let Some(id) = selected_id
                    .as_ref()
                    .and_then(|k| k.strip_prefix("cliamp:"))
                    .map(str::to_string)
                {
                    *play_channel = Some(id);
                }
            }
        });
    });
}
