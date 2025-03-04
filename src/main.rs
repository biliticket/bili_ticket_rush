use eframe::{egui, epaint::Vec2};
use egui::FontId;
use std::fs::read;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1024.0, 768.0)),
        min_window_size: Some(Vec2::new(600.0, 400.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "原神",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc)))
    )
}

struct MyApp {
    left_panel_width: f32,
    selected_tab: usize, // 当前选中标签页索引
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置中文字体
        Self::configure_fonts(&cc.egui_ctx);
        
        Self {
            left_panel_width: 250.0,
            selected_tab: 0, // 默认选中第一个标签
        }
    }

    // 配置字体函数
    fn configure_fonts(ctx: &egui::Context) {
        // 创建字体配置
        let mut fonts = egui::FontDefinitions::default();
        
        // 使用std::fs::read读取字体文件
        let font_data = read("C:/Windows/Fonts/msyh.ttc").unwrap_or_else(|_| {
            // 备用字体
            read("C:/Windows/Fonts/simhei.ttf").unwrap()
        });
        
        // 使用from_owned方法创建FontData
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            egui::FontData::from_owned(font_data)
        );
        
        // 将中文字体添加到所有字体族中
        for family in fonts.families.values_mut() {
            family.insert(0, "microsoft_yahei".to_owned());
        }
        
        // 应用字体
        ctx.set_fonts(fonts);
    }
    
    // 各标签页内容渲染函数
    fn render_tab_content(&mut self, ui: &mut egui::Ui) {
        match self.selected_tab {
            0 => {
                ui.heading("预留抢票界面公告栏1");
                ui.separator();
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                // 主要抢票按钮实现
                let botton_size = egui::vec2(300.0,150.0);
                let (rect, response)= ui.allocate_exact_size(botton_size, egui::Sense::click());
                ui.painter().rect_filled(rect, 20.0,egui::Color32::from_rgb(131, 175, 155));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "开始抢票",
                    FontId::proportional(20.0),
                    egui::Color32::WHITE,
                );
                if response.clicked(){
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "测试测试",
                        FontId::proportional(20.0),
                        egui::Color32::WHITE,
                    );
                    //待完善鉴权账号及有效信息
                }});
            },
            1 => {
                ui.heading("订单管理");
                ui.separator();
                ui.label("这里显示订单管理相关内容");
                
                // 示例表格
                egui::Grid::new("orders_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("订单号");
                        ui.label("演出名称");
                        ui.label("票数");
                        ui.label("状态");
                        ui.end_row();
                        
                        ui.label("2025031001");
                        ui.label("演唱会");
                        ui.label("2张");
                        ui.label("已付款");
                        ui.end_row();
                    });
            },
            2 => {
                ui.heading("抢票设置");
                ui.separator();
                ui.label("这里配置自动抢票参数");
                
                ui.checkbox(&mut true, "启用自动抢票");
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label("刷新间隔:");
                    ui.add(egui::Slider::new(&mut 1.0, 0.5..=5.0).suffix(" 秒"));
                });
                
                ui.horizontal(|ui| {
                    ui.label("最大尝试次数:");
                    ui.add(egui::DragValue::new(&mut 50).clamp_range(10..=100));
                });
            },
            3 => {
                ui.heading("账号管理");
                ui.separator();
                ui.label("这里管理B站账号信息");
                
                ui.horizontal(|ui| {
                    ui.label("用户名:");
                    ui.text_edit_singleline(&mut "示例用户".to_string());
                }); 
                
                ui.horizontal(|ui| {
                    ui.label("密码:");
                    ui.text_edit_singleline(&mut "********".to_string());
                });
                
                if ui.button("保存账号信息").clicked() {
                    // 保存账号信息
                }
            },
            4 => {
                ui.heading("系统设置");
                ui.separator();
                ui.label("这里是系统配置项");
                
                ui.checkbox(&mut true, "开机启动");
                ui.checkbox(&mut false, "启用通知提醒");
                ui.checkbox(&mut true, "自动更新");
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("缓存大小:");
                    ui.add(egui::Slider::new(&mut 500.0, 100.0..=1000.0).suffix(" MB"));
                });
            },
            _ => unreachable!(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 创建左右两栏布局
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(self.left_panel_width)
            .width_range(150.0..=400.0)
            .show(ctx, |ui| {
                
                
                // 左侧五个选项
                let tab_names = ["开始抢票", "查看战绩", "修改信息", "设置/微调", "帮助/关于"];
                let icons = ["😎", "🎫", "⚙️", "🔧", "🧐"]; // 使用表情符号作为简单图标
                
                // 均分空间
                let available_height = ui.available_height();
                let item_count = tab_names.len();
                let item_height = available_height / item_count as f32;
                
                
                for (idx, (name, icon)) in tab_names.iter().zip(icons.iter()).enumerate() {
                    let is_selected = self.selected_tab == idx;
                    
                    
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), item_height), 
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight), 
                        |ui| {
                            // 选项样式 - 选中时突出显示
                            let mut text = egui::RichText::new(format!("{} {}", icon, name)).size(16.0);
                            if is_selected {
                                text = text.strong().color(egui::Color32::from_rgb(66, 150, 250));
                            }
                            
                            
                            
                            if ui.selectable_value(&mut self.selected_tab, idx, text).clicked() {
                               
                            }
                        }
                    );
                }
            });
            
        egui::CentralPanel::default().show(ctx, |ui| {
            // 渲染右侧对应内容
            self.render_tab_content(ui);
        });
    }
}