use eframe::egui;
use crate::app::Myapp;

pub fn render(app: &mut Myapp, ui: &mut egui::Ui) {
    
    ui.heading("预留帮助公告栏");
    ui.separator();
    ui.label("本项目地址：https://github.com/biliticket/bili_ticket_rush");
}