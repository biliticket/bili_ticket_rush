use common::taskmanager::*;
use common::http_utils::request_get;
use serde_json;
use common::login::QrCodeLoginStatus;
use reqwest::Client;
//这里实现抢票api

pub async fn perform_ticket_grab(request: &TicketRequest) -> Result<TicketResult, Box<dyn std::error::Error + Send + Sync>> {
    // 这里实现实际的抢票逻辑
    
    // 模拟抢票过程
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    print!("抢票成功");
    
    // 返回结果
    Ok(TicketResult {
        success: true,
        order_id: Some("ORDER12345".to_string()),
        message: Some("抢票成功".to_string()),
        ticket_info: common::TicketInfo {
            id: request.ticket_id.clone(),
            name: "测试演出".to_string(),
            price: 280.0,
        },
        timestamp: std::time::Instant::now(),
    })
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