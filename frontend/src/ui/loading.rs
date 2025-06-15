use crate::app::Myapp;
use eframe::egui;

pub fn render_loading_overlay(app: &mut Myapp, ctx: &egui::Context) {
    // 创建覆盖整个界面的区域
    let screen_rect = ctx.screen_rect();
    let layer_id = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("loading_overlay"));
    let ui = ctx.layer_painter(layer_id);

    // 半透明背景
    ui.rect_filled(
        screen_rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
    );

    // 在屏幕中央显示加载动画
    let center = screen_rect.center();

    // 更新动画角度
    app.loading_angle += 0.05;
    if app.loading_angle > std::f32::consts::TAU {
        app.loading_angle -= std::f32::consts::TAU;
    }

    // 背景圆环
    ui.circle_stroke(
        center,
        30.0,
        egui::Stroke::new(5.0, egui::Color32::from_gray(100)),
    );

    // 动画圆弧
    let mut points = Vec::new();
    let segments = 32;
    let start_angle = app.loading_angle;
    let end_angle = start_angle + std::f32::consts::PI;

    for i in 0..=segments {
        let angle = start_angle + (end_angle - start_angle) * (i as f32 / segments as f32);
        let point = center + 30.0 * egui::Vec2::new(angle.cos(), angle.sin());
        points.push(point);
    }

    ui.add(egui::Shape::line(
        points,
        egui::Stroke::new(5.0, egui::Color32::from_rgb(66, 150, 250)),
    ));

    // 加载文字
    ui.text(
        center + egui::vec2(0.0, 50.0),
        egui::Align2::CENTER_CENTER,
        "加载中...",
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );

    // 强制持续重绘以保持动画
    ctx.request_repaint();
}
