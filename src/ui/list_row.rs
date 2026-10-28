use egui::{Align, Color32, Layout, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

pub struct RowAction {
    pub selected: bool,
    pub activate: bool,
}

pub fn selectable_row(
    ui: &mut Ui,
    row_id: &str,
    selected_id: &Option<String>,
    height: f32,
    on_click: impl FnOnce(&mut Ui) -> bool,
) -> RowAction {
    let is_selected = selected_id.as_deref() == Some(row_id);
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, is_selected);
        let rounding = visuals.corner_radius;
        ui.painter().rect_filled(rect, rounding, visuals.bg_fill);
        if is_selected || response.hovered() {
            ui.painter().rect_stroke(
                rect,
                rounding,
                Stroke::new(1.0_f32, visuals.bg_stroke.color),
                StrokeKind::Inside,
            );
        }
    }

    let mut activate = false;
    let inner = rect.shrink(10.0);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner), |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            activate = on_click(ui);
        });
    });

    let selected = if response.clicked() {
        true
    } else {
        false
    };

    RowAction {
        selected,
        activate,
    }
}

pub fn meta_label(ui: &mut Ui, text: &str, accent: bool) -> Response {
    let color = if accent {
        Color32::from_rgb(100, 190, 130)
    } else {
        ui.visuals().weak_text_color()
    };
    ui.label(egui::RichText::new(text).small().color(color))
}
