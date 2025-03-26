use crate::account::Account;
use crate::http_utils::{request_get,request_post};
use reqwest::Client;
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
    
    
    if let Some(url) = json["data"]["url"].as_str() {
        Ok(url.to_string())
    } else {
        Err("无法获取二维码URL".to_string())
    }
})
}
pub fn password_login(username: &str, password: &str) -> Result<String, String> {
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}

pub fn sms_login(username: &str, sms_code: &str) -> Result<String, String> {
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}

pub fn send_loginsms(phone: &str) -> Result<String, String> {
    //测试调用
    Ok("验证码已发送".to_string())
}

pub fn cookie_login(cookie: &str) -> Result<String, String> {
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}