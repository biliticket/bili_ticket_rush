use eframe::{egui, epaint::Vec2};
use egui::FontId;
use std::fs::read;
use chrono::Local;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1100.0, 600.0)),
        min_window_size: Some(Vec2::new(800.0, 600.0)),
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
    is_loading :bool,  //加载动画
    loading_angle : f32, //加载动画角度
    background_texture: Option<egui::TextureHandle>,//背景纹理
    show_log_window: bool,
    logs: Vec<String>,

    }

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置中文字体
        Self::configure_fonts(&cc.egui_ctx);
        
        let mut app =Self {
            left_panel_width: 250.0,
            selected_tab: 0, // 默认选中第一个标签
            is_loading : false,
            loading_angle : 0.0,
            background_texture: None,
            //初始化日志
            show_log_window: false,
            logs: Vec::new(),
            
        };

        /* app.load_background(&cc.egui_ctx);*/
        app 
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
                //开始抢票按钮

                ui.vertical_centered(|ui| {
                    // 垂直居中
                    ui.add_space(ui.available_height() * 0.2);
                    
                    // 创建按钮
                    let button = egui::Button::new(
                        egui::RichText::new("开始抢票").size(40.0).color(egui::Color32::WHITE)
                    )
                    .min_size(egui::vec2(300.0, 150.0))
                    .fill(egui::Color32::from_rgb(131, 175, 155))
                    .rounding(20.0);
                    
                    // 只有点击按钮时才触发
                    if ui.add(button).clicked() {
                        self.is_loading = true;
                        
                        //待完善鉴权账号及有效信息
                    }
                });

                
            },
            1 => {
                self.show_log_window = true;
                ui.heading("预留监视公告栏2");
                ui.separator();
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
    //背景图
    /* fn load_background(&mut self, ctx:&egui::Context){
        let image_byte= include_bytes!("../assets/background.jpg");
        if let Ok(image) =image::load_from_memory(image_byte){
            let rgb_image = image.to_rgba8();
            let dimensions= rgb_image.dimensions();
            let image = egui::ColorImage::from_rgba_unmultiplied([dimensions.0 as usize, dimensions.1 as usize] , &rgb_image.into_raw());
            let texture = ctx.load_texture(
                "background", image, Default::default());
            self.background_texture = Some(texture);}}
 */
/* fn load_background(&mut self, ctx: &egui::Context) {
    println!("开始加载背景图片");
    //let image_path = "../assets/background.jpg";
    let image_byte = include_bytes!("../assets/background.jpg");
    
    println!("图片数据大小: {} 字节", image_byte.len());
    
    match image::load_from_memory(image_byte) {
        Ok(image) => {
            
            let rgb_image = image.to_rgba8();
            let dimensions = rgb_image.dimensions();
            println!("图片加载成功，尺寸: {:?}", dimensions);
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [dimensions.0 as usize, dimensions.1 as usize], 
                &rgb_image.into_raw()
            );
            let texture = ctx.load_texture("background", image, Default::default());
            self.background_texture = Some(texture);
            println!("背景纹理创建成功");
        },
        Err(e) => {
            println!("图片加载失败: {}", e);
        }
    }
} */
    
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /* // 加载背景
        if let Some(texture) = &self.background_texture {
            let screen_rect = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Background, // 确保在最底层
                egui::Id::new("background_layer")
            ));
            
            painter.image(
                texture.id(),
                screen_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 150)
            );
        } */
        // 创建左右两栏布局
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(self.left_panel_width)
            .width_range(150.0..=400.0)
            .show(ctx, |ui| {
                
                
                // 左侧五个选项
                let tab_names = ["开始抢票", "监视面板", "修改信息", "设置/微调", "帮助/关于"];
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
    // 如果在加载中，绘制覆盖层
    if self.is_loading {
        // 创建覆盖整个界面的区域
        let screen_rect = ctx.input(|i| i.screen_rect());
        let layer_id = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("loading_overlay"));
        let ui = ctx.layer_painter(layer_id);
        
        // 半透明背景
        ui.rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)
        );
        
        // 在屏幕中央显示加载动画
        let center = screen_rect.center();

        // 更新动画角度
        self.loading_angle += 0.05;
        if self.loading_angle > std::f32::consts::TAU {
            self.loading_angle -= std::f32::consts::TAU;
        }
        
        // 绘制动画
        // 背景圆环
        ui.circle_stroke(
            center,
            30.0,
            egui::Stroke::new(5.0, egui::Color32::from_gray(100))
        );
        
        // 动画圆弧
        let mut points = Vec::new();
        let segments = 32;
        let start_angle = self.loading_angle;
        let end_angle = start_angle + std::f32::consts::PI;
        
        for i in 0..=segments {
            let angle = start_angle + (end_angle - start_angle) * (i as f32 / segments as f32);
            let point = center + 30.0 * egui::Vec2::new(angle.cos(), angle.sin());
            points.push(point);
        }
        
        ui.add(egui::Shape::line(
            points,
            egui::Stroke::new(5.0, egui::Color32::from_rgb(66, 150, 250))
        ));

        // 加载文字
        ui.text(
            center + egui::vec2(0.0, 50.0),
            egui::Align2::CENTER_CENTER,
            "加载中...",
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE
        );
        
        // 强制持续重绘以保持动画
        ctx.request_repaint();
    }

    //日志窗口
    if self.show_log_window {
        // Using a temporary variable to track window close action
        let mut window_open = self.show_log_window;
        egui::Window::new("监视面板")
            .open(&mut window_open) // 使用临时变量
            .default_size([500.0, 400.0]) // 设置默认大小
            .resizable(true) // 允许调整大小
            .show(ctx, |ui| {
                // 顶部工具栏
                ui.horizontal(|ui| {
                    if ui.button("清空日志").clicked() {
                        self.logs.clear();
                    }
                    
                    if ui.button("添加测试日志").clicked() {
                        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                        self.logs.push(format!("[{}] 测试日志消息", timestamp));
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        if ui.button("❌").clicked() {
                            // 使用close_button替代直接修改window_open
                            self.show_log_window = false;
                        }
                    });
                });
                
                ui.separator();
                
                // 日志内容区域（可滚动）
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        // 显示当前状态
                        ui.label(format!("当前状态: {}", 
                            if self.is_loading {"正在抢票中..."} else {"空闲"}));
                        
                        ui.separator();
                        
                        // 显示所有日志
                        if self.logs.is_empty() {
                            ui.label("暂无日志记录");
                        } else {
                            for log in &self.logs {
                                ui.label(log);
                                ui.separator();
                            }
                        }
                    });
                // 底部状态栏
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(format!("共 {} 条日志", self.logs.len()));
                });
            });
        
        // 更新窗口状态
        self.show_log_window = window_open;
    
    }
       
    }
}