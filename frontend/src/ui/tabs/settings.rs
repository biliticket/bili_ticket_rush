use eframe::egui;
use crate::app::Myapp;

fn on_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // 开关尺寸
    let width = 60.0;
    let height = 30.0;
    
    // 分配空间并获取响应
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(width, height), 
        egui::Sense::click()
    );
    
    // 处理点击
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    
    // 动画参数
    let animation_progress = ui.ctx().animate_bool(response.id, *on);
    let radius = height / 2.0;
    
    // 计算滑块位置
    let circle_x = rect.left() + radius + animation_progress * (width - height);
    
    // 绘制轨道
    ui.painter().rect_filled(
        rect.expand(-1.0), 
        radius, 
        if *on {
            egui::Color32::from_rgb(46, 182, 125)  // 启用状态颜色
        } else {
            egui::Color32::from_rgb(150, 150, 150)  // 禁用状态颜色
        }
    );
    
    // 绘制滑块
    ui.painter().circle_filled(
        egui::pos2(circle_x, rect.center().y),
        radius - 4.0,
        egui::Color32::WHITE
    );
    
    response
}

pub fn push_input(ui: &mut egui::Ui, title: &str,text: &mut String,hint: &str) -> bool{
    ui.label(
        egui::RichText::new(title)
              .size(15.0)                               
              .color(egui::Color32::from_rgb(0,0,0))  

              
    );
    ui.add_space(8.0);
    let input = egui::TextEdit::singleline( text)
                .hint_text(hint)//提示
                .desired_rows(1)//限制1行       
                .min_size(egui::vec2(120.0, 35.0));
                
                
    let response = ui.add(input);
    if response.changed(){
        *text = text.chars()//过滤非法字符
            .filter(|c| c.is_ascii_alphanumeric() || *c == '@' || *c == '.' || *c == '-' || *c == '_')
            .collect();
    }
    response.changed()

}
pub fn render(app: &mut Myapp, ui: &mut egui::Ui) {
    
        
    ui.heading("设置");
    ui.separator();
            //推送设置：
    // 创建圆角长方形框架  
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(245, 245, 250))  // 背景色
        .rounding(12.0)  // 圆角半径
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 220)))  // 边框
        .inner_margin(egui::Margin { left: 10.0, right: 20.0, top: 15.0, bottom: 15.0 })  // 内边距
        .show(ui, |ui| {
            //推送开关
            
            // 开关
            ui.horizontal(|ui| {
                
                ui.label(
                    egui::RichText::new("开启推送")
                          .size(18.0)                               
                          .color(egui::Color32::from_rgb(0,0,0))  

                          .strong()   
                );
                on_switch(ui, &mut app.push_config.enabled);
                let available = ui.available_width();
                ui.add_space(available-100.0);
                let button = egui::Button::new(
                    egui::RichText::new("测试推送").size(15.0).color(egui::Color32::WHITE)
                    )
                      .min_size(egui::vec2(100.0,50.0))
                      .fill(egui::Color32::from_rgb(102,204,255))
                      .rounding(15.0);//圆角成度
                  ui.add(button);

            });
            ui.separator();
            //推送设置
            ui.horizontal(|ui|{
                 
                push_input(ui, "bark推送：",&mut app.push_config.bark_token,"请输入推送地址");
                ui.add_space(12.0);
                push_input(ui, "pushplus推送：",&mut app.push_config.pushplus_token,"请输入推送地址");
                });
                //TODO补充每个推送方式使用方法

            ui.horizontal(|ui|{
                 
                push_input(ui, "方糖推送：",&mut app.push_config.fangtang_token,"请输入推送地址");
                ui.add_space(12.0);
                push_input(ui, "钉钉机器人推送：",&mut app.push_config.dingtalk_token,"请输入推送地址");
                });

            ui.horizontal(|ui|{
                push_input(ui, "企业微信推送：",&mut app.push_config.wechat_token,"请输入推送地址");
                ui.add_space(12.0);

                });
            
        });

   
}
// 创建设置卡片
fn settings_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    icon: &str,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(245, 245, 250))
        .rounding(8.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 240)))
        .inner_margin(15.0)
        .outer_margin(2.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("{} {}", icon, title));
            });
            ui.separator();
            ui.add_space(8.0);
            
            content(ui)
        })
        .inner
}
