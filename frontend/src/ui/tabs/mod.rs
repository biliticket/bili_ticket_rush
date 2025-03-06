pub mod home;
pub mod monitor;
pub mod account;
/* pub mod settings;
pub mod help;  */

use eframe::egui;
use crate::app::Myapp;


pub fn render_tab_content(app: &mut Myapp, ui: &mut egui::Ui) {
    match app.selected_tab {
        0 => home::render(app, ui),
        1 => monitor::render(app, ui),
        2 => account::render(app, ui),
        /* 3 => settings::render(app, ui),
        4 => help::render(app, ui),  */
        _ => unreachable!(),
    }
}