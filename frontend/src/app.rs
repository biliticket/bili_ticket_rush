use common::show_orderlist::Order;
use common::show_orderlist::OrderResponse;
use common::taskmanager::GetAllorderRequest;
use common::taskmanager::TaskRequest;
use eframe::egui;

use crate::ui;
use crate::windows;
use std::collections::HashMap;
use common::taskmanager::{TaskManager, TaskStatus, TicketRequest, TaskResult,TicketTask};
use backend::taskmanager::TaskManagerImpl;
use common::LOG_COLLECTOR;
use common::account::{Account,add_account};
use common::utils::{Config,save_config};
use common::utility::CustomConfig;
use common::push::{PushConfig, SmtpConfig};
use common::login::LoginInput;
use crate::windows::login_windows::LoginTexture;
use crate::windows::add_buyer::AddBuyerInput;
use reqwest::{Client, header};


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
    //登录窗口
    pub show_login_windows: bool,
    //用户信息
    
    pub default_avatar_texture: Option<egui::TextureHandle>, // 默认头像
        

    
    pub ticket_id: String,
   
   //任务管理
   pub task_manager: Box<dyn TaskManager>,
   pub account_manager: AccountManager,

   //推送设置
   pub push_config: PushConfig,

   //config
    pub config: Config,

    //自定义配置
    pub custom_config: CustomConfig,
    //登录背景
    pub login_texture: LoginTexture,

    //登录方式
    pub login_method: String,
    
    //用于登录的client，登录后存入account
    pub client: Client,

    //登录用，防止重复刷新二维码
    pub login_qrcode_url: Option<String>,

    //登录用异步回调taskid
    pub qrcode_polling_task_id: Option<String>,

    //登录用输入
    pub login_input: LoginInput,

    //登录用发送短信任务id
    pub pending_sms_task_id: Option<String>,

   /*  //账号1 client
    pub client1: Option<Client>,
    //账号2 client
    pub client2: Option<Client>, */

    //默认ua
    pub default_ua: String,

    //发送短信captcha_key
    pub sms_captcha_key: String,

    //删除账号
    pub delete_account: Option<String>,

    //cookie登录，暂存cookie
    pub cookie_login: Option<String>,

    //该账号开启抢票开关
    pub account_switch: Option<AccountSwitch>,

    //添加购票人的输入
    pub add_buyer_input: AddBuyerInput,

    //添加购票人窗口
    pub show_add_buyer_window: Option<String>, //如果是bool类型会导致无法对应申请添加的账号，
                                        //所以使用string表示要添加购票人的账号的uid

    pub show_orderlist_window: Option<String>, //订单列表窗口的账号uid

    pub total_order_data: Option<OrderData>, //订单数据缓存

    pub orderlist_need_reload: bool, //订单列表是否需要重新加载

    pub orderlist_last_request_time: Option<std::time::Instant>,  // 上次请求的时间
    pub orderlist_requesting: bool,  // 是否正在请求中
                                   
                                    }


//账号管理

pub struct AccountManager{
    pub accounts: Vec<Account>,
    pub active_tasks: HashMap<String, TicketTask>,
}

//获取全部订单结构体（便于区分）
pub struct OrderData {
    pub account_id: String,
    pub data : Option<OrderResponse>,
}




impl Myapp{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        
        //中文字体
        ui::fonts::configure_fonts(&cc.egui_ctx);
        let config = match Config::load_config() {
            Ok(load_config) => {
                log::info!("配置文件加载成功");
                load_config
            },
            Err(e) => {
                log::error!("配置文件加载失败: {}", e);
                Config::new()
            }
        };
        
        
        
