use common::cookie_manager::CookieManager;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;

use crate::api::*;
use crate::show_orderlist::get_orderlist;
use common::captcha::handle_risk_verification;
use common::login::{send_loginsms, sms_login};
use common::taskmanager::*;
use common::ticket::ConfirmTicketResult;
use common::ticket::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct TaskManagerImpl {
    task_sender: mpsc::Sender<TaskMessage>,
    result_receiver: mpsc::Receiver<TaskResult>,
    running_tasks: HashMap<String, Task>, // 使用 Task 枚举
    runtime: Arc<Runtime>,
    _worker_thread: Option<thread::JoinHandle<()>>,
}

enum TaskMessage {
    SubmitTask(TaskRequest),
    CancelTask(String),
    Shutdown,
}

impl TaskManager for TaskManagerImpl {
    fn new() -> Self {
        // 创建通道
        let (task_tx, mut task_rx) = mpsc::channel(100);
        let (result_tx, result_rx) = mpsc::channel(100);

        // 创建tokio运行时
        let runtime = Arc::new(Runtime::new().unwrap());
        let rt = runtime.clone();

        // 启动工作线程
        let worker = thread::spawn(move || {
            rt.block_on(async {
                while let Some(msg) = task_rx.recv().await {
                    match msg {
                        TaskMessage::SubmitTask(request) => {
                            let result_tx_clone = result_tx.clone();
                            match request {
                                TaskRequest::QrCodeLoginRequest(req) => {
                                    handle_qrcode_login(req, result_tx_clone).await;
                                }
                                TaskRequest::LoginSmsRequest(req) => {
                                    handle_sms_send(req, result_tx_clone).await;
                                }
                                TaskRequest::PushRequest(req) => {
                                    handle_push(req, result_tx_clone).await;
                                }
                                TaskRequest::SubmitLoginSmsRequest(req) => {
                                    handle_sms_submit(req, result_tx_clone).await;
                                }
                                TaskRequest::GetAllorderRequest(req) => {
                                    handle_get_all_orders(req, result_tx_clone).await;
                                }
                                TaskRequest::GetTicketInfoRequest(req) => {
                                    handle_get_ticket_info(req, result_tx_clone).await;
                                }
                                TaskRequest::GetBuyerInfoRequest(req) => {
                                    handle_get_buyer_info(req, result_tx_clone).await;
                                }
                                TaskRequest::GrabTicketRequest(req) => {
                                    handle_grab_ticket(req, result_tx_clone).await;
                                }
                            }
                        }
                        TaskMessage::CancelTask(task_id) => {
                            handle_cancel_task(task_id).await;
                        }
                        TaskMessage::Shutdown => break,
                    }
                }
            });
        });

        Self {
            task_sender: task_tx,
            result_receiver: result_rx,
            running_tasks: HashMap::new(),
            runtime,
            _worker_thread: Some(worker),
        }
    }

    fn submit_task(&mut self, request: TaskRequest) -> Result<String, String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = create_task_from_request(&request, task_id.clone());
        
        self.running_tasks.insert(task_id.clone(), task);

        if let Err(e) = self
            .task_sender
            .blocking_send(TaskMessage::SubmitTask(request))
        {
            self.running_tasks.remove(&task_id);
            return Err(format!("无法提交任务: {}", e));
        }

        Ok(task_id)
    }

    fn get_results(&mut self) -> Vec<TaskResult> {
        let mut results = Vec::new();
        
        // 非阻塞方式获取所有可用结果
        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }
        
        results
    }

    fn cancel_task(&mut self, task_id: &str) -> Result<(), String> {
        if !self.running_tasks.contains_key(task_id) {
            return Err("任务不存在".to_string());
        }
        
        if let Err(e) = self.task_sender.blocking_send(TaskMessage::CancelTask(task_id.to_owned())) {
            return Err(format!("无法取消任务: {}", e));
        }
        
        Ok(())
    }

    fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        if let Some(task) = self.running_tasks.get(task_id) {
            match task {
                Task::QrCodeLoginTask(t) => Some(t.status.clone()),
                Task::LoginSmsRequestTask(t) => Some(t.status.clone()),
                Task::PushTask(t) => Some(t.status.clone()),
                Task::SubmitLoginSmsRequestTask(t) => Some(t.status.clone()),
                Task::GetAllorderRequestTask(t) => Some(t.status.clone()),
                Task::GetTicketInfoTask(t) => Some(t.status.clone()),
                Task::GetBuyerInfoTask(t) => Some(t.status.clone()),
                Task::GrabTicketTask(t) => Some(t.status.clone()),
            }
        } else {
            None
        }
    }

    fn shutdown(&mut self) {
        let _ = self.task_sender.blocking_send(TaskMessage::Shutdown);
        if let Some(handle) = self._worker_thread.take() {
            let _ = handle.join();
        }
    }
}

