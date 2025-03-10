use eframe::egui;
use crate::ui;
use crate::windows;
use std::collections::HashMap;
use common::taskmanager::{TaskManager, TaskStatus, TicketRequest, TaskResult,TicketTask};
use backend::taskmanager::TaskManagerImpl;
use common::LOG_COLLECTOR;


//UI
pub struct Myapp{  
    //ui
    pub left_panel_width: f32,  //左面板宽度
    pub selected_tab: usize,    //左侧已选中标签
    //加载动画
    pub loading_angle: f32,
    pub is_loading: bool,
    //运行状态（显示用）
    pub running_status: String,
    //自定义背景图  （未启用，效果不好，预留暂时不用）
    pub background_texture: Option<egui::TextureHandle>,
    //日志记录
    pub logs: Vec<String>,
    pub show_log_window: bool,
    //用户信息
    pub user_info: UserInfo,
    pub default_avatar_texture: Option<egui::TextureHandle>, // 默认头像
    

    pub push_settings: Option<PushSettings>,
    pub ticket_id: String,
   
   //任务管理
   pub task_manager: Box<dyn TaskManager>,
   pub account_manager: AccountManager,




}

//账号管理
pub struct AccountManager{
    pub accounts: Vec<UserAccount>,
    pub active_tasks: HashMap<String, TicketTask>,
}

//账号
pub struct UserAccount{
    pub uid : String,        // UID
    pub username: String,   // 用户名
    pub is_logged: bool,    // 是否已登录
    pub AccountStatus: String,     // 状态
}





//账号状态
pub enum AccountStatus{
    Idle,   // 空闲
    Running,    // 运行中
    Error(String),  // 错误
}



pub struct UserInfo{
    pub username: String,
    pub show_info: String,
    pub is_logged: bool,
    pub avatar_texture: Option<egui::TextureHandle>, // 用户头像（如果已登录）
    pub avatar_path: Option<String>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PushType {
    None,
    Bark,
    PushPlus,
    FangTang,
    DingTalkWebhook,
    WeChatWebhook,
    Email,
}

// 邮箱设置

pub struct EmailSettings {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub recipient: String,
    pub use_ssl: bool,
}

// 推送设置
#[derive(Default)]
pub struct PushSettings {
    pub enabled: bool,
    pub push_type: PushType,
    
    // 各种推送服务的配置
    pub bark_url: String,
    pub pushplus_token: String,
    pub fangtang_key: String,
    pub dingtalk_webhook: String,
    pub dingtalk_secret: String,
    pub wechat_webhook: String,
    
    // 邮箱设置
    pub email_settings: EmailSettings,
    
    // 通用设置
    pub notification_title: String,
    
    // 程序设置
    pub start_with_system: bool,
    pub minimize_to_tray: bool,
}

impl Default for PushType {
    fn default() -> Self {
        PushType::None
    }
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            server: String::new(),
            port: 465,
            username: String::new(),
            password: String::new(),
            recipient: String::new(),
            use_ssl: true,
        }
    }
}

impl Myapp{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        
        //中文字体
        ui::fonts::configure_fonts(&cc.egui_ctx);
        
        Self {
            left_panel_width: 250.0,
            selected_tab: 0,
            is_loading: false,
            loading_angle: 0.0,
            background_texture: None,
            show_log_window: false,
            logs: Vec::new(),
            user_info: UserInfo { username: String::from("未登录"), show_info: String::from(" LV6 | 哔哩哔哩大会员"), is_logged: false, avatar_texture: None , avatar_path: None},
            default_avatar_texture: None,
            push_settings: Some(PushSettings::default()),
            running_status: String::from("空闲ing"),
            ticket_id: String::from("85939"),
             // 初始化任务管理器
             task_manager: Box::new(TaskManagerImpl::new()),
             account_manager: AccountManager {
                 accounts: Vec::new(),
                 active_tasks: HashMap::new(),
             },
           
        }
    }

    pub fn add_log(&mut self, message: &str){
        self.logs.push(format!("{}",message));
    }
    // 处理任务结果的方法
    fn process_task_results(&mut self) {
        // 获取所有可用结果
        let results = self.task_manager.get_results();
        
        // 存储需要记录的日志消息
        let mut pending_logs = Vec::new();
        let mut account_updates = Vec::new();
        
        for result in results {
            // 更新任务状态
            if let Some(task) = self.account_manager.active_tasks.get_mut(&result.task_id) {
                // 记录需要更新的账号ID
                let account_id = task.account_id.clone();
                
                match &result.result {
                    Ok(ticket_result) => {
                        // 更新任务状态
                        task.status = TaskStatus::Completed(ticket_result.success);
                        
                        // 直接克隆值而非引用
                        task.result = Some((*ticket_result).clone());
                        
                        // 准备日志，但不立即添加
                        let message = if ticket_result.success {
                            format!("抢票成功! 订单号: {}", 
                                ticket_result.order_id.as_ref().unwrap_or(&String::new()))
                        } else {
                            format!("抢票未成功: {}", 
                                ticket_result.message.as_ref().unwrap_or(&String::new()))
                        };
                        
                        pending_logs.push(message);
                    },
                    Err(error) => {
                        // 更新失败状态
                        task.status = TaskStatus::Failed(error.clone());
                        pending_logs.push(format!("任务失败: {}", error));
                    }
                }
                
                // 将账号添加到待更新列表
                account_updates.push(account_id);
            }
        }
        
        // 更新账号状态
        for account_id in account_updates {
            if let Some(account) = self.account_manager.accounts.iter_mut()
                .find(|a| a.uid == account_id) {
                account.AccountStatus = "空闲".to_string();
            }
        }
        
        // 一次性添加所有日志，避免借用冲突
        for message in pending_logs {
            self.add_log(&message);
        }
    }

    pub fn add_log_windows(&mut self) { //从env_log添加日志进窗口
        if let Some(logs) = LOG_COLLECTOR.lock().unwrap().get_logs() {
            for log in logs {
                self.add_log(&log);
            }
        }
    }
}



impl eframe::App for Myapp{
    fn update(&mut self, ctx:&egui::Context, frame: &mut eframe::Frame){
        //侧栏
        ui::sidebar::render_sidebar(self,ctx);

        //主窗口
        egui::CentralPanel::default().show(ctx, |ui|{
            ui::tabs::render_tab_content(self, ui);
        } );


        //加载动画
        if self.is_loading{
            ui::loading::render_loading_overlay(self, ctx);
        }

        //日志
        if self.show_log_window{
            windows::log_windows::show(self, ctx);
        }

        //处理异步任务结果
        self.process_task_results();

        //从env_log添加日志进窗口
        self.add_log_windows();

        
    }

    
}



// 在UI中实现抢票功能
