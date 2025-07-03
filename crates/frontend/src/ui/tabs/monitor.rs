use crate::app::Myapp;
use eframe::egui;

pub fn render(app: &mut Myapp, ui: &mut egui::Ui) {
    app.show_log_window = true;
    if let Some(accounce) = app.announce3.clone() {
        ui.label(accounce);
    } else {
        ui.label("暂无监视公告");
    }

    ui.separator();
}