// ============== 任务处理函数 ==============

async fn handle_qrcode_login(req: QrCodeLoginRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = uuid::Uuid::new_v4().to_string();
    tokio::spawn(async move {
        log::info!("处理二维码登录任务 ID: {}", task_id);
        
        let status = poll_qrcode_login(&req.qrcode_key, req.user_agent.as_deref()).await;
        let (cookie, error) = match &status {
            common::login::QrCodeLoginStatus::Success(cookie) => (Some(cookie.clone()), None),
            common::login::QrCodeLoginStatus::Failed(err) => (None, Some(err.clone())),
            _ => (None, None)
        };
        
        let task_result = TaskResult::QrCodeLoginResult(TaskQrCodeLoginResult {
            task_id,
            status,
            cookie,
            error,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_sms_send(req: LoginSmsRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = uuid::Uuid::new_v4().to_string();
    tokio::spawn(async move {
        log::info!("处理短信发送任务 ID: {}", task_id);
        
        let response = send_loginsms(
            &req.phone,
            &req.client,
            req.custom_config,
            req.local_captcha,
        ).await;
        
        let success = response.is_ok();
        let message = match &response {
            Ok(msg) => msg.clone(),
            Err(err) => {
                log::error!("发送短信验证码失败: {}", err);
                err.to_string()
            },
        };
        
        let task_result = TaskResult::LoginSmsResult(LoginSmsRequestResult {
            task_id,
            phone: req.phone,
            success,
            message,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_push(req: PushRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = uuid::Uuid::new_v4().to_string();
    tokio::spawn(async move {
        log::info!("处理推送任务 ID: {}, 类型: {:?}", task_id, req.push_type);
        
        let (success, result_message) = match req.push_type {
            PushType::All => {
                req.push_config.push_all_async(&req.title, &req.message, &req.jump_url).await
            },
            _ => (false, "未实现的推送类型".to_string())
        };
        
        let task_result = TaskResult::PushResult(PushRequestResult {
            task_id,
            success,
            message: result_message,
            push_type: req.push_type,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_sms_submit(req: SubmitLoginSmsRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = uuid::Uuid::new_v4().to_string();
    tokio::spawn(async move {
        log::info!("处理短信提交任务 ID: {}", task_id);
        
        let response = sms_login(&req.phone, &req.code, &req.captcha_key, &req.client).await;
        let success = response.is_ok();
        let message = match &response {
            Ok(msg) => msg.clone(),
            Err(err) => {
                log::error!("提交短信验证码失败: {}", err);
                err.to_string()
            },
        };
        let cookie = match &response {
            Ok(msg) => Some(msg.clone()),
            Err(_) => None,
        };
        
        let task_result = TaskResult::SubmitSmsLoginResult(SubmitSmsLoginResult {
            task_id,
            phone: req.phone,
            success,
            message,
            cookie,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_get_all_orders(req: GetAllorderRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    tokio::spawn(async move {
        log::info!("处理获取订单任务 ID: {}", task_id);
        
        let response = get_orderlist(req.cookie_manager).await;
        let success = response.is_ok();
        let data = response.as_ref().ok();
        let message = match &response {
            Ok(msg) => format!("获取全部订单成功: {}", msg.data.total),
            Err(err) => err.to_string(),
        };

        let task_result = TaskResult::GetAllorderRequestResult(GetAllorderRequestResult {
            task_id: task_id.clone(),
            success,
            message,
            order_info: data.cloned(),
            account_id: req.account_id,
            timestamp: std::time::Instant::now(),
        });

        send_result(result_tx, task_result).await;
    });
}

async fn handle_get_ticket_info(req: GetTicketInfoRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    tokio::spawn(async move {
        log::debug!("处理获取票务信息任务 ID: {}", task_id);
        
        let response = get_project(req.cookie_manager, &req.project_id).await;
        let success = response.is_ok();
        let ticket_info = response.as_ref().ok().cloned();
        let message = match &response {
            Ok(info) => format!("项目{}请求成功", info.errno),
            Err(e) => e.to_string(),
        };

        let task_result = TaskResult::GetTicketInfoResult(GetTicketInfoResult {
            task_id: task_id.clone(),
            uid: req.uid,
            ticket_info,
            success,
            message,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_get_buyer_info(req: GetBuyerInfoRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    tokio::spawn(async move {
        log::debug!("处理获取购票人信息任务 ID: {}", task_id);
        
        let response = get_buyer_info(req.cookie_manager).await;
        let success = response.is_ok();
        let message = match &response {
            Ok(_) => "购票人信息请求成功".to_string(),
            Err(e) => e.to_string(),
        };
        let buyer_info = response.ok();

        let task_result = TaskResult::GetBuyerInfoResult(GetBuyerInfoResult {
            task_id: task_id.clone(),
            uid: req.uid,
            buyer_info,
            success,
            message,
        });
        
        send_result(result_tx, task_result).await;
    });
}

async fn handle_grab_ticket(req: GrabTicketRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    tokio::spawn(async move {
        log::info!("处理抢票任务 ID: {}, 模式: {}", task_id, req.grab_mode);
        
        match req.grab_mode {
            0 => timing_mode_grab(req, result_tx).await,
            1 => direct_mode_grab(req, result_tx).await,
            2 => pickup_mode_grab(req, result_tx).await,
            _ => {
                let error_msg = format!("未知抢票模式: {}", req.grab_mode);
                log::error!("{}", error_msg);
                send_grab_result(
                    &result_tx,
                    &task_id,
                    req.uid.try_into().unwrap(),
                    false,
                    &error_msg,
                    None,
                    None,
                    None,
                    None
                ).await;
            }
        }
    });
}

async fn handle_cancel_task(task_id: String) {
    log::info!("取消任务: {}", task_id);
    // 实现具体的取消逻辑
}

// ============== 抢票模式实现 ==============

async fn timing_mode_grab(req: GrabTicketRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    let project_info = req.biliticket.project_info.clone();
    
    match get_countdown(req.cookie_manager.clone(), project_info).await {
        Ok(countdown) => {
            if countdown > 0.0 {
                log::info!("距离抢票时间还有{}秒", countdown);
                await_countdown(countdown as f32).await;
            }
        }
        Err(e) => {
            send_grab_error(&result_tx, &task_id, req.uid.try_into().unwrap(), &format!("获取倒计时失败: {}", e)).await;
            return;
        }
    }
    
    log::info!("开始抢票！");
    grab_ticket_core(req, result_tx).await;
}

async fn direct_mode_grab(req: GrabTicketRequest, result_tx: mpsc::Sender<TaskResult>) {
    log::info!("开始抢票！");
    grab_ticket_core(req, result_tx).await;
}

async fn pickup_mode_grab(mut req: GrabTicketRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    
    'main_loop: loop {
        // 获取项目数据
        let project_data = match get_project(req.cookie_manager.clone(), req.project_id.clone().as_str()).await {
            Ok(data) => data,
            Err(e) => {
                log::error!("获取项目数据失败: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        
        // 检查项目是否可售
        if ![8, 2].contains(&project_data.data.sale_flag_number) {
            send_grab_error(&result_tx, &task_id, req.uid.try_into().unwrap(), "当前项目已停售").await;
            break 'main_loop;
        }
        
        if ![1, 2].contains(&project_data.data.id_bind) {
            send_grab_error(&result_tx, &task_id, req.uid.try_into().unwrap(), "暂不支持抢非实名票捡漏模式").await;
            break 'main_loop;
        }
        
        req.biliticket.id_bind = project_data.data.id_bind.clone() as usize;
        
        'screen_loop: for screen_data in project_data.data.screen_list {
            if !screen_data.clickable {
                continue;
            }
            
            req.screen_id = screen_data.id.clone().to_string();
            req.biliticket.screen_id = screen_data.id.clone().to_string();
            
            for ticket_data in screen_data.ticket_list {
                if !ticket_data.clickable {
                    continue;
                }
                
                if should_skip_ticket(&ticket_data, &req.skip_words) {
                    continue;
                }
                
                req.ticket_id = ticket_data.id.clone().to_string();
                req.biliticket.select_ticket_id = Some(ticket_data.id.clone().to_string());
                
                match get_ticket_token(
                    req.cookie_manager.clone(),
                    &req.project_id,
                    &req.screen_id,
                    &req.ticket_id,
                    req.count
                ).await {
                    Ok(token) => {
                        if handle_ticket_grab(&req, &token, &result_tx).await {
                            break 'main_loop; // 抢票成功，退出捡漏模式
                        }
                    }
                    Err(risk_param) => {
                        if !handle_risk(&req, &risk_param, &result_tx).await {
                            break 'screen_loop; // 遇到无法处理的错误
                        }
                    }
                }
            }
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// ============== 抢票核心逻辑 ==============

async fn grab_ticket_core(req: GrabTicketRequest, result_tx: mpsc::Sender<TaskResult>) {
    let task_id = req.task_id.clone();
    let mut token_retry_count = 0;
    const MAX_TOKEN_RETRY: i8 = 5;
    
    loop {
        match get_ticket_token(
            req.cookie_manager.clone(),
            &req.project_id,
            &req.screen_id,
            &req.ticket_id,
            req.count
        ).await {
            Ok(token) => {
                if handle_ticket_grab(&req, &token, &result_tx).await {
                    break; // 抢票流程结束
                }
            }
            Err(risk_param) => {
                if !handle_risk(&req, &risk_param, &result_tx).await {
                    token_retry_count += 1;
                    if token_retry_count >= MAX_TOKEN_RETRY {
                        send_grab_error(
                            &result_tx,
                            &task_id,
                            req.uid.try_into().unwrap(),
                            "获取token失败，达到最大重试次数"
                        ).await;
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_ticket_grab(
    req: &GrabTicketRequest,
    token: &str,
    result_tx: &mpsc::Sender<TaskResult>
) -> bool {
    let task_id = req.task_id.clone();
    let mut confirm_retry_count = 0;
    const MAX_CONFIRM_RETRY: i8 = 4;
    
    loop {
        let (success, should_break) = process_grab_ticket(
            req.cookie_manager.clone(),
            &req.project_id,
            token,
            &task_id,
            req.uid,
            result_tx,
            req,
            &req.buyer_info
        ).await;
        
        if success || should_break {
            return true;
        }
        
        confirm_retry_count += 1;
        if confirm_retry_count >= MAX_CONFIRM_RETRY {
            send_grab_error(
                result_tx,
                &task_id,
                req.uid.try_into().unwrap(),
                "确认订单失败，达到最大重试次数"
            ).await;
            return true;
        }
        
        tokio::time::sleep(Duration::from_secs_f32(0.3)).await;
    }
}

async fn handle_risk(
    req: &GrabTicketRequest,
    risk_param: &TokenRiskParam,
    result_tx: &mpsc::Sender<TaskResult>
) -> bool {
    let task_id = req.task_id.clone();
    
    if risk_param.code == -401 || risk_param.code == 401 {
        log::warn!("需要验证码，开始处理...");
        match handle_risk_verification(
            req.cookie_manager.clone(),
            risk_param.clone(),
            &req.biliticket.config,
            &req.biliticket.account.csrf,
            req.local_captcha.clone(),
        ).await {
            Ok(()) => {
                log::info!("验证码处理成功！");
                return true;
            }
            Err(e) => {
                log::error!("验证码处理失败: {}", e);
                return false;
            }
        }
    } else {
        handle_critical_error(&risk_param, result_tx, &task_id, req.uid.try_into().unwrap()).await;
        return false;
    }
}

// ============== 辅助函数 ==============

async fn send_result(tx: mpsc::Sender<TaskResult>, result: TaskResult) {
    if let Err(e) = tx.send(result).await {
        log::error!("发送任务结果失败: {}", e);
    }
}

async fn send_grab_result(
    tx: &mpsc::Sender<TaskResult>,
    task_id: &str,
    uid: u64,
    success: bool,
    message: &str,
    order_id: Option<String>,
    pay_token: Option<String>,
    pay_result: Option<CheckFakeResultData>,
    confirm_result: Option<ConfirmTicketResult>,
) {
    let result = TaskResult::GrabTicketResult(GrabTicketResult {
        task_id: task_id.to_string(),
        uid: uid.try_into().unwrap(),
        success,
        message: message.to_string(),
        order_id,
        pay_token,
        pay_result,
        confirm_result,
    });
    
    send_result(tx.clone(), result).await;
}

async fn send_grab_error(
    tx: &mpsc::Sender<TaskResult>,
    task_id: &str,
    uid: u64,
    message: &str
) {
    send_grab_result(tx, task_id, uid, false, message, None, None, None, None).await;
}

async fn handle_critical_error(
    risk_param: &TokenRiskParam,
    tx: &mpsc::Sender<TaskResult>,
    task_id: &str,
    uid: u64
) {
    let message = match risk_param.code {
        100080 | 100082 => "场次/项目/日期选择有误",
        100039 => "该场次已停售",
        _ => "未知错误"
    };
    
    let full_message = format!("错误代码: {} - {}", risk_param.code, message);
    send_grab_error(tx, task_id, uid, &full_message).await;
}

fn create_task_from_request(request: &TaskRequest, task_id: String) -> Task {
    match request {
        TaskRequest::QrCodeLoginRequest(qrcode_req) => {
            log::info!("创建二维码登录任务 ID: {}", task_id);
            Task::QrCodeLoginTask(QrCodeLoginTask {
                task_id: task_id.clone(),
                qrcode_key: qrcode_req.qrcode_key.clone(),
                qrcode_url: qrcode_req.qrcode_url.clone(),
                status: TaskStatus::Pending,
                start_time: Some(Instant::now()),
            })
        }
        TaskRequest::LoginSmsRequest(login_sms_req) => {
            log::info!("创建短信验证码任务 ID: {}, 手机号: {}", task_id, login_sms_req.phone);
            Task::LoginSmsRequestTask(LoginSmsRequestTask {
                task_id: task_id.clone(),
                phone: login_sms_req.phone.clone(),
                status: TaskStatus::Pending,
                start_time: Some(Instant::now()),
            })
        }
        TaskRequest::PushRequest(push_req) => {
            log::info!("创建推送任务 ID: {}", task_id);
            Task::PushTask(PushTask {
                task_id: task_id.clone(),
                push_type: push_req.push_type.clone(),  // 使用push_type
                title: push_req.title.clone(),
                message: push_req.message.clone(),
                status: TaskStatus::Pending,
                start_time: Some(std::time::Instant::now()),
            })
        }
        TaskRequest::SubmitLoginSmsRequest(submit_sms_req) => {
            log::info!("创建提交短信验证码任务 ID: {}, 手机号: {}", task_id, submit_sms_req.phone);
            Task::SubmitLoginSmsRequestTask(SubmitLoginSmsRequestTask {
                task_id: task_id.clone(),
                phone: submit_sms_req.phone.clone(),
                code: submit_sms_req.code.clone(),
                captcha_key: submit_sms_req.captcha_key.clone(),
                status: TaskStatus::Pending,
                start_time: Some(Instant::now()),
            })
        }
        TaskRequest::GetAllorderRequest(get_all_order_req) => {
            log::info!("创建获取全部订单任务 ID: {}", task_id);
            Task::GetAllorderRequestTask(GetAllorderRequest {
                task_id: task_id.clone(),
                cookie_manager: get_all_order_req.cookie_manager.clone(),
                status: TaskStatus::Pending,
                cookies: get_all_order_req.cookies.clone(),
                account_id: get_all_order_req.account_id.clone(),
                start_time: Some(std::time::Instant::now()),
            })
        }
        TaskRequest::GetTicketInfoRequest(get_ticket_info_req) => {
            log::info!("创建获取票务信息任务 ID: {}", task_id);
            Task::GetTicketInfoTask(GetTicketInfoTask {
                task_id: task_id.clone(),
                project_id: get_ticket_info_req.project_id.clone(),
                status: TaskStatus::Running,
                start_time: Some(Instant::now()),
                cookie_manager: get_ticket_info_req.cookie_manager.clone(),
            })
        }
        TaskRequest::GetBuyerInfoRequest(get_buyer_info_req) => {
            log::info!("创建获取购票人信息任务 ID: {}", task_id);
            Task::GetBuyerInfoTask(GetBuyerInfoTask {
                uid: get_buyer_info_req.uid.clone(),
                task_id: task_id.clone(),
                cookie_manager: get_buyer_info_req.cookie_manager.clone(),
                status: TaskStatus::Pending,
                start_time: Some(Instant::now()),
            })
        }
        TaskRequest::GrabTicketRequest(grab_ticket_req) => {
            log::info!("创建抢票任务 ID: {}, 模式: {}", task_id, grab_ticket_req.grab_mode);
            Task::GrabTicketTask(GrabTicketTask {
                task_id: task_id.clone(),
                biliticket: grab_ticket_req.biliticket.clone(),
                status: TaskStatus::Pending,
                client: reqwest::Client::new().into(),
                start_time: Some(Instant::now()),
            })
        }
    }
}

async fn await_countdown(mut countdown: f32) {
    if countdown > 20.0 {
        loop {
            if countdown <= 20.0 {
                break;
            }
            countdown -= 15.0;
            tokio::time::sleep(Duration::from_secs(15)).await;
            log::info!("距离抢票时间还有{}秒", countdown);
        }
    }
    
    loop {
        if countdown <= 1.3 {
            tokio::time::sleep(Duration::from_secs_f32(0.8)).await;
            break;
        }
        log::info!("距离抢票时间还有{}秒", countdown);
        countdown -= 1.0;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn should_skip_ticket(ticket_data: &ScreenTicketInfo, skip_words: &Option<Vec<String>>) -> bool {
    if let Some(skip_words) = skip_words {
        let title = ticket_data.screen_name.to_lowercase();
        let ticket_title = ticket_data.desc.to_lowercase();
        
        if skip_words.iter().any(|word| title.contains(&word.to_lowercase())) {
            log::info!("跳过包含过滤关键词的场次: {}", ticket_data.screen_name);
            return true;
        }
        
        if skip_words.iter().any(|word| ticket_title.contains(&word.to_lowercase())) {
            log::info!("跳过包含过滤关键词的票种: {}", ticket_data.screen_name);
            return true;
        }
    }
    false
}

async fn process_grab_ticket(
    cookie_manager: Arc<CookieManager>,
    project_id: &str,
    token: &str,
    task_id: &str,
    uid: i64,
    result_tx: &mpsc::Sender<TaskResult>,
    grab_ticket_req: &GrabTicketRequest,
    buyer_info: &Vec<BuyerInfo>,
) -> (bool, bool) {
    // 确认订单
    match confirm_ticket_order(cookie_manager.clone(), project_id, token).await {
        Ok(confirm_result) => {
            log::info!("确认订单成功！准备下单");

            if let Some((success, retry_limit)) = try_create_order(
                cookie_manager.clone(),
                project_id,
                token,
                &confirm_result,
                grab_ticket_req,
                buyer_info,
                task_id,
                uid,
                result_tx,
            )
            .await
            {
                return (success, retry_limit);
            }

            (true, false) // 订单流程已完成
        }
        Err(e) => {
            log::error!("确认订单失败，原因：{}  正在重试...", e);
            (false, false) // 需要继续重试
        }
    }
}

// 处理创建订单逻辑
async fn try_create_order(
    cookie_manager: Arc<CookieManager>,
    project_id: &str,
    token: &str,
    confirm_result: &ConfirmTicketResult,
    grab_ticket_req: &GrabTicketRequest,
    buyer_info: &Vec<BuyerInfo>,
    task_id: &str,
    uid: i64,
    result_tx: &mpsc::Sender<TaskResult>,
) -> Option<(
    bool,
    bool, // 第二个参数标记是因为达到重试上限
)> {
    let mut order_retry_count = 0;
    let mut need_retry = false;

    // 下单循环
    loop {
        if order_retry_count >= 3 {
            need_retry = true;
        }

        match create_order(
            cookie_manager.clone(),
            project_id,
            token,
            confirm_result,
            &grab_ticket_req.biliticket,
            buyer_info,
            true,
            need_retry,
            false,
            None,
        )
        .await
        {
            Ok(order_result) => {
                log::info!("下单成功！订单信息{:?}", order_result);
                let empty_json = json!({});
                let order_data = order_result.get("data").unwrap_or(&empty_json);

                let zero_json = json!(0);
                let order_id = order_data
                    .get("orderId")
                    .unwrap_or(&zero_json)
                    .as_i64()
                    .unwrap_or(0);

                let empty_string_json = json!("");
                let pay_token = order_data
                    .get("token")
                    .unwrap_or(&empty_string_json)
                    .as_str()
                    .unwrap_or("");

                log::info!("下单成功！正在检测是否假票！");
                // 检测假票
                let check_result = match check_fake_ticket(
                    cookie_manager.clone(),
                    project_id,
                    pay_token,
                    order_id,
                )
                .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        log::error!("检测假票失败，原因：{}，请前往订单列表查看是否下单成功", e);
                        continue; // 继续重试
                    }
                };
                let errno = check_result
                    .get("errno")
                    .unwrap_or(&zero_json)
                    .as_i64()
                    .unwrap_or(0);
                if errno != 0 {
                    log::error!("假票，继续抢票");
                    continue;
                }
                let analyze_result =
                    match serde_json::from_value::<CheckFakeResult>(check_result.clone()) {
                        Ok(result) => result,
                        Err(e) => {
                            log::error!("解析假票结果失败，原因：{}", e);
                            continue; // 继续重试
                        }
                    };

                let pay_result = analyze_result.data.pay_param;
                // 通知成功
                let task_result = TaskResult::GrabTicketResult(GrabTicketResult {
                    task_id: task_id.to_string(),
                    uid,
                    success: true,
                    message: "抢票成功".to_string(),
                    order_id: Some(order_id.to_string()),
                    pay_token: Some(pay_token.to_string()),
                    confirm_result: Some(confirm_result.clone()),
                    pay_result: Some(pay_result),
                });
                let _ = result_tx.send(task_result).await;

                return Some((true, false)); // 成功，不需要继续重试
            }

            Err(e) => {
                // 处理错误情况
                match e {
                    //需要继续重试的临时错误
                    100001 | 429 | 900001 => log::info!("b站限速，正常现象"),
                    100009 => {
                        log::info!("当前票种库存不足");
                        //再次降速，不给b站服务器带来压力
                        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.6)).await;
                    }
                    211 => {
                        log::info!("很遗憾，差一点点抢到票，继续加油吧！");
                    }

                    //需要暂停的情况
                    3 => {
                        log::info!("抢票速度过快，即将被硬控5秒");
                        log::info!("暂停4.8秒");
                        tokio::time::sleep(tokio::time::Duration::from_secs_f32(4.8)).await;
                    }

                    //需要重新获取token的情况
                    100041 | 100050 => {
                        log::info!("token失效，即将重新获取token");
                        return Some((true, false)); // 需要重新获取token
                    }

                    //需要终止抢票的致命错误
                    100017 | 100016 => {
                        log::info!("当前项目/类型/场次已停售");
                        return Some((true, false));
                    }
                    1 => {
                        log::error!(
                            "超人 请慢一点，这是仅限1人抢票的项目，或抢票格式有误，请重新提交任务"
                        );
                        return Some((true, false));
                    }
                    83000004 => {
                        log::error!("没有配置购票人信息！请重新配置");
                        return Some((true, false));
                    }
                    100079 | 100003 => {
                        log::error!("购票人存在待付款订单，请前往支付或取消后重新下单");
                        return Some((true, false));
                    }
                    100039 => {
                        log::error!("活动收摊啦,下次要快点哦");
                        return Some((true, false));
                    }
                    919 => {
                        log::error!(
                            "该项目区分绑定非绑定项目错误，传入意外值，请尝试重新下单以及提出issue"
                        );
                        return Some((true, false));
                    }
                    209001 => {
                        log::error!("当前项目只能选择一个购票人！不支持多选，请重新提交任务");
                        return Some((true, false));
                    }
                    737 => {
                        log::error!(
                            "B站传了一个NUll回来，请看一下上一行的message提示信息，自行决定是否继续，如果取消请关闭重新打开该应用"
                        );
                    }

                    //未知错误
                    _ => log::error!("下单失败，未知错误码：{} 可以提出issue修复该问题", e),
                }
            }
        }

        // 增加重试计数并等待
        order_retry_count += 1;
        if grab_ticket_req.grab_mode == 2 && order_retry_count >= 30 {
            log::error!(
                "捡漏模式下单失败，已达最大重试次数，放弃该票种抢票，准备检测其他票种继续捡漏"
            );
            return Some((false, true)); // 捡漏模式下单失败，放弃该票种抢票
        }
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.4)).await;
        //降低速度，不带来b站服务器压力
    }
}
