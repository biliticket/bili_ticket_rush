use common::http_utils::request_get;
use common::ticket::{*};
use serde_json;
use common::login::QrCodeLoginStatus;
use reqwest::Client;
use std::sync::Arc;

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


pub async fn get_ticket_token(client:Arc<Client>, project_id : &str , screen_id: &str, ticket_id: &str) -> Result<String,TokenRiskParam>{
    
    

    let params = serde_json::json!({
        "project_id": project_id,
        "screen_id": screen_id,
        "sku_id": ticket_id,
        "count": 1,
        "order_type": 1,
        "token": "",
        "requestSource": "pc-new",
        "newRisk": "true",
    });
    
    let url = format!("https://show.bilibili.com/api/ticket/order/prepare?project_id={}",project_id);
    let response = client.post(url)
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

pub async fn confirm_ticket_order(client:Arc<Client>,project_id : &str,token: &str) -> Result<ConfirmTicketResult, String> {
    let url = format!("https://show.bilibili.com/api/ticket/order/confirmInfo?token={}&voucher=&project_id={}&requestSource=pc-new",token,project_id);
    let response = client.get(&url)
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