use crate::app::Myapp;
use crate::windows::login_windows::create_qrcode;
use eframe::egui::{self, RichText};

pub fn show(app: &mut Myapp, ctx: &egui::Context) {
    let mut window_open = app.show_qr_windows.is_some();
    let qr_data = app.show_qr_windows.clone().unwrap_or_default();

    egui::Window::new("扫码支付")
        .open(&mut window_open)
        .resizable(false)
        .default_size([700.0, 400.0])
        .show(ctx, |ui| {
            if let Some(texture) = create_qrcode(ui.ctx(), qr_data.as_str()) {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    let rich_text = RichText::new("请使用 微信/支付宝 扫描二维码进行支付")
                        .size(20.0)
                        .color(egui::Color32::from_rgb(102, 204, 255));
                    ui.label(rich_text);
                    ui.add_space(20.0);
                    ui.image(&texture);
                });
            }
        });
    if !window_open {
        app.show_qr_windows = None;
    }
}
