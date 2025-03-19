use eframe::{epaint::Vec2};
mod app;
mod ui;
mod windows;
fn main() -> Result<(), eframe::Error> {
    
    if let Err(e) = common::init_logger() {
        eprintln!("初始化日志失败，原因: {}", e);
    }
    log::info!("日志初始化成功");

    let options = eframe::NativeOptions{
        initial_window_size:Some(Vec2::new(1200.0, 600.0)),
        min_window_size:Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "原神",
        options,
        Box::new(|cc| Box::new(app::Myapp::new(cc)))
    )

}