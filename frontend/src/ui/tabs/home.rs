use eframe::egui;
use crate::app::Myapp;
use common::account::{Account};
use common::taskmanager::{TaskStatus, TicketRequest};
pub fn render(app: &mut Myapp, ui: &mut egui::Ui){
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
                    .fill(egui::Color32::from_rgb(102,204,255))
                    .rounding(20.0);
                    
                    // 只有点击按钮时才触发
                    if ui.add(button).clicked() {
                        app.is_loading = true;
                        app.running_status = String::from("抢票初始化...");
                        app.add_log("开始抢票流程");
                        
                        //app.account_manager.accounts.push();
                        //待完善鉴权账号及有效信息
                        if let Err(error) = start_grab_ticket(app,"123456","85939"){
                            app.add_log(&format!("抢票失败: {}", error));
                            app.is_loading = false;
                            app.running_status = String::from("抢票失败");
                        }
                        
                    }
                    
                });

}

/* pub fn start_grab_ticket(app: &mut Myapp) -> bool{
    if !check_setting_info(app){
        app.add_log("请先登录账号");
        return false
        
    }
    app.add_log("设置检测通过");
    if !check_input_ticket(app){
        app.add_log("请输入项目ID：");
        return false

    }
    true

}
pub fn check_setting_info( app: &mut Myapp) -> bool{
    if !app.user_info.is_login {
            app.is_loading = false;
            app.add_log("请先登录账号");
            // 弹出登录窗口
            return false
    }
    true
}

pub fn check_input_ticket( app: &mut Myapp)  -> bool{
    if app.ticket_id.is_empty(){
        return false

    }
    return true

}

 */

 pub fn start_grab_ticket(app: &mut Myapp, account_id: &str, ticket_id: &str) -> Result<(), String> {
    // 验证输入
    if ticket_id.is_empty() {
        return Err("请输入票务ID".to_string());
    }
    
    // 验证账号状态
    let account = app.account_manager.accounts.iter()
        .find(|a| a.uid ==  account_id.parse::<i64>().unwrap_or(-1))
        .ok_or("未找到账号")?;
    
    if !account.is_login {
        return Err("账号未登录".to_string());
    }
    
    // 创建请求
    let request = common::taskmanager::TaskRequest::TicketRequest (
        TicketRequest { ticket_id: ticket_id.to_string(),
        account_id: account_id.to_string(),}
    );
    println!("请求创建成功");
    // 提交任务
    match app.task_manager.submit_task(request) {
        Ok(task_id) => {
            // 创建任务记录
            let task = common::taskmanager::TicketTask {
                task_id: task_id.clone(),
                account_id: account_id.to_string(),
                ticket_id: ticket_id.to_string(),
                status: TaskStatus::Pending,
                start_time: Some(std::time::Instant::now()),
                result: None,
            };
            log::error!("任务创建成功");
            // 保存任务
            app.account_manager.active_tasks.insert(task_id, task);
            
            // 更新账号状态
            if let Some(account) = app.account_manager.accounts.iter_mut()
                .find(|a| a.uid ==  account_id.parse::<i64>().unwrap_or(-1)) {
                account.account_status = "忙碌".to_string();
            }
            
            log::info!("为账号 {} 创建抢票任务: {}", account_id, ticket_id);
            app.running_status = "抢票中...".to_string();
            
            Ok(())
        },
        Err(e) => Err(e),
    }
}