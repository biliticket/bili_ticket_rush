use eframe::{epaint::Vec2};
mod app;
mod ui;
mod windows;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions{
        initial_window_size:Some(Vec2::new(1100.0, 600.0)),
        min_window_size:Some(Vec2::new(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "原神",
        options,
        Box::new(|cc| Box::new(app::Myapp::new(cc)))
    )

}