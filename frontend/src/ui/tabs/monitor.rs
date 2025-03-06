use eframe::egui;
use crate::app::Myapp;

pub fn render(app: &mut Myapp, ui: &mut egui::Ui){
    app.show_log_window = true;
    ui.heading("预留监视公告栏2");
    ui.separator();

}