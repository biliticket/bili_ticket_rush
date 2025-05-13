// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::epaint::Vec2;
mod app;
mod ui;
mod windows;
fn main() -> Result<(), eframe::Error> {
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");      // 强制软件渲染
    std::env::set_var("MESA_GL_VERSION_OVERRIDE", "3.3"); // 尝试覆盖 GL 版本
    std::env::set_var("GALLIUM_DRIVER", "llvmpipe");      // 使用 llvmpipe 软件渲染器
    if let Err(e) = common::init_logger() {
        eprintln!("初始化日志失败，原因: {}", e);
    }
    log::info!("日志初始化成功");

    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            if s.contains("swap") || s.contains("vsync") {
                log::warn!("图形渲染非致命错误: {}", s);
                // 继续允许程序运行
            } else {
                log::error!("程序panic: {}", panic_info);
            }
        } else {
            log::error!("程序panic: {}", panic_info);
        }
    }));

    // 检查程序是否已经在运行
    if !common::utils::ensure_single_instance() {
        eprintln!("程序已经在运行中，请勿重复启动！");
        //增加休眠时间，防止程序过快退出
        std::thread::sleep(std::time::Duration::from_secs(5));
        std::process::exit(1);
    }

    // 创建资源目录（如果不存在）
    create_resources_directory();

    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1200.0, 600.0)),
        min_window_size: Some(Vec2::new(800.0, 600.0)),
        vsync: false,
        
        ..Default::default()
    };

    eframe::run_native(
        "原神",
        options,
        Box::new(|cc| Box::new(app::Myapp::new(cc))),
    )
}

// 确保资源目录存在
fn create_resources_directory() {
    let resources_dir = std::path::Path::new("resources/fonts");
    if !resources_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(resources_dir) {
            log::warn!("无法创建资源目录: {}", e);
        }
    }
}
