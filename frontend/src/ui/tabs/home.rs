use eframe::egui;
use crate::app::Myapp;

pub fn render(app: &mut Myapp, ui: &mut egui::Ui){
    ui.heading("预留抢票界面公告栏1");
                ui.separator();
                //开始抢票按钮

                ui.vertical_centered(|ui| {
                    // 垂直居中
                    ui.add_space(ui.available_height() * 0.2);
                    
                    // 创建按钮
                    let button = egui::Button::new(
                        egui::RichText::new("开始抢票").size(40.0).color(egui::Color32::WHITE)
                    )
                    .min_size(egui::vec2(300.0, 150.0))
                    .fill(egui::Color32::from_rgb(131, 175, 155))
                    .rounding(20.0);
                    
                    // 只有点击按钮时才触发
                    if ui.add(button).clicked() {
                        app.is_loading = true;
                        app.add_log("开始抢票流程");
                        
                        //待完善鉴权账号及有效信息
                    }
                });

}