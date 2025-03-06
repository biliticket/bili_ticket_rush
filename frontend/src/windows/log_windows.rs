use eframe::egui;
use crate::app::Myapp;

pub fn show(app: &mut Myapp, ctx: &egui::Context) {
    let mut window_open = app.show_log_window;
    
    egui::Window::new("监视面板")
        .open(&mut window_open)
        .default_size([500.0, 400.0])
        .resizable(true)
        .show(ctx, |ui| {
            // 顶部工具栏
            ui.horizontal(|ui| {
                if ui.button("清空日志").clicked() {
                    app.logs.clear();
                }
                
                if ui.button("添加测试日志").clicked() {
                    app.add_log("测试日志消息");
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.button("❌").clicked() {
                        app.show_log_window = false;
                    }
                });
            });
            
            ui.separator();
            
            // 日志内容区域
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // 显示当前状态
                    ui.label(format!("当前状态: {}", 
                        if app.is_loading {"正在抢票中..."} else {"空闲"}));
                    
                    ui.separator();
                    
                    // 显示所有日志
                    if app.logs.is_empty() {
                        ui.label("暂无日志记录");
                    } else {
                        for log in &app.logs {
                            ui.label(log);
                            ui.separator();
                        }
                    }
                });
                
            // 底部状态栏
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label(format!("共 {} 条日志", app.logs.len()));
            });
        });
    
    // 更新窗口状态
    app.show_log_window = window_open;
}