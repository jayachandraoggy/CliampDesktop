use egui::{RichText, Ui};

use crate::ipc::types::ProviderPlaylist;
use crate::ui::list_row::{meta_label, selectable_row};

fn row_key(row: &ProviderPlaylist, index: usize) -> String {
    row.id
        .clone()
        .unwrap_or_else(|| format!("browser:row:{index}"))
}

fn is_browse_row(row: &ProviderPlaylist) -> bool {
    if let Some(id) = &row.id {
        if id.starts_with("browse:") || id == "loc:ask" {
            return true;
        }
    }
    let name = row.display_name();
    let section = row.section.as_deref().unwrap_or("");
    row.extra.get("browse").and_then(|v| v.as_bool()) == Some(true)
        || name.starts_with("Browse ")
        || section == "Countries"
        || section == "Genres & Tags"
}

fn can_load_channels(id: &str) -> bool {
    !id.starts_with("browse:") && id != "loc:ask"
}

pub fn channel_stream_path(row: &ProviderPlaylist) -> Option<String> {
    row.extra
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

pub fn show_radio_browser(
    ui: &mut Ui,
    search_query: &mut String,
    playlists: &[ProviderPlaylist],
    loading: bool,
    channel_tracks: &[ProviderPlaylist],
    channel_source_name: &str,
    channel_total: u32,
    channels_loading: bool,
    selected_list_id: &mut Option<String>,
    selected_channel_id: &mut Option<String>,
    reload_home: &mut bool,
    run_search: &mut bool,
    load_channels: &mut Option<String>,
    open_catalog: &mut Option<String>,
    load_more_channels: &mut bool,
    play_stream_url: &mut Option<String>,
) {
    ui.heading("Radio Browser");
    ui.horizontal(|ui| {
        ui.label("Search");
        let search = ui.text_edit_singleline(search_query);
        if search.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            *run_search = true;
        }
        if ui.button("Search").clicked() {
            *run_search = true;
        }
        if ui.button("Browse home").clicked() {
            *reload_home = true;
        }
        if loading {
            ui.spinner();
        }
    });
    ui.add_space(4.0);
    ui.label(
        RichText::new("Pick a station or playlist above, then choose a channel below to play.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);

    ui.label(RichText::new("Browse").strong());
    egui::ScrollArea::vertical()
        .id_salt("radio_browser_main")
        .max_height(240.0)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (index, row) in playlists.iter().enumerate() {
                let key = row_key(row, index);
                let name = row.display_name();
                let section = row.section.as_deref().unwrap_or("").to_string();
                let browse = is_browse_row(row);
                let row_id = row.id.clone();

                let action = selectable_row(ui, &key, selected_list_id, 64.0, |ui| {
                    let mut activate = false;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 52.0);
                            ui.label(RichText::new(&name).strong());
                            if !section.is_empty() {
                                ui.label(RichText::new(&section).small().weak());
                            } else if browse {
                                let _ = meta_label(ui, "Category", false);
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(id) = &row_id {
                                if browse {
                                    if ui.button("Open").clicked() {
                                        activate = true;
                                    }
                                } else if can_load_channels(id) && ui.button("Channels").clicked() {
                                    activate = true;
                                }
                            }
                        });
                    });
                    activate
                });

                if action.selected {
                    *selected_list_id = Some(key.clone());
                    if let Some(id) = &row_id {
                        if can_load_channels(id) && !browse {
                            *load_channels = Some(id.clone());
                        }
                    }
                }
                if action.activate {
                    if let Some(id) = row_id {
                        *selected_list_id = Some(key.clone());
                        if browse {
                            *open_catalog = Some(id);
                        } else if can_load_channels(&id) {
                            *load_channels = Some(id);
                        }
                    }
                }
                ui.add_space(4.0);
            }
            if playlists.is_empty() && !loading {
                ui.label("Load the radio home or search for stations.");
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let selected = playlists
            .iter()
            .enumerate()
            .find(|(i, row)| selected_list_id.as_deref() == Some(row_key(row, *i).as_str()));
        if let Some((_, row)) = selected {
            if is_browse_row(row) {
                if ui.button("Open selected").clicked() {
                    if let Some(id) = &row.id {
                        *open_catalog = Some(id.clone());
                    }
                }
            } else if let Some(id) = &row.id {
                if can_load_channels(id) && ui.button("Load channels").clicked() {
                    *load_channels = Some(id.clone());
                }
            }
        }
    });

    ui.separator();
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Channels").strong());
        if !channel_source_name.is_empty() {
            ui.label(RichText::new(format!("· {channel_source_name}")).weak());
        }
        if channels_loading {
            ui.spinner();
        } else if channel_total > 0 {
            ui.label(
                RichText::new(format!("({}/{})", channel_tracks.len(), channel_total))
                    .small()
                    .weak(),
            );
        }
    });

    egui::ScrollArea::vertical()
        .id_salt("radio_browser_channels")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            if channel_tracks.is_empty() && !channels_loading {
                ui.label(
                    RichText::new("Select a playlist or station above to list playable channels here.")
                        .weak(),
                );
                return;
            }

            for (index, row) in channel_tracks.iter().enumerate() {
                let key = row_key(row, index);
                let name = row.display_name();
                let meta = row.section.as_deref().unwrap_or("");

                let action = selectable_row(ui, &key, selected_channel_id, 56.0, |ui| {
                    let mut activate = false;
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width() - 52.0);
                            ui.label(RichText::new(&name).strong());
                            if !meta.is_empty() {
                                ui.label(RichText::new(meta).small().weak());
                            }
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
                    *selected_channel_id = Some(key.clone());
                }
                if action.activate {
                    *selected_channel_id = Some(key);
                    if let Some(path) = channel_stream_path(row) {
                        *play_stream_url = Some(path);
                    }
                }
                ui.add_space(4.0);
            }
        });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can_play = selected_channel_id.as_ref().and_then(|key| {
            channel_tracks
                .iter()
                .enumerate()
                .find(|(i, row)| row_key(row, *i) == *key)
                .and_then(|(_, row)| channel_stream_path(row))
        });
        ui.add_enabled_ui(can_play.is_some(), |ui| {
            if ui.button("Play selected channel").clicked() {
                *play_stream_url = can_play;
            }
        });
        let can_load_more =
            !channels_loading && channel_total as usize > channel_tracks.len() && !channel_tracks.is_empty();
        ui.add_enabled_ui(can_load_more, |ui| {
            if ui.button("Load more channels").clicked() {
                *load_more_channels = true;
            }
        });
    });
}
