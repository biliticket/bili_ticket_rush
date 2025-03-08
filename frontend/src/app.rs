use eframe::egui;
use crate::ui;
use crate::windows;

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
            user_info: UserInfo { username: String::from("未登录"), show_info: String::from(" LV6 | 哔哩哔哩大会员"), is_logged: true, avatar_texture: None , avatar_path: None},
            default_avatar_texture: None,
            push_settings: Some(PushSettings::default()),
            running_status: String::from("空闲ing"),
            ticket_id: String::from("85939"),
           
        }
    }

    pub fn add_log(&mut self, message: &str){
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(format!("[{}] {}",timestamp,message));
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

        
    }
}