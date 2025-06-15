use crate::app::Myapp;
use eframe::egui;

// 定义横幅类型枚举
#[derive(PartialEq)]
pub enum BannerType {
    Error,
    Success,
}

// 重命名函数为更通用的名称
pub fn render_notification_banner(app: &Myapp, ctx: &egui::Context) {
    let screen_rect = ctx.available_rect();
    let banner_height = 40.0;

    // 创建一个位于屏幕顶部的区域
    let banner_rect = egui::Rect::from_min_size(
        egui::pos2(screen_rect.min.x, screen_rect.min.y),
        egui::vec2(screen_rect.width(), banner_height),
    );

    // 使用Area绝对定位横幅
    egui::Area::new("notification_banner")
        .fixed_pos(banner_rect.min)
        .show(ctx, |ui| {
            // 根据当前激活的横幅类型选择颜色
            let (fill_color, stroke_color) = if app.success_banner_active {
                // 成功横幅 - 浅绿色
                (
                    egui::Color32::from_rgba_premultiplied(
                        130,
                        220,
                        130,
                        (app.success_banner_opacity * 255.0) as u8,
                    ),
                    egui::Color32::from_rgba_premultiplied(
                        100,
                        200,
                        100,
                        (app.success_banner_opacity * 255.0) as u8,
                    ),
                )
            } else {
                // 错误横幅 - 橙红色
                (
                    egui::Color32::from_rgba_premultiplied(
                        245,
                        130,
                        90,
                        (app.error_banner_opacity * 255.0) as u8,
                    ),
                    egui::Color32::from_rgba_premultiplied(
                        225,
                        110,
                        70,
                        (app.error_banner_opacity * 255.0) as u8,
                    ),
                )
            };

            // 设置框架样式
            let frame = egui::Frame::none()
                .fill(fill_color)
                .stroke(egui::Stroke::new(1.0, stroke_color));

            frame.show(ui, |ui| {
                ui.set_max_width(screen_rect.width());

                // 居中白色文本
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    let banner_text = if app.success_banner_active {
                        &app.success_banner_text
                    } else {
                        &app.error_banner_text
                    };
                    let text = egui::RichText::new(banner_text)
                        .color(egui::Color32::WHITE)
                        .size(16.0)
                        .strong();
                    ui.label(text);
                    ui.add_space(5.0);
                });
            });
        });
}

// 为了向后兼容，保留原函数名，但内部调用新函数
pub fn render_error_banner(app: &Myapp, ctx: &egui::Context) {
    render_notification_banner(app, ctx);
}
