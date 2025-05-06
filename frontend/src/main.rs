// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::epaint::Vec2;
mod app;
mod ui;
mod windows;
fn main() -> Result<(), eframe::Error> {
    if let Err(e) = common::init_logger() {
        eprintln!("初始化日志失败，原因: {}", e);
    }
    log::info!("日志初始化成功");

    // 检查程序是否已经在运行
    if !common::utils::ensure_single_instance() {
        eprintln!("程序已经在运行中，请勿重复启动！");
        //增加休眠时间，防止程序过快退出
        std::thread::sleep(std::time::Duration::from_secs(5));
        std::process::exit(1);
    }

    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1200.0, 600.0)),
        min_window_size: Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "原神",
        options,
        Box::new(|cc| Box::new(app::Myapp::new(cc))),
    )
}
