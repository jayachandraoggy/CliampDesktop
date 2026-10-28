use egui::Ui;

use crate::ipc::types::ProviderPlaylist;

pub fn show_radio_browser(
    ui: &mut Ui,
    search_query: &mut String,
    playlists: &[ProviderPlaylist],
    loading: bool,
    reload_home: &mut bool,
    run_search: &mut bool,
    play_id: &mut Option<String>,
    open_catalog: &mut Option<String>,
) {
    ui.heading("Radio Browser");
    ui.horizontal(|ui| {
        ui.label("Search");
        ui.text_edit_singleline(search_query);
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

    egui::ScrollArea::vertical().show(ui, |ui| {
        for row in playlists {
            let name = row.display_name();
            let section = row.section.as_deref().unwrap_or("");
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&name).strong());
                    if !section.is_empty() {
                        ui.label(egui::RichText::new(section).small().weak());
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(id) = &row.id {
                        let is_browse = row.extra.get("browse").and_then(|v| v.as_bool()) == Some(true)
                            || name.starts_with("Browse ")
                            || section == "Countries"
                            || section == "Genres & Tags";
                        if is_browse {
                            if ui.button("Open").clicked() {
                                *open_catalog = Some(id.clone());
                            }
                        } else if ui.button("Play").clicked() {
                            *play_id = Some(id.clone());
                        }
                    }
                });
            });
            ui.separator();
        }
        if playlists.is_empty() && !loading {
            ui.label("Load the radio home or search for stations.");
        }
    });
}
