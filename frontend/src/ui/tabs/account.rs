use eframe::egui;
use crate::app::Myapp;
use common::account::Account;
pub fn render(app: &mut Myapp, ui: &mut egui::Ui){
    ui.heading("我的账户");
    ui.separator();
    let example_account = Account{
        uid: 0,
        name: "请登录账号".to_string(),
        vip_label: "未登录，请登录账号".to_string(),
        level: "未登录".to_string(),
        cookie: "".to_string(), 
        csrf: "".to_string(),
        is_logged: false,
        account_status: "未登录".to_string(),
        is_active: false,
    };
    

    // 加载用户头像
    
    load_user_avatar(ui.ctx(), app);
    
    if let Some(texture) = &app.default_avatar_texture {
        let account_to_show = app.account_manager.accounts.first().unwrap_or(&example_account);
        show_user(ui, texture, account_to_show);
    
    

}
//show_user_control(ui,&app.user_info);
ui.separator();
if let Some(texture) = &app.default_avatar_texture {
    let account_to_show = app.account_manager.accounts.get(1).unwrap_or(&example_account);
    show_user(ui, texture, account_to_show);
    

}
ui.separator();



}
/// 将任意图片显示为圆形
/// - texture: 要显示的图像纹理
/// - size: 圆形图片的直径大小
fn draw_user_avatar(ui: &mut egui::Ui, texture: &egui::TextureHandle, size: f32) -> egui::Response {
    // 分配正方形区域
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(size, size),
        egui::Sense::click()
    );
    
    if ui.is_rect_visible(rect) {
        // 创建一个离屏渲染的自定义形状层
        let layer_id = egui::layers::LayerId::new(
            egui::layers::Order::Background, 
            egui::Id::new("circular_image")
        );
        
        let painter = ui.ctx().layer_painter(layer_id);
        
        // 绘制圆形背景 (这一步可选)
        painter.circle_filled(
            rect.center(), 
            size / 2.0,
            egui::Color32::from_rgb(220, 220, 240)
        );
        
        // 使用圆形纹理蒙版技术
        // 1. 创建一个与图像大小相同的圆形遮罩
        let circle_mask = egui::Shape::circle_filled(
            rect.center(), 
            size / 2.0 - 1.0,
            egui::Color32::WHITE
        );
        
        // 2. 将图像绘制为自定义着色器，使用圆形遮罩
        let uv = egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(1.0, 1.0)
        );
        
        // 使用裁剪圆绘制
        painter.add(circle_mask);
        
        // 以混合模式绘制图像，只在圆形区域内可见
        painter.image(
            texture.id(),
            rect,
            uv,
            egui::Color32::WHITE
        );
        
        // 添加边框
        painter.circle_stroke(
            rect.center(),
            size / 2.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(180, 180, 180, 180))
        );
    }
    
    response
}
fn load_texture_from_path(ctx: &egui::Context, path: &str, name: &str) -> Option<egui::TextureHandle> {
    use std::io::Read;
    
    match std::fs::File::open(path) {
        Ok(mut file) => {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() {
                match image::load_from_memory(&bytes) {
                    Ok(image) => {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        
                        Some(ctx.load_texture(
                            name,
                            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                            Default::default()
                        ))
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn load_user_avatar(ctx: &egui::Context, app: &mut Myapp) {
    // 确保默认头像已加载（作为回退选项）
    if app.default_avatar_texture.is_none() {
        load_default_avatar(ctx, app);
    }
    
    // 如果用户已登录且提供了头像路径，尝试加载
    /* if app.user_info.is_logged && app.user_info.avatar_texture.is_none() {
        if let Some(avatar_path) = &app.user_info.avatar_path {
            // 尝试加载用户头像
            app.user_info.avatar_texture = load_texture_from_path(ctx, avatar_path, "user_avatar");
            
            // 如果加载失败，记录日志
            if app.user_info.avatar_texture.is_none() {
                println!("无法加载用户头像: {}", avatar_path);
                // 用户也可以在这里添加一个日志
                app.add_log(&format!("无法加载头像: {}", avatar_path));
            }
        }
    } */
}
// 加载默认头像
fn load_default_avatar(ctx: &egui::Context, app: &mut Myapp) {
    // 使用include_bytes!宏将图片直接嵌入到二进制文件中
    // 路径是相对于项目根目录的
    let default_avatar_bytes = include_bytes!("../../../assets/default_avatar.jpg");
    
    // 从内存中加载图片
    match image::load_from_memory(default_avatar_bytes) {
        Ok(image) => {
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            
            app.default_avatar_texture = Some(ctx.load_texture(
                "default_avatar",
                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                Default::default()
            ));
        },
        Err(_) => {
            // 图片加载失败，生成占位符头像
            app.default_avatar_texture = generate_placeholder_avatar(ctx);
        }
    }
}

// 生成一个占位符头像
fn generate_placeholder_avatar(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let size = 128; // 头像尺寸
    let mut image_data = vec![0; size * size * 4];
    
    // 生成一个简单的渐变图案
    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) * 4;
            // 浅蓝色调渐变
            image_data[i] = 180; // R
            image_data[i + 1] = 180 + (y as u8) / 2; // G
            image_data[i + 2] = 230; // B
            image_data[i + 3] = 255; // A
        }
    }
    
    Some(ctx.load_texture(
        "default_avatar",
        egui::ColorImage::from_rgba_unmultiplied([size, size], &image_data),
        Default::default()
    ))
}

fn show_user( //显示用户头像等信息
    ui: &mut egui::Ui, 
    texture: &egui::TextureHandle, 
    user: &Account,
) {
    // 创建圆角长方形框架
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(245, 245, 250))  // 背景色
        .rounding(12.0)  // 圆角半径
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 220)))  // 边框
        .inner_margin(egui::Margin { left: 10.0, right: 20.0, top: 15.0, bottom: 15.0 })  // 内边距
        .show(ui, |ui| {
            // 水平布局放置图片和文字
            ui.horizontal(|ui| {
                // 左侧图片区域，这里使用小尺寸的圆形图片
                let image_size = 84.0;
                draw_user_avatar(ui, texture, image_size);
                
                ui.add_space(12.0);  // 图片和文字之间的间距
                
                // 右侧文字区域
                ui.vertical(|ui| {
                    //第一行
                    //显示uid和昵称
                    ui.horizontal(|ui|{
                        ui.add(egui::widgets::Label::new(
                            egui::RichText::new(&user.name)
                                .size(30.0)
                                .strong()
                                .color(egui::Color32::from_rgb(60, 60, 80))
                        ));
                        ui.add_space(15.0);
                        ui.add(egui::widgets::Label::new(
                            egui::RichText::new(format!("UID: {}", user.uid))
                                .color(egui::Color32::from_rgb(100, 100, 120))
                                .size(16.0)
                        ));
                    });
                    //第二行
                    ui.add_space(10.0);
                    //显示大会员
                    ui.horizontal(|ui|{

                        match user.vip_label.as_str(){
                            "月度大会员"=> {
                                egui::Frame::none()
                                .fill(egui::Color32::from_rgb(251, 114, 153)) // 粉色背景 #FB7299
                                .rounding(10.0)  // 圆角
                                
                                .inner_margin(egui::vec2(6.0, 3.0))  // 内边距
                                .show(ui, |ui| {
                                    // 白色文字 #FFFFFF
                                        ui.label(
                                              egui::RichText::new("月度大会员")
                                              .color(egui::Color32::from_rgb(255, 255, 255))
                                                // 白色文字
                                              .size(15.0)
                                                    );
                                 });
                            }
                            "年度大会员" =>{
                                egui::Frame::none()
                                .fill(egui::Color32::from_rgb(251, 114, 153))  // 粉色背景 #FB7299
                                .rounding(10.0)  // 圆角
                                .inner_margin(egui::vec2(6.0, 3.0))  // 内边距
                                .show(ui, |ui| {
                                    // 白色文字 #FFFFFF
                                        ui.label(
                                              egui::RichText::new("年度大会员")
                                              .color(egui::Color32::from_rgb(255, 255, 255))  // 白色文字
                                              .size(15.0)
                                                    );
                                 });
                            }
                            "十年大会员" =>{
                                egui::Frame::none()
                                .fill(egui::Color32::from_rgb(251, 114, 153))  // 粉色背景 #FB7299
                                .rounding(10.0)  // 圆角
                                .inner_margin(egui::vec2(6.0, 3.0))  // 内边距
                                .show(ui, |ui| {
                                    // 白色文字 #FFFFFF
                                        ui.label(
                                              egui::RichText::new("十年大会员")
                                              .color(egui::Color32::from_rgb(255, 255, 255))  // 白色文字
                                              .size(15.0)
                                                    );
                                 });
                            }
                            _ => {
                                egui::Frame::none()
                                 
                                .inner_margin(egui::vec2(6.0, 3.0))  // 内边距
                                .show(ui, |ui| {
                                    // 白色文字rgb(0, 0, 0)
                                        ui.label(
                                              egui::RichText::new("正式会员")
                                              .color(egui::Color32::from_rgb(0, 0, 0))  // 白色文字
                                              .size(15.0)
                                                    );
                                 });
                            }



                        }
                    })
                    
                });
                
                
            });
            ui.separator();
            ui.vertical(|ui|{
                ui.add_space(15.0);
                ui.horizontal(|ui|{
                    ui.add_space(15.0);
                    let button = egui::Button::new(
                      egui::RichText::new("登录").size(20.0).color(egui::Color32::WHITE)
                      )
                        .min_size(egui::vec2(120.0,50.0))
                        .fill(egui::Color32::from_rgb(102,204,255))
                        .rounding(15.0);//圆角成度
                    ui.add(button);
                    ui.add_space(150.0);
                    let button = egui::Button::new(
                        egui::RichText::new("添加购票人").size(18.0).color(egui::Color32::WHITE)
                        )
                          .min_size(egui::vec2(120.0,50.0))
                          .fill(egui::Color32::from_rgb(102,204,255))
                          .rounding(15.0);
                    ui.add(button);
                    ui.add_space(150.0);
                    let button = egui::Button::new(
                          egui::RichText::new("停止抢票").size(18.0).color(egui::Color32::WHITE)
                          )
                            .min_size(egui::vec2(120.0,50.0))
                            .fill(egui::Color32::from_rgb(102,204,255))
                            .rounding(15.0);
                    ui.add(button);
                    });
                    
                    
            })
        });
}


// 显示登录对话框（仅为示例，需要实现具体功能）
fn show_login_dialog(app: &mut Myapp) {
    // 这里可以触发登录弹窗或其他操作
    // 实际项目中，可能是显示一个新窗口或与后端交互
    println!("显示登录对话框");


}