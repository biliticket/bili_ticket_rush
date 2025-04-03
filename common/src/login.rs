use crate::account::add_account;
use crate::account::Account;
use crate::http_utils::{request_get,request_post,request_get_sync};
use serde_json::json;
use crate::utility::CustomConfig;
use crate::captcha::captcha;
use reqwest::Client;

pub struct LoginInput{
    pub phone: String,
    pub account: String,
    pub password: String,
    pub sms_code: String,
    pub cookie: String,
}

pub struct QrCodeLoginTask {
    pub qrcode_key: String,
    pub qrcode_url: String,
    pub start_time: std::time::Instant,
    pub status: QrCodeLoginStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QrCodeLoginStatus {
    Pending,
    Scanning,
    Confirming,
    Success(String), //成功时返回cookie信息
    Failed(String),  //失败时返回错误信息
    Expired,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SendLoginSmsStatus{
    Success(String),
    Failed(String),
}

pub  fn qrcode_login(client: &Client) -> Result<String, String> {
   // 创建一个临时的运行时来执行异步代码
   let rt = tokio::runtime::Runtime::new().unwrap();
   rt.block_on(async {
   let response = request_get(
    client,
    "https://passport.bilibili.com/x/passport-login/web/qrcode/generate",
    
        None,
    ).await.map_err(|e| e.to_string())?;
    
    let json = response.json::<serde_json::Value>()
    .await.map_err(|e| e.to_string())?;
    
    
    if let Some(qrcode_key) = json["data"]["qrcode_key"].as_str()  {
        Ok(qrcode_key.to_string())
    } else {
        Err("无法获取二维码URL".to_string())
    }
})
}
pub fn password_login(username: &str, password: &str) -> Result<String, String> {
    Err("暂不支持账号密码登录".to_string())
}

pub async fn send_loginsms(phone: &str, client: &Client, custom_config: CustomConfig) -> Result<String, String> {
        
        let response = request_get(
            client,
            "https://www.bilibili.com/",
       
            None,
        ).await.map_err(|e| e.to_string())?;
        
        log::debug!("{:?}", response.cookies().collect::<Vec<_>>());
        
        
    
        // 发送请求
        let response = request_get(
            client,
            "https://passport.bilibili.com/x/passport-login/captcha",
       
            None,
        ).await.map_err(|e| e.to_string())?;
        log::info!("获取验证码: {:?}", response);
       
        let json = response.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
        let gt = json["data"]["geetest"]["gt"].as_str().unwrap_or("");
        let challenge = json["data"]["geetest"]["challenge"].as_str().unwrap_or("");
        let token = json["data"]["token"].as_str().unwrap_or("");
        let referer = "https://passport.bilibili.com/x/passport-login/captcha";
        match captcha(custom_config.clone(), gt, challenge, referer, 33).await {
            Ok(result_str) => {
                log::info!("验证码识别成功: {}", result_str);
                let result: serde_json::Value = serde_json::from_str(&result_str).map_err(|e| e.to_string())?;
               
                let json_data = json!({
                            "cid": 86,
                            "tel": phone.parse::<i64>().unwrap_or(0),
                            "token": token,
                            "source":"main_mini",
                            "challenge": result["challenge"],
                            "validate": result["validate"],
                            "seccode": result["seccode"],
                            });
                log::debug!("验证码数据: {:?}", json_data);
                let send_sms = request_post(
                    client,
                    "https://passport.bilibili.com/x/passport-login/web/sms/send",
                    
                    None,
                    Some(&json_data),
                ).await.map_err(|e| e.to_string())?;
                
                let json_response = send_sms.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
                log::debug!("验证码发送响应: {:?}", json_response);
                if json_response["code"].as_i64() == Some(0) {
                    let captcha_key = json_response["data"]["captcha_key"].as_str().unwrap_or("");
                    log::info!("验证码发送成功");
                    log::debug!("captcha_key: {:?}", captcha_key);
                    Ok(captcha_key.to_string())
                } else {
                    log::error!("验证码发送失败: {}", json_response["message"].as_str().unwrap_or("未知错误"));
                    Err("验证码发送失败".to_string())
                }
                }
            Err(e) => {
                log::error!("验证码识别失败: {}", e);
                Err("验证码识别失败".to_string())
            }
                
        }
       
    
}

pub async fn sms_login(phone: &str, sms_code: &str, captcha_key:&str, client: &Client) -> Result<String, String> {
    let data = serde_json::json!({
        "cid": 86,
        "tel": phone.parse::<i64>().unwrap_or(0),
        "code": sms_code.parse::<i64>().unwrap_or(0),
        "source":"main_mini",
        "captcha_key":captcha_key,
    });
    log::debug!("短信登录数据: {:?}", data);
    let login_response = request_post(
        client,
        "https://passport.bilibili.com/x/passport-login/web/login/sms",
        
        None,
        Some(&data),
    ).await.map_err(|e| e.to_string())?;
    let mut all_cookies = Vec::new();
    let cookie_headers = login_response.headers().get_all(reqwest::header::SET_COOKIE);
    log::debug!("headers返回：{:?}",cookie_headers);
    for value in cookie_headers {
        if let Ok(cookie_str) = value.to_str() {
     
          if let Some(end_pos) = cookie_str.find(';') {
            all_cookies.push(cookie_str[0..end_pos].to_string());
           } else {
           all_cookies.push(cookie_str.to_string());
           }
         }
     }
     log::info!("获取cookie: {:?}", all_cookies);
    let json_response = login_response.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    log::debug!("登录接口响应：{:?}",json_response);
    if json_response["code"].as_i64() == Some(0) {
        log::info!("短信登录成功！");
       log::info!("登录cookie：{:?}", all_cookies);
       return Ok(all_cookies.to_vec().join(";"));
        
    }
    Err("短信登录失败".to_string())
    
    
}

pub fn cookie_login(cookie: &str, client: &Client, ua: &str) -> Result<Account, String> {
    match add_account(cookie,client,ua){
        Ok(account) => {
            log::info!("ck登录成功");
            Ok(account)
        },
        Err(e) => {
            log::error!("ck登录失败: {}", e);
            Err(e)
        }
    }
}


