use crate::account::add_account;
use crate::account::Account;
use crate::http_utils::{request_get,request_post,request_get_sync};
use serde_json::json;
use crate::utility::CustomConfig;
use crate::chapcha::chapcha;
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
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}

pub async fn send_loginsms(phone: &str, client: &Client, ua: &str, custom_config: CustomConfig) -> Result<String, String> {
    
    
        // 发送请求
        let response = request_get(
            client,
            "https://passport.bilibili.com/x/passport-login/captcha",
            Some(ua.to_string()),
            None,
        ).await.map_err(|e| e.to_string())?;
        log::info!("获取验证码: {:?}", response);
        // 解析 JSON
        let json = response.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
        let gt = json["data"]["geetest"]["gt"].as_str().unwrap_or("");
        let challenge = json["data"]["geetest"]["challenge"].as_str().unwrap_or("");
        let token = json["data"]["token"].as_str().unwrap_or("");
        let referer = "https://passport.bilibili.com/x/passport-login/captcha";
        match chapcha(custom_config.clone(), gt, challenge, referer, 33).await {
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
                log::info!("验证码数据: {:?}", json_data);
                let send_sms = request_post(
                    client,
                    "https://passport.bilibili.com/x/passport-login/web/sms/send",
                    Some(ua.to_string()),
                    None,
                    Some(&json_data),
                ).await.map_err(|e| e.to_string())?;
                
                let json_response = send_sms.json::<serde_json::Value>().await.map_err(|e| e.to_string())?;
                if json_response["code"].as_i64() == Some(0) {
                    log::info!("验证码发送成功");
                    Ok("验证码发送成功".to_string())
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

pub fn sms_login(username: &str, sms_code: &str, client: &Client, ua: &str, custom_config: CustomConfig) -> Result<String, String> {
    //测试调用
    Ok("验证码已发送".to_string())
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

