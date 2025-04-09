use std::u32;

use eframe::egui;
use crate::app::Myapp;
use common::account::{Account};
use common::taskmanager::{TaskStatus, TicketRequest, TaskManager_debug};


pub fn render(app: &mut Myapp, ui: &mut egui::Ui) {
    //页面标题
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading(egui::RichText::new("B站抢票小助手").size(32.0).strong());
        ui.add_space(10.0);
        ui.label(egui::RichText::new(TaskManager_debug())
            .size(14.0)
            .color(egui::Color32::from_rgb(255, 120, 50))
            .strong());
        ui.add_space(10.0);
        ui.label(egui::RichText::new("请输入项目ID或粘贴票务链接，点击开始抢票").size(16.0).color(egui::Color32::GRAY));
        ui.add_space(40.0);
        
        //输入区域
        ticket_input_area(ui, app);
    });
}

fn ticket_input_area(ui: &mut egui::Ui, app: &mut Myapp) {
    //居中布局的输入框和按钮组合
    ui.vertical_centered(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 20.0);
        
        //输入框布局
        let response = styled_ticket_input(ui, &mut app.ticket_id);
        
        //抢票按钮
        if styled_grab_button(ui).clicked() {
            if !check_input_ticket(&mut app.ticket_id) {app.show_log_window = true; return};
            if app.account_manager.accounts.is_empty() {
                log::info!("没有可用账号，请登录账号");
                app.show_login_windows = true;
                return
            }
            
           
        }
        
        //底部状态文本
        ui.add_space(30.0);
        let status_text = match app.is_loading {
            true => egui::RichText::new(&app.running_status).color(egui::Color32::from_rgb(255, 165, 0)),
            false => egui::RichText::new("等待开始...").color(egui::Color32::GRAY),
        };
        ui.label(status_text);
    });
}

//输入框
fn styled_ticket_input(ui: &mut egui::Ui, text: &mut String) -> egui::Response {
    //创建一个适当大小的容器
    let desired_width = 250.0;
    
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - desired_width) / 2.0);
        
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(245, 245, 250))
            .rounding(10.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 220)))
            .shadow(egui::epaint::Shadow::small_light())
            .inner_margin(egui::vec2(12.0, 10.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                
                // 左侧图标
                ui.label(egui::RichText::new("🎫").size(18.0));
                
                // 输入框
                let font_id = egui::FontId::new(20.0, egui::FontFamily::Proportional);
                ui.style_mut().override_font_id = Some(font_id.clone());
                
                let input = egui::TextEdit::singleline(text)
                    .hint_text("输入票务ID")
                    .desired_width(180.0)
                    .margin(egui::vec2(0.0, 6.0))
                    .frame(false);
                
                ui.add(input)
            })
            .inner
    }).inner
}

//抢票按钮
fn styled_grab_button(ui: &mut egui::Ui) -> egui::Response {
    let button_width = 200.0;
    let button_height = 60.0;
    
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - button_width) / 2.0);
        
        let button = egui::Button::new(
            egui::RichText::new("开始抢票")
                .size(24.0)
                .strong()
                .color(egui::Color32::from_rgb(255,255,255))
        )
        .min_size(egui::vec2(button_width, button_height))
        .fill(egui::Color32::from_rgb(102, 204, 255))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 118, 210)))
        .rounding(12.0);
        
        ui.add(button)
    }).inner
}

fn check_input_ticket(ticket_id: &mut String) -> bool{
    //检查输入的票务ID是否有效
    if ticket_id.is_empty(){
        log::info!("请输入有效的票务id");
        return false;
    }
    if ticket_id.contains("https") {
        if let Some(position) = ticket_id.find("id="){
            let mut id = ticket_id.split_off(position+3);
            if id.contains("&") {
                let position = id.find("&").unwrap();
                id.truncate(position);
            }
            if id.len() == 5 || id.len() == 6 {
                match id.parse::<u32>(){
                    Ok(_) => {
                        log::info!("获取到的id为：{}", id);
                        *ticket_id = id;
                        return true;
                    }
                    Err(_) => {
                        log::error!("输入的id不合法，请检查输入，可尝试直接输入id");
                        return false;
                    }
                }
            }
            
            

        }else{
            log::error!("未找到对应的id，请不要使用b23开头的短连接，正确连接以show.bilibili或mall.bilibili开头");
            return false;
        }
    }
    match ticket_id.parse::<u32>() {
        Ok(_) => {
            log::info!("获取到的id为：{}", ticket_id);
            return true;
        }
        Err(_) => {
            log::error!("输入的id不是数字类型，请检查输入");
        }
    }
    return false;
}