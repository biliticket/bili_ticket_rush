use crate::app::Myapp;
use eframe::egui;

pub fn render_sidebar(app: &mut Myapp, ctx: &egui::Context) {
    // 创建左右两栏布局
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(app.left_panel_width)
        .width_range(150.0..=400.0)
        .show(ctx, |ui| {
            // 左侧五个选项
            let tab_names = ["开始抢票", "监视面板", "修改信息", "设置/微调", "帮助/关于"];
            let icons = ["😎", "🎫", "📁", "🔧", "📋"]; // 使用表情符号作为简单图标

            // 均分空间
            let available_height = ui.available_height();
            let item_count = tab_names.len();
            let item_height = available_height / item_count as f32;

            for (idx, (name, icon)) in tab_names.iter().zip(icons.iter()).enumerate() {
                let is_selected = app.selected_tab == idx;

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), item_height),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        // 选项样式 - 选中时突出显示
                        let mut text = egui::RichText::new(format!("{} {}", icon, name)).size(16.0);
                        if is_selected {
                            text = text.strong().color(egui::Color32::from_rgb(255, 255, 255));
                        }

                        if ui
                            .selectable_value(&mut app.selected_tab, idx, text)
                            .clicked()
                        {}
                    },
                );
            }
        });
}
