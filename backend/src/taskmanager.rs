use std::collections::HashMap;
use std::sync::Arc;
use std::{result, thread};
use common::cookie_manager::CookieManager;
use serde_json::json;


use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use common::taskmanager::{*};
use common::captcha::handle_risk_verification;
use common::login::{send_loginsms,sms_login};
use common::ticket::ConfirmTicketResult;
use common::ticket::{*};
use crate::show_orderlist::get_orderlist;
use crate::api::{*};


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
                            let task_id = uuid::Uuid::new_v4().to_string();
                            let result_tx = result_tx.clone();
                            
                            // 根据任务类型处理
                            match request {
                                
                                TaskRequest::QrCodeLoginRequest(qrcode_req) => {
                                    tokio::spawn(async move {
                                        // 二维码登录逻辑
                                        let status = poll_qrcode_login(&qrcode_req.qrcode_key,qrcode_req.user_agent.as_deref()).await;
                                        
                                        let (cookie, error) = match &status {
                                            common::login::QrCodeLoginStatus::Success(cookie) => 
                                                (Some(cookie.clone()), None),
                                            common::login::QrCodeLoginStatus::Failed(err) => 
                                                (None, Some(err.clone())),
                                            _ => (None, None)
                                        };
                                        
                                        // 创建正确的结果类型
                                        let task_result = TaskResult::QrCodeLoginResult(TaskQrCodeLoginResult {
                                            task_id,
                                            status,
                                            cookie,
                                            error,
                                        });
                                        
                                        let _ = result_tx.send(task_result).await;
                                    });
                                }
                                TaskRequest::LoginSmsRequest(login_sms_req) => {
                                    let task_id = uuid::Uuid::new_v4().to_string();
                                    let phone = login_sms_req.phone.clone();
                                    let client = login_sms_req.client.clone();
                                    let custom_config = login_sms_req.custom_config.clone();
                                    let result_tx = result_tx.clone();
                                    let local_captcha = login_sms_req.local_captcha.clone();
                                    

                                    /* let client = match reqwest::Client::builder()
                                        .user_agent(user_agent.clone())
                                        .cookie_store(true)
                                        .build() {
                                            Ok(client) => client,
                                            Err(err) => {
                                               // 记录错误并发送错误结果
                                               log::error!("创建请求客户端失败 ID: {}, 错误: {}", task_id, err);
                
                                               let task_result = TaskResult::LoginSmsResult(LoginSmsRequestResult {
                                                    task_id,
                                                    phone,
                                                    success: false,
                                                    message: format!("创建客户端失败: {}", err),
                                                    });
                
                                               let _ = result_tx.send(task_result).await;
                                               return; 
                                               }
                                               }; */
                                    

                                    tokio::spawn(async move{
                                        log::info!("开始发送短信验证码 ID: {}", task_id);

                                       
                                            log::info!("开始发送短信验证码 ID: {}", task_id);
                                            let response = send_loginsms(
                                                &phone, 
                                                &client, 
                                                custom_config,
                                                local_captcha,
                                            ).await;
                                            log::info!("开始发送短信验证码 ID: {}", task_id);
                                            let success = response.is_ok();
                                            let message = match &response {
                                                    Ok(msg) => msg.clone(),
                                                    Err(err) => {
                                                        log::error!("发送短信验证码失败: {}", err);
                                                        err.to_string()
                                                    },
                                                };
                                            log::info!("发送短信任务完成 ID: {}, 结果: {}", 
                                                task_id, 
                                                if success { "成功" } else { "失败" }
                                            );

                                            let task_result = TaskResult::LoginSmsResult(LoginSmsRequestResult {
                                                task_id,
                                                phone,
                                                success,
                                                message,
                                            });

                                            let _ = result_tx.send(task_result).await;

                                        
                                        


                                    });
                                
                                }
                                TaskRequest::PushRequest(push_req) => {
                                    let task_id = uuid::Uuid::new_v4().to_string();
                                    let push_config = push_req.push_config.clone();
                                    let title = push_req.title.clone();
                                    let message = push_req.message.clone();
                                    let jump_url = push_req.jump_url.clone();
                                    let push_type = push_req.push_type.clone();
                                    let result_tx = result_tx.clone();
                                    
                                    // 启动异步任务处理推送
                                    tokio::spawn(async move {
                                        log::info!("开始处理推送任务 ID: {}, 类型: {:?}", task_id, push_type);
                                        
                                        let (success, result_message) = match push_type {
                                            PushType::All => {
                                                push_config.push_all_async( &title, &message,&jump_url).await
                                            },
                                            
                                            // 其他推送类型的处理...
                                            _ => (false, "未实现的推送类型".to_string())
                                        };
                                        
                                        // 创建任务结果
                                        let task_result = TaskResult::PushResult(PushRequestResult {
                                            task_id: task_id.clone(),
                                            success,
                                            message: result_message,
                                            push_type: push_type.clone(),
                                        });
                                        
                                        // 发送结果
                                        if let Err(e) = result_tx.send(task_result).await {
                                            log::error!("发送推送任务结果失败: {}", e);
                                        }
                                        
                                        log::info!("推送任务 ID: {} 完成, 结果: {}", task_id, 
                                                  if success { "成功" } else { "失败" });
                                    });
                                    
                                 
                                }
                                TaskRequest::SubmitLoginSmsRequest(login_sms_req) => {
                                    let task_id = uuid::Uuid::new_v4().to_string();
                                    let phone = login_sms_req.phone.clone();
                                    let client = login_sms_req.client.clone();
                                    let captcha_key = login_sms_req.captcha_key.clone();
                                    let code = login_sms_req.code.clone();
                                    let result_tx = result_tx.clone();

                                    tokio::spawn(async move{
                                        log::info!("短信验证码登录进行中 ID: {}", task_id);
                                        
                                        let result = async{
                                            let response = sms_login(&phone,  &code,&captcha_key, &client).await;
                                            let success = response.is_ok();
                                            let message: String = match &response {
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
                                            log::info!("提交短信任务完成 ID: {}, 结果: {}", 
                                                task_id, 
                                                if success { "成功" } else { "失败" }
                                            );

                                            let task_result = TaskResult::SubmitSmsLoginResult(SubmitSmsLoginResult {
                                                task_id,
                                                phone,
                                                success,
                                                message,
                                                cookie,
                                            });

                                            let _ = result_tx.send(task_result).await;

                                        }.await;
                                    });
                                }
                                TaskRequest::GetAllorderRequest(get_order_req) => {
                                    let cookie_manager = get_order_req.cookie_manager.clone();
                                    let result_tx = result_tx.clone();
                                    let task_id = get_order_req.task_id;
                                    let account_id = get_order_req.account_id.clone();
                                    let cookies = get_order_req.cookies.clone();
                                    tokio::spawn(async move{
                                        log::info!("正在获取全部订单 ID: {}", task_id);
                                        let response = get_orderlist(cookie_manager,cookies.as_str()).await;
                                        let success = response.is_ok();
                                        let data = match &response {
                                            Ok(order_resp) => {order_resp.clone()},
                                            Err(err) => {
                                                log::error!("获取全部订单失败: {}", err);
                                                return;
                                            }
                                        };
                                        let message = match &response {
                                            Ok(msg) => {format!("获取全部订单成功: {}", msg.data.total)},
                                            Err(err) => {
                                                log::error!("获取全部订单失败: {}", err);
                                                err.to_string()
                                            },
                                        };

                                        let task_result = TaskResult::GetAllorderRequestResult(GetAllorderRequestResult {
                                            task_id: task_id.clone(),
                                            success,
                                            message,
                                            order_info: Some(data.clone()),
                                            account_id: account_id.clone(),
                                            timestamp: std::time::Instant::now(),
                                        });

                                        let _ = result_tx.send(task_result).await;
                                    });
                                }
                                TaskRequest::GetTicketInfoRequest(get_ticketinfo_req) => {
                                    let cookie_manager = get_ticketinfo_req.cookie_manager.clone();
                                    let task_id = get_ticketinfo_req.task_id.clone();
                                    let result_tx = result_tx.clone();
                                    let project_id = get_ticketinfo_req.project_id.clone();
                                    let uid = get_ticketinfo_req.uid.clone();
                                    tokio::spawn(async move{
                                        log::debug!("正在获取project{}",task_id);
                                        let response  = get_project(cookie_manager, &project_id).await;
                                        let success = response.is_ok();
                                        let ticket_info = match &response{
                                            Ok(info) => {
                                                // 检查数据是否有效
                                                if info.data.screen_list.is_empty() {
                                                    log::warn!("项目信息获取成功但场次列表为空，可能是API格式变化");
                                                }
                                                Some(info.clone())
                                            },
                                            Err(e) => {
                                                log::error!("获取项目时失败，原因：{}",e);
                                                None
                                            }
                                        };
                                        let message = match &response{
                                            Ok(info) => {
                                                //log::debug!("项目{}请求成功",info.errno);
                                                format!("项目{}请求成功",info.errno)
                                            }
                                            Err(e) => {
                                                e.to_string()
                                            }
                                        };
                                        let task_result = TaskResult::GetTicketInfoResult(GetTicketInfoResult{
                                            task_id : task_id.clone(),
                                            uid: uid.clone(),
                                            ticket_info : ticket_info.clone(),
                                            success : success,
                                            message : message.clone(),

                                        });
                                        let _ = result_tx.send(task_result).await;
                                    });
                                }
                                TaskRequest::GetBuyerInfoRequest(get_buyerinfo_req)=>{
                                    let cookie_manager = get_buyerinfo_req.cookie_manager.clone();
                                    let task_id = get_buyerinfo_req.task_id.clone();
                                    let result_tx = result_tx.clone();
                                    let uid = get_buyerinfo_req.uid.clone();
                                    tokio::spawn(async move{
                                        log::debug!("正在获取购票人信息{}",task_id);
                                        let response  = get_buyer_info(cookie_manager).await;
                                        let success = response.is_ok();
                                        let buyer_info = match &response{
                                            Ok(info) => {Some(info.clone())},
                                            Err(e) => {
                                                log::error!("获取购票人信息失败，原因：{}",e);
                                                None
                                            }
                                        };
                                        let message = match &response{
                                            Ok(info) => {
                                                //log::debug!("项目{}请求成功",info.errno);
                                                format!("购票人信息请求成功")
                                            }
                                            Err(e) => {
                                                e.to_string()
                                            }
                                        };
                                        let task_result = TaskResult::GetBuyerInfoResult(GetBuyerInfoResult{
                                            task_id : task_id.clone(),
                                            uid: uid.clone(),
                                            buyer_info : buyer_info.clone(),
                                            success : success,
                                            message : message.clone(),

                                        });
                                        let _ = result_tx.send(task_result).await;
                                        
                                    });
                                }
                                TaskRequest::GrabTicketRequest(grab_ticket_req)=>{
                                    let project_id = grab_ticket_req.project_id.clone();
                                    let screen_id = grab_ticket_req.screen_id.clone();
                                    let ticket_id = grab_ticket_req.ticket_id.clone();
                                    let buyer_info = grab_ticket_req.buyer_info.clone();
                                    let cookie_manager = grab_ticket_req.cookie_manager.clone();
                                    let task_id = grab_ticket_req.task_id.clone();
                                    let result_tx = result_tx.clone();
                                    let uid = grab_ticket_req.uid.clone();
                                    let mode = grab_ticket_req.grab_mode.clone();
                                    let custon_config = grab_ticket_req.biliticket.config.clone();
                                    let csrf = grab_ticket_req.biliticket.account.csrf.clone();
                                    let local_captcha = grab_ticket_req.local_captcha.clone();     
                                    let count = grab_ticket_req.count.clone();                                                               
                                    let project_info = grab_ticket_req.biliticket.project_info.clone();                                                                    
                                    
                                    tokio::spawn(async move{
                                        log::debug!("开始分析抢票任务：{}",task_id);
                                       
                                        match mode {
                                            0 => {
                                                log::debug!("定时抢票模式");
                                                //log::debug!("开售时间：{}",project_info.clone().unwrap().sale_begin);
                                                let mut countdown = match get_countdown(cookie_manager.clone(),project_info).await{
                                                    Ok(countdown) => countdown,
                                                    Err(e) => {
                                                        log::error!("获取倒计时失败: {}", e);
                                                        return;
                                                    }
                                                };
                                                //log::debug!("获取倒计时成功：{}",countdown);
                                                if countdown > 0.0{
                                                    log::info!("距离抢票时间还有{}秒",countdown);
                                                    loop{
                                                        if countdown <= 20.0 {
                                                            break;
                                                        }
                                                        countdown = countdown - 15.0;
                                                        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                                                        log::info!("距离抢票时间还有{}秒",countdown);
                                                        
                                                    }
                                                    loop{
                                                        if countdown <= 1.3 {  //按道理来说countdown是1秒，为了保险多设置几秒
                                                            tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.8)).await;
                                                            break;
                                                        }
                                                        log::info!("距离抢票时间还有{}秒",countdown);
                                                        countdown = countdown - 1.0;
                                                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                                    }
                                                }
                                                log::info!("开始抢票！");
                                                let mut token_retry_count = 0;
                                                const MAX_TOKEN_RETRY: i8 = 5;
                                                let mut confirm_order_retry_count = 0;
                                                const MAX_CONFIRM_ORDER_RETRY: i8 = 4;
                                                let mut order_retry_count = 0;
                                                let mut need_retry = false;

                                                //抢票主循环
                                                loop{

                                                    let token_result = get_ticket_token(cookie_manager.clone(), &project_id, &screen_id, &ticket_id, count).await;
                                                    match token_result {
                                                        Ok(token) => {
                                                            //获取token成功！
                                                            log::info!("获取抢票token成功！:{}",token);
                                                            let mut confirm_retry_count = 0;
                                                            const MAX_CONFIRM_RETRY: i8 = 4;
        
                                                            //尝试下单
                                                            loop {
                                                               let (success, retry_limit) = handle_grab_ticket(
                                                                cookie_manager.clone(), 
                                                                  &project_id, 
                                                                  &token, 
                                                                  &task_id, 
                                                                  uid, 
                                                                  &result_tx,
                                                                  &grab_ticket_req,
                                                                  &buyer_info
                                                                ).await ;
                                                                if success {
                                                                    log::info!("抢票流程结束，退出定时抢票模式");
                                                                    break; //成功或致命错误，跳出循环
                                                                }
            
                                                            
                                                            confirm_retry_count += 1;
                                                            if confirm_retry_count >= MAX_CONFIRM_RETRY {
                                                                  log::error!("确认订单失败，已达最大重试次数");
                                                                  let task_result = TaskResult::GrabTicketResult(GrabTicketResult {
                                                                    task_id: task_id.clone(),
                                                                    uid,
                                                                    success: false,
                                                                    message: "确认订单失败，已达最大重试次数".to_string(),
                                                                    order_id: None,
                                                                    pay_token: None,
                                                                    pay_result: None,
                                                                    confirm_result: None,
                                                                    });
                                                                  let _ = result_tx.send(task_result).await;
                                                                    break;
                                                                }
                                                           }
        
                                                        break; // 跳出token获取循环
                                                        },
                                                        Err(risk_param) => {
                                                            //获取token失败！分析原因
                                                            if risk_param.code == -401 || risk_param.code == 401 {
                                                                //需要处理验证码
                                                                log::warn!("需要验证码，开始处理验证码...");
                                                                match handle_risk_verification(
                                                                    cookie_manager.clone(), 
                                                                    risk_param,
                                                                    &custon_config,
                                                                    &csrf,
                                                                    local_captcha.clone(),
                                                                ).await {
                                                                    Ok(()) => {
                                                                        //验证码处理成功，继续抢票
                                                                        log::info!("验证码处理成功！");
                                                                    }
                                                                    Err(e) => {
                                                                        //验证码失败
                                                                        log::error!("验证码处理失败: {}", e);
                                                                        token_retry_count +=1;
                                                                        if token_retry_count >= MAX_TOKEN_RETRY {
                                                                            let task_result = TaskResult::GrabTicketResult(GrabTicketResult{
                                                                                task_id: task_id.clone(),
                                                                                uid,
                                                                                success: false,
                                                                                message: format!("验证码处理失败，已达最大重试次数: {}", e),
                                                                                order_id: None,
                                                                                pay_token: None,
                                                                                pay_result: None,
                                                                                confirm_result: None,
                                                                            });
                                                                            let _ = result_tx.send(task_result).await;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            }else{
                                                             //人为导致无法重试的错误
                                                             match risk_param.code {
                                                                100080 | 100082 => {
                                                                    log::error!("抢票失败，场次/项目/日期选择有误，请重新提交任务");
                                                                }
                                                                100039 => {
                                                                    log::error!("抢票失败，该场次已停售，请重新提交任务");
                                                                }
                                                                _ => {
                                                                    log::error!("抢票失败，未知错误，请重新提交任务");
                                                                }
                                                             }
                                                             token_retry_count +=1;
                                                             if token_retry_count >= MAX_TOKEN_RETRY {
                                                                let task_result = TaskResult::GrabTicketResult(GrabTicketResult{
                                                                    task_id: task_id.clone(),
                                                                    uid,
                                                                    success: false,
                                                                    message: format!("获取token失败，错误代码: {}，错误信息：{}", risk_param.code, risk_param.message),
                                                                    order_id: None,
                                                                    pay_token: None,
                                                                    pay_result: None,
                                                                    confirm_result: None,
                                                                });
                                                                let _ = result_tx.send(task_result).await;
                                                                break;
                                                             }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                                        }
                                                }

                                                }

                                                
                                            } 
                                            1 => {
                                                log::debug!("直接抢票模式");
                                                let mut token_retry_count = 0;
                                                const MAX_TOKEN_RETRY: i8 = 5; 
                                                let mut confirm_order_retry_count = 0;
                                                const MAX_CONFIRM_ORDER_RETRY: i8 = 4;
                                                let mut order_retry_count = 0;
                                                let mut need_retry = false;
                                                
                                                
                                                //抢票主循环
                                                loop{

                                                    let token_result = get_ticket_token(cookie_manager.clone(), &project_id, &screen_id, &ticket_id, count).await;
                                                    match token_result {
                                                        Ok(token) => {
                                                            //获取token成功！
                                                            log::info!("获取抢票token成功！:{}",token);
                                                            let mut confirm_retry_count = 0;
                                                            const MAX_CONFIRM_RETRY: i8 = 4;
        
                                                            //尝试下单
                                                            loop {
                                                               let (success, retry_limit) = handle_grab_ticket(
                                                                cookie_manager.clone(), 
                                                                  &project_id, 
                                                                  &token, 
                                                                  &task_id, 
                                                                  uid, 
                                                                  &result_tx,
                                                                  &grab_ticket_req,
                                                                  &buyer_info
                                                                ).await; 
                                                                if success {
                                                                    log::info!("抢票流程结束，退出捡漏模式");
                                                                    
                                                                    break; //成功或致命错误，跳出循环
                                                                }
            
                                                            
                                                            confirm_retry_count += 1;
                                                            if confirm_retry_count >= MAX_CONFIRM_RETRY {
                                                                  log::error!("确认订单失败，已达最大重试次数");
                                                                  let task_result = TaskResult::GrabTicketResult(GrabTicketResult {
                                                                    task_id: task_id.clone(),
                                                                    uid,
                                                                    success: false,
                                                                    message: "确认订单失败，已达最大重试次数".to_string(),
                                                                    order_id: None,
                                                                    pay_token: None,
                                                                    pay_result: None,
                                                                    confirm_result: None,
                                                                    });
                                                                  let _ = result_tx.send(task_result).await;
                                                                    break;
                                                                }
                                                           }
        
                                                        break; // 跳出token获取循环
                                                        },
                                                        Err(risk_param) => {
                                                            //获取token失败！分析原因
                                                            if risk_param.code == -401 || risk_param.code == 401 {
                                                                //需要处理验证码
                                                                log::warn!("需要验证码，开始处理验证码...");
                                                                match handle_risk_verification(
                                                                    cookie_manager.clone(), 
                                                                    risk_param,
                                                                    &custon_config,
                                                                    &csrf,
                                                                    local_captcha.clone(),
                                                                ).await {
                                                                    Ok(()) => {
                                                                        //验证码处理成功，继续抢票
                                                                        log::info!("验证码处理成功！");
                                                                    }
                                                                    Err(e) => {
                                                                        //验证码失败
                                                                        log::error!("验证码处理失败: {}", e);
                                                                        token_retry_count +=1;
                                                                        if token_retry_count >= MAX_TOKEN_RETRY {
                                                                            let task_result = TaskResult::GrabTicketResult(GrabTicketResult{
                                                                                task_id: task_id.clone(),
                                                                                uid,
                                                                                success: false,
                                                                                message: format!("验证码处理失败，已达最大重试次数: {}", e),
                                                                                order_id: None,
                                                                                pay_token: None,
                                                                                pay_result: None,
                                                                                confirm_result: None,
                                                                            });
                                                                            let _ = result_tx.send(task_result).await;
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                            }else{
                                                             //人为导致无法重试的错误
                                                             match risk_param.code {
                                                                100080 | 100082 => {
                                                                    log::error!("抢票失败，场次/项目/日期选择有误，请重新提交任务");
                                                                }
                                                                100039 => {
                                                                    log::error!("抢票失败，该场次已停售，请重新提交任务");
                                                                }
                                                                _ => {
                                                                    log::error!("抢票失败，未知错误，请重新提交任务");
                                                                }
                                                             }
                                                             token_retry_count +=1;
                                                             if token_retry_count >= MAX_TOKEN_RETRY {
                                                                let task_result = TaskResult::GrabTicketResult(GrabTicketResult{
                                                                    task_id: task_id.clone(),
                                                                    uid,
                                                                    success: false,
                                                                    message: format!("获取token失败，错误代码: {}，错误信息：{}", risk_param.code, risk_param.message),
                                                                    order_id: None,
                                                                    pay_token: None,
                                                                    pay_result: None,
                                                                    confirm_result: None,
                                                                });
                                                                let _ = result_tx.send(task_result).await;
                                                                break;
                                                             }
                                                    }
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                                        }
                                                }

                                                }
                                            }
                                            2=> {
                                                log::debug!("捡漏模式");
                                                let mut local_grab_request = grab_ticket_req.clone();
                                                
                                                // 外层循环，一旦抢票成功或遇到致命错误就退出
                                                'main_loop: loop {
                                                    log::debug!("project_id: {}, screen_id: {}, ticket_id: {}", project_id, screen_id, ticket_id);
                                                    
                                                    // 获取项目数据
                                                    let project_data = match get_project(cookie_manager.clone(), project_id.clone().as_str()).await {
                                                        Ok(data) => data,
                                                        Err(e) => {
                                                            log::error!("获取项目数据失败，原因：{}", e);
                                                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                                                            continue;
                                                        }
                                                    };
                                                    
                                                    // 检查项目是否可售
                                                    if ![8,2].contains(&project_data.data.sale_flag_number){
                                                        log::error!("当前项目已停售，暂时不会放出回流票，请等等重新提交任务");
                                                        break 'main_loop; // 直接退出整个捡漏模式
                                                    }
                                                    
                                                    if ![1, 2].contains(&project_data.data.id_bind) {
                                                        log::error!("暂不支持抢非实名票捡漏模式");
                                                        break 'main_loop; 
                                                    } 
                                                    local_grab_request.biliticket.id_bind = project_data.data.id_bind.clone() as usize;
                                                    'screen_loop: for screen_data in project_data.data.screen_list {
                                                        if !screen_data.clickable {
                                                            continue; 
                                                        }
                                                        
                                                        local_grab_request.screen_id = screen_data.id.clone().to_string();
                                                        local_grab_request.biliticket.screen_id = screen_data.id.clone().to_string();
                                                        log::info!("当前项目有可抢票场次，开始抢票！");
                                                        
                                                        // 遍历票种
                                                        'ticket_loop: for ticket_data in screen_data.ticket_list {
                                                            if !ticket_data.clickable {
                                                                continue; // 跳过不可点击的票种
                                                            }
                                                            
                                                            log::info!("当前{} {}票种可售，开始抢票！", ticket_data.screen_name, ticket_data.desc);
                                                            local_grab_request.ticket_id = ticket_data.id.clone().to_string();
                                                            local_grab_request.biliticket.select_ticket_id = Some(ticket_data.id.clone().to_string());
                                                            
                                                            // 获取token
                                                            let token = get_ticket_tokne(
                                                                cookie_manager.clone(),
                                                                project_data.data.id as u32, 
                                                                screen_data.id as u32, 
                                                                ticket_data.id as u32,
                                                                count as u16, 1, None
                                                            ).await;
                                                            
                                                            log::debug!("获取token成功！:{}", token);
                                                            
                                                            // 尝试抢票
                                                            let mut confirm_retry_count = 0;
                                                            const MAX_CONFIRM_RETRY: i8 = 4;
                                                            
                                                            loop {
                                                                let (success, retry_limit) = handle_grab_ticket(
                                                                    cookie_manager.clone(),
                                                                    &project_id,
                                                                    &token,
                                                                    &task_id,
                                                                    uid,
                                                                    &result_tx,
                                                                    &local_grab_request,
                                                                    &buyer_info
                                                                ).await;
                                                                if success {
                                                                    log::info!("抢票流程结束，退出捡漏模式");
                                                                    
                                                                    break 'main_loop;
                                                                }
                                                                if retry_limit {
                                                                    log::info!("该票种已达到最大重试次数，恢复捡漏模式，尝试其他票种");
                                                                    break 'screen_loop;
                                                                }
                                                                
                                                                confirm_retry_count += 1;
                                                                if confirm_retry_count >= MAX_CONFIRM_RETRY {
                                                                    log::error!("确认订单失败，已达最大重试次数，尝试其他票种");
                                                                    break; // 只跳出当前票种的重试循环
                                                                }
                                                                
                                                                tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.3)).await;
                                                            }
                                                        }
                                                    }
                                                    
                                                    // 本轮所有场次和票种都检查完毕，休息一秒后继续下一轮
                                                    log::info!("所有场次和票种检查完毕，等待2秒后重新检查");
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                                }
                                                
                                                log::info!("捡漏模式任务已退出");
                                            }
                                            _=> {
                                                log::error!("未知模式");
                                            }
                                        }
                                    });
                                }
                            }
                        },
                        TaskMessage::CancelTask(_task_id) => {
                            // 取消任务逻辑
                        },
                        TaskMessage::Shutdown => break,
                    }
                }
            });
        });
        
        Self {
            task_sender: task_tx,
            result_receiver: result_rx,
            running_tasks: HashMap::new(),
            runtime: runtime,
            _worker_thread: Some(worker),
        }
    }
    
    fn submit_task(&mut self, request: TaskRequest) -> Result<String, String> {
        // 生成任务ID
        let task_id = uuid::Uuid::new_v4().to_string();
        
        // 根据请求类型创建相应的任务
        match &request {
            
            TaskRequest::QrCodeLoginRequest(qrcode_req) => {
                log::info!("提交二维码登录任务 ID: {}", task_id);
                // 创建二维码登录任务
                let task = QrCodeLoginTask {
                    task_id: task_id.clone(),
                    qrcode_key: qrcode_req.qrcode_key.clone(),
                    qrcode_url: qrcode_req.qrcode_url.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::QrCodeLoginTask(task));
            }
            TaskRequest::LoginSmsRequest(login_sms_req) => {
                log::info!("提交短信验证码任务 ID: {}, 手机号: {}", task_id, login_sms_req.phone);
                
                // 创建短信任务
                let task = LoginSmsRequestTask {
                    task_id: task_id.clone(),
                    phone: login_sms_req.phone.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::LoginSmsRequestTask(task));
            }
            TaskRequest::PushRequest(push_req) => {
                log::info!("提交推送任务 ID: {}", task_id);
                // 创建推送任务
                let task = PushTask {
                    task_id: task_id.clone(),
                    push_type: push_req.push_type.clone(),  // 使用push_type
                    title: push_req.title.clone(),
                    message: push_req.message.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::PushTask(task));
            }

            TaskRequest::SubmitLoginSmsRequest(login_sms_req) => {
                log::info!("提交短信验证码登录任务 ID: {}, 手机号: {}", task_id, login_sms_req.phone);
                
                // 创建短信验证码登录任务
                let task = SubmitLoginSmsRequestTask {
                    task_id: task_id.clone(),
                    phone: login_sms_req.phone.clone(),
                    code: login_sms_req.code.clone(),
                    captcha_key: login_sms_req.captcha_key.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::SubmitLoginSmsRequestTask(task));
            }
            TaskRequest::GetAllorderRequest(get_order_req) => {
                log::info!("提交获取全部订单任务 ID: {}", task_id);
                
                // 创建获取全部订单任务
                let task = GetAllorderRequest {
                    task_id: task_id.clone(),
                    cookie_manager: get_order_req.cookie_manager.clone(),
                    status: TaskStatus::Pending,
                    cookies: get_order_req.cookies.clone(),
                    account_id: get_order_req.account_id.clone(),
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::GetAllorderRequestTask(task));
            }
            TaskRequest::GetTicketInfoRequest(get_ticketinfo_req) => {
                log::info!("{}",task_id);
                let task = GetTicketInfoTask{
                    task_id : task_id.clone(),
                    project_id: get_ticketinfo_req.project_id.clone(),
                    status: TaskStatus::Running,
                    start_time: Some(std::time::Instant::now()),
                    cookie_manager: get_ticketinfo_req.cookie_manager.clone(), 
                };
                self.running_tasks.insert(task_id.clone(),Task::GetTicketInfoTask(task));
            }
            TaskRequest::GetBuyerInfoRequest(get_buyerinfo_req) => {
                log::info!("提交获取购票人信息任务 ID: {}", task_id);
                
                //创建任务
                let task = GetBuyerInfoTask {
                    uid: get_buyerinfo_req.uid.clone(),
                    task_id: task_id.clone(),
                    cookie_manager: get_buyerinfo_req.cookie_manager.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                    
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::GetBuyerInfoTask(task));
            }
            TaskRequest::GrabTicketRequest(grab_ticket_req) => {
                log::info!("提交抢票任务 ID: {}", task_id);
                
               /*  // 创建抢票任务
                let task = GrabTicketTask {
                    task_id: task_id.clone(),
                    project_id: grab_ticket_req.project_id.clone(),
                    screen_id: grab_ticket_req.screen_id.clone(),
                    ticket_id: grab_ticket_req.ticket_id.clone(),
                    buyer_info: grab_ticket_req.buyer_info.clone(),
                    client: grab_ticket_req.client.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                    uid: grab_ticket_req.uid.clone(),
                    grab_mode: grab_ticket_req.grab_mode.clone(),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::GrabTicketTask(task)); */
            }

        }
        
        // 发送任务
        if let Err(e) = self.task_sender.blocking_send(TaskMessage::SubmitTask(request)) {
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



async fn handle_grab_ticket(
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
            
            
            if let Some((success,retry_limit)) = try_create_order(
                cookie_manager.clone(),
                project_id,
                token,
                &confirm_result,
                grab_ticket_req,
                buyer_info,
                task_id,
                uid,
                result_tx,
            ).await {
                
                return (success,retry_limit);
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
    bool  // 第二个参数标记是因为达到重试上限
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
            None
        ).await {
            Ok(order_result) => {
                log::info!("下单成功！订单信息{:?}", order_result);
                let empty_json = json!({});
                let order_data = order_result.get("data").unwrap_or(&empty_json);
                
                let zero_json = json!(0);
                let order_id = order_data.get("orderId").unwrap_or(&zero_json).as_i64().unwrap_or(0);
                
                let empty_string_json = json!("");
                let pay_token = order_data.get("token").unwrap_or(&empty_string_json).as_str().unwrap_or("");
                
                log::info!("下单成功！正在检测是否假票！");
                // 检测假票
                let check_result = match check_fake_ticket(cookie_manager.clone(), project_id, pay_token, order_id).await{
                    Ok(result) => result,
                    Err(e) => {
                        log::error!("检测假票失败，原因：{}，请前往订单列表查看是否下单成功", e);
                        continue; // 继续重试
                    }
                };
                let errno = check_result.get("errno").unwrap_or(&zero_json).as_i64().unwrap_or(0);
                if errno != 0 {
                    log::error!("假票，继续抢票");
                    continue;
                }
                let analyze_result = match serde_json::from_value::<CheckFakeResult>(check_result.clone()){
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
                    pay_result : Some(pay_result),

                });
                let _ = result_tx.send(task_result).await;
                
                return Some((true,false)); // 成功，不需要继续重试
            }
            
            Err(e) => {
                // 处理错误情况
                match e {
                    //需要继续重试的临时错误
                    100001 => log::info!("b站限速，正常现象"),
                    100009 => log::info!("当前票种库存不足"),
                    
                    //需要暂停的情况
                    3 => {
                        log::info!("抢票速度过快，即将被硬控5秒");
                        log::info!("暂停4.8秒");
                        tokio::time::sleep(tokio::time::Duration::from_secs_f32(4.8)).await;
                    },
                    
                    //需要重新获取token的情况
                    100041 => {
                        log::info!("token失效，即将重新获取token");
                        return Some((true,false)); // 需要重新获取token
                    },
                    
                    //需要终止抢票的致命错误
                    100017 | 100016 => {
                        log::info!("当前项目/类型/场次已停售");
                        return Some((true,false));
                    },
                    83000004 => {
                        log::error!("没有配置购票人信息！请重新配置");
                        return Some((true,false));
                    },
                    100079 | 100003  => {
                        log::error!("购票人存在待付款订单，请前往支付或取消后重新下单");
                        return Some((true,false));
                    },
                    100039 => {
                        log::error!("活动收摊啦,下次要快点哦");
                        return Some((true,false));
                    }
                    919 => {
                        log::error!("该项目区分绑定非绑定项目错误，传入意外值，请尝试重新下单以及提出issue");
                        return Some((true,false));
                    }
                    209001 => {
                        log::error!("当前项目只能选择一个购票人！不支持多选，请重新提交任务");
                        return Some((true,false));
                    }
                    737 => {
                        log::error!("B站传了一个NUll回来，请看一下上一行的message提示信息，自行决定是否继续，如果取消请关闭重新打开该应用");
                    }

                    //未知错误
                    _ => log::error!("下单失败，未知错误码：{} 可以提出issue修复该问题", e),
                }
            }
        }
        
        // 增加重试计数并等待
        order_retry_count += 1;
        if grab_ticket_req.grab_mode == 2 && order_retry_count >= 30 {
            log::error!("捡漏模式下单失败，已达最大重试次数，放弃该票种抢票，准备检测其他票种继续捡漏");
            return Some((false,true)); // 捡漏模式下单失败，放弃该票种抢票
        }
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.22)).await;
    }
}