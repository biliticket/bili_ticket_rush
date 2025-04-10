
use eframe::egui;
use crate::app::Myapp;
pub fn render_error_banner(app: &Myapp, ctx: &egui::Context) {
    let screen_rect = ctx.available_rect();
    let banner_height = 40.0;
    
    // 创建一个位于屏幕顶部的区域
    let banner_rect = egui::Rect::from_min_size(
        egui::pos2(screen_rect.min.x, screen_rect.min.y), 
        egui::vec2(screen_rect.width(), banner_height)
    );
    
    // 使用Area绝对定位横幅
    egui::Area::new("error_banner")
        .fixed_pos(banner_rect.min)
        .show(ctx, |ui| {
            // 设置框架样式
            let frame = egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(
                    245, 130, 90,  (app.error_banner_opacity * 255.0) as u8
                ))
                // rgb(245,130,90)
                .stroke(egui::Stroke::new(
                    1.0, 
                    egui::Color32::from_rgba_premultiplied(
                        225, 110, 70,  (app.error_banner_opacity * 255.0) as u8
                    )
                ));
                // rgb(225,110,70)
            
            frame.show(ui, |ui| {
                ui.set_max_width(screen_rect.width());
                
                // 居中白色文本
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    let text = egui::RichText::new(&app.error_banner_text)
                        .color(egui::Color32::WHITE)
                        .size(16.0)
                        .strong();
                    ui.label(text);
                    ui.add_space(5.0);
                });
            });
        });
}