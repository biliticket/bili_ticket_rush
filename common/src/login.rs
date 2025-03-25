use crate::account::Account;

pub fn qrcode_login() -> Result<String, String> {
    
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
    }

pub fn password_login(username: &str, password: &str) -> Result<String, String> {
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}

pub fn sms_login(username: &str, sms_code: &str) -> Result<String, String> {
    //测试调用
    Ok("https://account.bilibili.com/h5/account-h5/auth/scan-web?navhide=1&callback=close&qrcode_key=7d0bd3e133117eab86bc5f42f8731e0e&from=main-fe-header".to_string())
}