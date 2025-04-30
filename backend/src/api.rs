use common::cookie_manager::CookieManager;
use common::http_utils::request_get;
use common::ticket::{*};
use reqwest::cookie::Cookie;
use serde_json;
use common::login::QrCodeLoginStatus;
use reqwest::Client;
use std::sync::Arc;
use serde_json::{json, Value};
use rand::{thread_rng, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_buyer_info(client: Arc<Client>) -> Result<BuyerInfoResponse,String>{
    let req = client.get("https://show.bilibili.com/api/ticket/buyer/list");
    let response = req.send().await;
    match response {
        Ok(resp)=>{
            if resp.status().is_success(){
                match tokio::task::block_in_place(||{
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(resp.text())
                }){
                    Ok(text) => {
                        log::debug!("获取购票人信息：{}",text);
                        match serde_json::from_str::<BuyerInfoResponse>(&text){
                            Ok(buyer_info) => {
                                return Ok(buyer_info);
                            }
                            Err(e) => {
                                log::error!("获取购票人信息json解析失败：{}",e);
                                return Err(format!("获取购票人信息json解析失败：{}",e))
                            }

                        }
                    }
                    Err(e) => {
                        log::error!("获取购票人信息失败：{}",e);
                        return Err(format!("获取购票人信息失败：{}",e))
                    }

                }
            }
            else{
                
                log::debug!("请求响应失败: {:?}", resp);
                return Err(format!("请求响应失败: {}", resp.status()));
            }
        }
        Err(e) => {
            Err(format!("请求失败: {}", e))
        }
    }
}

pub async fn get_project(client: Arc<Client>, project_id : &str) -> Result<InfoResponse,String>{
    let req = client.get(format!("https://show.bilibili.com/api/ticket/project/getV2?id={}",project_id));
    let response = req.send().await;
    match response {
        Ok(resp)=>{
            if resp.status().is_success(){
                match tokio::task::block_in_place(||{
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(resp.text())
                }){
                    Ok(text) => {
                        log::debug!("获取项目详情：{}",text);
                        match serde_json::from_str::<InfoResponse>(&text){
                            Ok(ticket_info) => {
                                return Ok(ticket_info);
                            }
                            Err(e) => {
                                log::error!("获取项目详情json解析失败：{}",e);
                                return Err(format!("获取项目详情json解析失败：{}",e))
                            }

                        }
                    }
                    Err(e) => {
                        log::error!("获取项目详情失败：{}",e);
                        return Err(format!("获取项目详情失败：{}",e))
                    }

                }
            }
            else{
                
                log::debug!("请求响应失败: {:?}", resp);
                return Err(format!("请求响应失败: {}", resp.status()));
            }
        }
        Err(e) => {
            Err(format!("请求失败: {}", e))
        }
    }

}


//轮询登录状态
pub async fn poll_qrcode_login(qrcode_key: &str,user_agent: Option<&str>) ->QrCodeLoginStatus {
    
    
    let client_builder = Client::builder();
    let client = if let Some(ua) = user_agent {
        client_builder.user_agent(ua)
    } else {
        client_builder.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/110.0.0.0 Safari/537.36")
    }.build()
    .unwrap_or_default();
    
    let max_attempts = 60;
    
    for attempt in 1..max_attempts{

    
    //轮询
    let response = match request_get(
        &client,
        &format!("https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}", qrcode_key),
       
        None,
    ).await {
        Ok(resp) => resp,
        Err(e) => return QrCodeLoginStatus::Failed(e.to_string()),
    };

    let mut all_cookies = Vec::new();
    let cookie_headers = response.headers().get_all(reqwest::header::SET_COOKIE);
    for value in cookie_headers {
     if let Ok(cookie_str) = value.to_str() {
        
        if let Some(end_pos) = cookie_str.find(';') {
            all_cookies.push(cookie_str[0..end_pos].to_string());
        } else {
            all_cookies.push(cookie_str.to_string());
        }
    }
    }
    
    let json = match response.json::<serde_json::Value>().await {
        Ok(j) => j,
        Err(e) => return QrCodeLoginStatus::Failed(e.to_string()),
    };
    
    
    let code = json["data"]["code"].as_i64().unwrap_or(-1);
    match code {
        0 => {
            //json获取cookie
            
            if let Some(cookie_info) = json["data"]["cookie_info"].as_object() {
                for (key, value) in cookie_info {
                    if let Some(val_str) = value["value"].as_str() {
                        all_cookies.push(format!("{}={}", key, val_str));
                    }
                }
            }
            
            
            if !all_cookies.is_empty() {
                return QrCodeLoginStatus::Success(all_cookies.join("; "));
            } else {
                return QrCodeLoginStatus::Failed("无法获取Cookie信息".to_string());
            }
        },
        86038 => return QrCodeLoginStatus::Expired,
        86090 => {
            log::info!("二维码已扫描，等待确认 (尝试 {} / {} 次)", attempt, max_attempts);
            //return QrCodeLoginStatus::Scanning;
        },
        86101 => {
            log::info!("二维码已生成，等待扫描 (尝试 {} / {} 次)", attempt, max_attempts);
            //return QrCodeLoginStatus::Pending
        },
        _ => {
            let message = json["message"].as_str().unwrap_or("未知错误");

            return QrCodeLoginStatus::Failed(message.to_string())
        }
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
QrCodeLoginStatus::Expired
}


pub async fn get_ticket_token(cookie_manager:Arc<CookieManager>, project_id : &str , screen_id: &str, ticket_id: &str) -> Result<String,TokenRiskParam>{
    
    

    let params = serde_json::json!({
        "project_id": project_id,
        "screen_id": screen_id,
        "sku_id": ticket_id,
        "count": 1,
        "order_type": 1,
        "token": "",
        "requestSource": "neul-next",
        "newRisk": "true",
    });
    
    let url = format!("https://show.bilibili.com/api/ticket/order/prepare?project_id={}",project_id);
    let response = cookie_manager
        .post(&url).await
        .json(&params)
        .send()
        .await;
    match response {
        Ok(resp) => {
            if resp.status().is_success(){
                match tokio::task::block_in_place(||{
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(resp.json::<serde_json::Value>())
                }){
                    Ok(json) => {
                        log::debug!("获取票token：{}",json);
                        let code = json["errno"].as_i64().unwrap_or(-1);
                        let msg = json["msg"].as_str().unwrap_or("未知错误");
                        
                        match code {
                            0 => {
                                let token = json["data"]["token"].as_str().unwrap_or("");
                                return Ok(token.to_string());
                            }
                            -401 => {
                                log::info!("需要进行人机验证");
                                let mid = json["data"]["ga_data"]["riskParams"]["mid"].as_str().unwrap_or("");
                                let decision_type = json["data"]["ga_data"]["riskParams"]["decision_type"].as_str().unwrap_or("");
                                let buvid = json["data"]["ga_data"]["riskParams"]["buvid"].as_str().unwrap_or("");
                                let ip = json["data"]["ga_data"]["riskParams"]["ip"].as_str().unwrap_or("");
                                let scene = json["data"]["ga_data"]["riskParams"]["scene"].as_str().unwrap_or("");
                                let ua = json["data"]["ga_data"]["riskParams"]["ua"].as_str().unwrap_or("");
                                let v_voucher = json["data"]["ga_data"]["riskParams"]["v_voucher"].as_str().unwrap_or("");
                                let risk_param = json["data"]["ga_data"]["riskParams"].clone();
                                let token_risk_param = TokenRiskParam {
                                    code: code as i32,
                                    message: msg.to_string(),
                                    mid: Some(mid.to_string()),
                                    decision_type: Some(decision_type.to_string()),
                                    buvid: Some(buvid.to_string()),
                                    ip: Some(ip.to_string()),
                                    scene: Some(scene.to_string()),
                                    ua: Some(ua.to_string()),
                                    v_voucher: Some(v_voucher.to_string()),
                                    risk_param: Some(risk_param.clone()),
                                };
                                log::debug!("{:?}", token_risk_param);
                                return Err(token_risk_param);
                            }
                            _ => {
                                log::error!("获取token失败，未知错误码：{}，错误信息：{}，请提issue修复此问题", code, msg);
                                log::error!("{:?}", json);
                                return Err(TokenRiskParam {
                                    code: code as i32,
                                    message: msg.to_string(),
                                    mid: None,
                                    decision_type: None,
                                    buvid: None,
                                    ip: None,
                                    scene: None,
                                    ua: None,
                                    v_voucher: None,
                                    risk_param: None,
                                });
                            }
                        }
                },
                Err(e) => {
                    log::error!("解析票务token响应失败: {}", e);
                    return Err(TokenRiskParam{
                        code: 999 as i32,
                        message: e.to_string(),
                        mid: None,
                        decision_type: None,
                        buvid: None,
                        ip: None,
                        scene: None,
                        ua: None,
                        v_voucher: None,
                        risk_param: None,
                    })
                }
            }
            }else{
                log::error!("获取票token失败，服务器不期待响应，响应状态码：{}",resp.status());
                return Err(TokenRiskParam{
                    code: 999 as i32,
                    message: resp.status().to_string(),
                    mid: None,
                    decision_type: None,
                    buvid: None,
                    ip: None,
                    scene: None,
                    ua: None,
                    v_voucher: None,
                    risk_param: None,
                });
            }
        }
        Err(e) => {
            log::error!("获取票token失败，错误信息：{}",e);
            return Err(TokenRiskParam{
                code: 999 as i32,
                message: e.to_string(),
                mid: None,
                decision_type: None,
                buvid: None,
                ip: None,
                scene: None,
                ua: None,
                v_voucher: None,
                risk_param: None,
            });
        }
    }

}

pub async fn confirm_ticket_order(cookie_manager:Arc<CookieManager>,project_id : &str,token: &str) -> Result<ConfirmTicketResult, String> {
    let url = format!("https://show.bilibili.com/api/ticket/order/confirmInfo?token={}&voucher=&project_id={}&requestSource=neul-next",token,project_id);
    let response = cookie_manager.get(&url)
        .await
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
        
    if !response.status().is_success() {
        return Err(format!("请求失败: {}", response.status()));
    }
    let text = response.text()
        .await
        .map_err(|e| format!("获取响应文本失败: {}", e))?;
    log::debug!("确认订单响应：{}", text);
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应文本失败: {}", e))?;
    if json["errno"]!=0 {
        return Err(format!("确认订单失败: {}", json["msg"].as_str().unwrap_or("未知错误")));
    }
    let confirm_result = serde_json::from_value(json["data"].clone())
        .map_err(|e| format!("解析确认订单结果失败: {}", e))?;
    Ok(confirm_result)
}

pub async fn create_order(
    cookie_manager: Arc<CookieManager>,
    project_id: &str,
    token: &str,
    confirm_result: &ConfirmTicketResult,
    biliticket: &BilibiliTicket,
    buyer_info: &Vec<BuyerInfo>,
    is_mobile: bool,
    need_retry: bool,
    fast_mode: bool,
    screen_size: Option<(u32, u32)> // 可选参数：(宽度,高度)
) -> Result<Value, i32> {
    let url = format!("https://show.bilibili.com/api/ticket/order/createV2?project_id={}", project_id);
    
    // 选择适当的位置类型
    let position_type = if need_retry && is_mobile {
        ClickPositionType::RetryButton
    } else if is_mobile {
        ClickPositionType::MobileConfirm
    } else {
        ClickPositionType::PcConfirm
    };
    
    // 提取屏幕尺寸（如果提供）
    let (width, height) = screen_size.unwrap_or((1080, 2400));
    
    // 生成点击位置
    let click_position = random_click_position(
        position_type, 
        fast_mode,
        Some(width),
        Some(height)
    ).await.to_string();

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let count = confirm_result.count.clone();
    let pay_money = confirm_result.pay_money.clone();

    let ticket_id = match biliticket.select_ticket_id.clone() {
        Some(id) => id,
        None => return Err(999), 
    };
    let ticket_id_int = ticket_id.parse::<i64>().map_err(|_| 999)?;

    let data = json!({
        "project_id": project_id.parse::<i64>().unwrap_or(0),
        "screen_id": biliticket.screen_id.parse::<i64>().unwrap_or(0),
        "sku_id": ticket_id_int, 
        "token": token,
        "buyer_info": serde_json::to_string(buyer_info).unwrap_or_default(),
        "clickPosition": click_position,
        "newRisk": true,
        "requestSource": if is_mobile { "neul-next" } else { "pc-new" }, 
        "deviceId": biliticket.device_id.clone(),
        "pay_money": pay_money,
        "count": count,
        "timestamp": timestamp,
        "order_type": 1, 
    });

    log::debug!("抢票data ：{:?}", data);
    let response = cookie_manager.post(&url).await
        .json(&data)
        .send()
        .await
        .map_err(|e| {
            log::error!("请求失败: {}", e);
            412
        })?;
    let text = response
        .text()
        .await
        .map_err(|e| {
            log::error!("获取响应文本失败: {}", e);
            412
        })?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| {
            log::error!("解析响应文本失败: {}", e);
            412
        })?;
    log::info!("{:?}", value);
    if value["errno"] != 0{
        
        return Err(value["errno"].as_i64().unwrap_or(412) as i32);

    }
    Ok(value)
}    


/// 点击位置类型枚举
#[derive(Debug, Clone, Copy)]
pub enum ClickPositionType {
    /// PC端确认下单按钮位置
    PcConfirm,
    /// 手机端确认下单按钮位置
    MobileConfirm,
    /// "再试一次"按钮位置（屏幕中间）
    RetryButton,
}

/// 生成随机点击位置
/// 
/// # 参数
/// * `position_type` - 位置类型：PC/手机/重试按钮
/// * `fast_mode` - 时间间隔模式：true为快模式(0.8-4.6秒)，false为慢模式(4-12秒)
/// * `screen_width` - (可选)手机屏幕宽度，用于计算比例坐标，默认1080
/// * `screen_height` - (可选)手机屏幕高度，用于计算比例坐标，默认2400
/// 
pub async fn random_click_position(
    position_type: ClickPositionType, 
    fast_mode: bool,
    screen_width: Option<u32>,
    screen_height: Option<u32>
) -> Value {
    let mut rng = thread_rng();
    
    // 获取手机屏幕尺寸（默认使用常见尺寸1080x2400）
    let mobile_width = screen_width.unwrap_or(1080);
    let mobile_height = screen_height.unwrap_or(2400);
    
    // 根据不同设备/按钮类型确定基准坐标和偏移范围
    let (base_x, base_y, offset_range) = match position_type {
        ClickPositionType::PcConfirm => {
            // PC端确认下单按钮位置(右侧中下部)
            (1131, 636, 10)
        },
        ClickPositionType::MobileConfirm => {
            // 手机端确认下单按钮位置(右下角)
            // 使用比例计算：x在屏幕宽度的0.55-0.9之间，y在屏幕底部附近
            let x_ratio = rng.gen_range(0.55..0.9);
            let y_ratio = rng.gen_range(0.9..0.95);
            
            let x = (mobile_width as f32 * x_ratio) as i32;
            let y = (mobile_height as f32 * y_ratio) as i32;
            
            (x, y, mobile_width.min(20) as i32 / 4) // 偏移范围根据屏幕宽度按比例缩放
        },
        ClickPositionType::RetryButton => {
            // 手机版"再试一次"按钮位置(屏幕中间靠下)
            // x坐标在屏幕宽度的1/3到2/3之间，y坐标在屏幕高度的2/3左右
            let x_ratio = rng.gen_range(0.33..0.67); // 屏幕宽度的2/6到4/6之间
            let y_ratio = rng.gen_range(0.6..0.7);   // 屏幕高度的2/3左右
            
            let x = (mobile_width as f32 * x_ratio) as i32;
            let y = (mobile_height as f32 * y_ratio) as i32;
            
            (x, y, mobile_width.min(30) as i32 / 4) // 偏移较大，因为这是个大按钮
        }
    };
    
    // 生成随机偏移
    let offset_x = rng.gen_range(-offset_range..=offset_range);
    let offset_y = rng.gen_range(-offset_range..=offset_range);
    
    // 计算最终坐标
    let final_x = base_x + offset_x;
    let final_y = base_y + offset_y;
    
    // 获取当前时间戳
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    // 根据模式生成不同的延迟时间
    let random_delay = if fast_mode {
        // 快模式：0.8-4.6秒
        rng.gen_range(800..4600)
    } else {
        // 慢模式：4-12秒
        rng.gen_range(4000..12000)
    };
    
    // 计算起始时间
    let origin = now - random_delay;
    
    // 构建JSON对象
    json!({
        "x": final_x,
        "y": final_y,
        "origin": origin,
        "now": now
    })
}