        let mut app = Self {
            left_panel_width: 250.0,
            selected_tab: 0,
            is_loading: false,
            loading_angle: 0.0,
            background_texture: None,
            show_log_window: false,
            show_login_windows: false,
            logs: Vec::new(),
            client: Client::new(),
            default_avatar_texture: None,
            running_status: String::from("空闲ing"),
            ticket_id: String::from("85939"),
             // 初始化任务管理器
             task_manager: Box::new(TaskManagerImpl::new()),
             account_manager: AccountManager {
                 accounts: Config::load_all_accounts(),
                 active_tasks: HashMap::new(),
             },
             
            push_config : match serde_json::from_value::<PushConfig>(config["push_config"].clone()) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("无法解析推送配置: {}, 使用默认值", e);
                    PushConfig::new()
                }
            },
        
            
               
            custom_config: match serde_json::from_value::<CustomConfig>(config["custom_config"].clone()) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("无法解析自定义配置: {}, 使用默认值", e);
                    CustomConfig::new()
                }
            },
            config: config.clone(),
            login_texture: LoginTexture { left_conrner_texture: None , right_conrner_texture: None},

                login_method: "扫码登录".to_string(),
              
                
                login_qrcode_url: None,
                qrcode_polling_task_id: None,
                login_input: LoginInput{
                    phone: String::new(),
                    account: String::new(),
                    password: String::new(),
                    cookie: String::new(),
                    sms_code: String::new(),
                },
            pending_sms_task_id: None,
            
            default_ua: String::from("Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36"),
            sms_captcha_key: String::new(),
            delete_account: None,
            cookie_login: None,
            account_switch: None,
            add_buyer_input: AddBuyerInput {
                name: String::new(),
                phone: String::new(),
                id_type: 0,
                id_number: String::new(),
                as_default_buyer: false,
            },
            show_add_buyer_window: None,
            show_orderlist_window: None,
            total_order_data: None,
            orderlist_need_reload: false,
            orderlist_last_request_time: None,
            orderlist_requesting: false,

        };
        // 初始化每个账号的 client
        for account in &mut app.account_manager.accounts {
        account.ensure_client();
        log::debug!("为账号 {} 初始化了专属客户端", account.name);
    }

    //初始化client和ua
    let random_value = generate_random_string(8);
    app.default_ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.0 {}", 
        random_value
    );
    if config["custom_config"]["enable_custom_ua"].as_bool().unwrap_or(false) && !config["custom_config"]["custom_ua"].is_null() {
        app.default_ua = config["custom_config"]["custom_ua"].as_str().unwrap_or(&app.default_ua).to_string();
        
    }
    let new_client = create_client(app.default_ua.clone());
    app.client = new_client;
        
      
    app
        
        
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
            match result {
                // 处理票务任务结果
                TaskResult::TicketResult(ticket_result) => {
                    // 获取任务ID和账号ID
                    let task_id = ticket_result.task_id.clone();
                    let account_id = ticket_result.account_id.clone();
                    
                    // 更新任务状态
                    if let Some(task) = self.account_manager.active_tasks.get_mut(&task_id) {
                        match &ticket_result.result {
                            Ok(ticket_result_data) => {
                                // 更新任务状态
                                task.status = TaskStatus::Completed(ticket_result_data.success);
                                
                                // 直接克隆值而非引用
                                task.result = Some(ticket_result_data.clone());
                                
                                // 准备日志，但不立即添加
                                let message = if ticket_result_data.success {
                                    format!("抢票成功! 订单号: {}", 
                                        ticket_result_data.order_id.as_ref().unwrap_or(&String::new()))
                                } else {
                                    format!("抢票未成功: {}", 
                                        ticket_result_data.message.as_ref().unwrap_or(&String::new()))
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
                },
                
                //处理qrcode登录结果
                TaskResult::QrCodeLoginResult(qrcode_result) => {
                    // 二维码登录的处理逻辑
                    match qrcode_result.status {
                        common::login::QrCodeLoginStatus::Success(cookie) => {
                            log::info!("二维码登录成功!");
                            
                            
                            if let Some(cookie_str) = qrcode_result.cookie {
                                
                                self.handle_login_success(&cookie_str);
                            }
                        },
                        common::login::QrCodeLoginStatus::Failed(err) => {
                            log::error!("二维码登录失败: {}", err);
                        },
                        common::login::QrCodeLoginStatus::Expired => {
                            log::warn!("二维码已过期，请刷新");
                        },
                        _ => {
                            
                        }
                    }
                }
                TaskResult::LoginSmsResult(sms_result) => {
                    // 处理短信登录结果
                    if sms_result.success {
                        self.sms_captcha_key = sms_result.message.clone();
                        log::debug!("发送captchakey：{}",sms_result.message);
                        log::info!("短信发送成功 ");
                    } else {
                        log::error!("短信发送失败: {}", sms_result.message);
                    }
                }
                TaskResult::SubmitSmsLoginResult(submit_result) => {
                    if submit_result.success{
                        if let Some(cookie_str) = submit_result.cookie {
                            self.handle_login_success(&cookie_str);
                        }
                    } else {
                        log::error!("短信登录失败: {}", submit_result.message);
                    }
                }
                TaskResult::PushResult(push_result) => {
                    // 处理推送结果
                    if push_result.success {
                        log::info!("推送成功: {}", push_result.message);
                    } else {
                        log::error!("推送失败: {}", push_result.message);
                    }
                }
                TaskResult::GetAllorderRequestResult(order_result) => {
                    // 处理订单请求结果
                    if order_result.success {
                        self.total_order_data = Some(OrderData {
                            account_id: order_result.account_id.clone(),
                            data: order_result.order_info.clone(),
                        });
                        log::info!("账号 {} 订单请求成功", order_result.account_id);
                    } else {    
                        log::error!("账号 {} 订单请求失败", order_result.account_id);
                    }
                }
            }
        }
        
        // 更新账号状态
        for account_id in account_updates {
            if let Some(account) = self.account_manager.accounts.iter_mut()
                .find(|a| a.uid == account_id.parse::<i64>().unwrap_or(-1)) {
                account.account_status = "空闲".to_string();
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

    pub fn handle_login_success(&mut self, cookie: &str) {
    log::debug!("登录成功，cookie: {}", cookie);
    match add_account(cookie, &self.client,&self.default_ua){
        Ok(account) => {
            self.account_manager.accounts.push(account.clone());
            match save_config(&mut self.config, None, None, Some(account.clone())){
                Ok(_) => {
                    log::info!("登录成功，账号已添加");
                    self.show_login_windows = false;
                },
                Err(e) => {
                    log::error!("登录成功，但保存账号失败: {}", e);
                }
            }
            log::info!("登录成功，账号已添加");
        },
        Err(e) => {
            log::error!("登录成功，但添加账号失败: {}", e);
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

        //登录窗口
        if self.show_login_windows{
            
            windows::login_windows::show(self, ctx);
        }

        //处理异步任务结果
        self.process_task_results();

        //从env_log添加日志进窗口
        self.add_log_windows();

        //删除账号
        if let Some(account_id) = self.delete_account.take() {
            self.account_manager.accounts.retain(|account| account.uid != account_id.parse::<i64>().unwrap_or(-1));
            self.config.delete_account(account_id.parse::<i64>().unwrap_or(-1));
            log::info!("账号 {} 已删除", account_id);
        }

        //检测是否有cookie
        if let Some(cookie) = &self.cookie_login {
            log::info!("检测到cookie: {}", cookie);
            if let Ok(account) = add_account(cookie, &self.client,&self.default_ua) {
                self.account_manager.accounts.push(account.clone());
                match save_config(&mut self.config, None, None, Some(account.clone())){
                    Ok(_) => {
                        log::info!("cookie登录成功，账号已添加");
                    },
                    Err(e) => {
                        log::error!("cookie登录成功，但保存账号失败: {}", e);
                    }
                }
                log::info!("cookie登录成功，账号已添加");
                self.cookie_login = None; // 清空cookie
            } else {
                log::error!("cookie登录失败");
                self.cookie_login = None;
            }
        }
        

        //检测是否有更新账号开关
        if let Some(account_switch) = &self.account_switch {
            log::info!("检测到账号开关: {}", account_switch.uid);
            if let Some(account) = self.account_manager.accounts.iter_mut().find(|a| a.uid == account_switch.uid.parse::<i64>().unwrap_or(-1)) {
                account.is_active = account_switch.switch;
                log::info!("账号 {} 开关已更新", account_switch.uid);
            } else {
                log::error!("未找到账号 {}", account_switch.uid);
            }
            self.account_switch = None; // 清空开关
        }

        //开启添加购票人窗口？
        if let Some(account_id) = &self.show_add_buyer_window {
            if account_id == "0"{
                self.show_add_buyer_window = None;
                
            }
            else{
                windows::add_buyer::show(self, ctx, account_id.clone().as_str());
            }
            
        }

        //开启查看订单窗口？
        if let Some(uid) = &self.show_orderlist_window {
            let account_id = uid.clone().parse::<i64>().unwrap_or(0);
            if account_id == 0{
                self.show_orderlist_window = None;
                
            }
            else{
                
                let account = self.account_manager.accounts.iter_mut().find(|a| a.uid == account_id.clone()).unwrap();
                let client = match account.client.clone() {
                    Some(client) => client,
                    None => {
                        log::error!("账号 {} 的客户端未初始化", account.name);
                        self.show_orderlist_window = None;
                        return;
                    }
                };
                if self.total_order_data.is_none() {
                    self.orderlist_need_reload = true;
                   

                }else{
                    if self.total_order_data.as_ref().unwrap().account_id == uid.clone(){
                        
                    }else{
                        log::error!("账号不匹配，正在重新加载");
                        self.orderlist_need_reload = true;
                        
                        
                    }
                    
                }

                // 防止频繁请求的逻辑
                let should_request = self.orderlist_need_reload && !self.orderlist_requesting && 
                match self.orderlist_last_request_time {
                     Some(last_time) => last_time.elapsed() > std::time::Duration::from_secs(5), // 5秒冷却时间
                     None => true, // 从未请求过，允许请求
                        };
                if should_request {
                    log::debug!("提交订单请求 (冷却期已过)");
                     self.orderlist_requesting = true;  // 标记为正在请求中
                     self.orderlist_last_request_time = Some(std::time::Instant::now());
                     self.orderlist_need_reload = false;
                    submit_get_total_order(&mut self.task_manager, &client, account);
                    self.orderlist_need_reload = false;
                }
                windows::show_orderlist::show(self, ctx);
            }
            
        }

        
    }

    
}


pub fn submit_get_total_order(task_manager: &mut Box<dyn TaskManager>,client: &Client, account: &Account){
    let request = TaskRequest::GetAllorderRequest(GetAllorderRequest{
        task_id: "".to_string(),
        account_id: account.uid.to_string().clone(),
        client: client.clone(),
        cookies: account.cookie.clone(),
        //ua: account.user_agent.clone(),
        status: TaskStatus::Pending,
        start_time: None,
    });

match task_manager.submit_task(request) {
    Ok(task_id) => {
        log::info!("订单请求提交成功，任务ID: {}", task_id);
    }
    Err(e) => {
        log::error!("查看全部订单请求提交失败：{}",e);
    }
}

}


pub fn create_client(user_agent: String) -> Client {
    let mut headers = header::HeaderMap::new();
    
    log::info!("客户端 User-Agent: {}", user_agent);
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(&user_agent).unwrap_or_else(|_| {
            header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        })
    );
    
    Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()
        .unwrap_or_default()
}

fn generate_random_string(length: usize) -> String {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;
    
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(|c| c as char)
        .collect()
}

pub struct AccountSwitch {
    pub uid: String,
    pub switch: bool,
}