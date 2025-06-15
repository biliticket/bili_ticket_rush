use crate::app::Myapp;
use eframe::egui;

pub fn render(app: &mut Myapp, ui: &mut egui::Ui) {
    ui.heading("预留帮助公告栏");
    ui.separator();
    ui.label("本项目地址：https://github.com/biliticket/bili_ticket_rush");
}
