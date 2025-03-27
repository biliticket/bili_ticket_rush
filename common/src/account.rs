use serde::{Serialize, Deserialize};
use reqwest::Client;
use crate::http_utils::{request_get_sync};
use serde_json;

#[derive(Clone, Serialize, Deserialize)]
pub struct Account{
    pub uid: i64,  //UID
    pub name: String,   //昵称
    pub level: String,
    pub cookie: String, //cookie
    pub csrf : String,  //csrf
    pub is_logged: bool,    //是否登录
    pub account_status: String,  //账号状态
    pub vip_label: String, //大会员，对应/nav请求中data['vip_label']['text']
    pub is_active: bool, //该账号是否启动抢票
}



pub fn add_account(cookie: &str ,client: &Client, ua: &str) -> Result<Account, String>{
    log::info!("添加账号: {}", cookie);
    let response = request_get_sync(
        client,
        "https://api.bilibili.com/x/web-interface/nav",
        Some(ua.to_string()),
        Some(cookie),
    ).map_err(|e| e.to_string())?;
    
    // 创建一个临时的运行时来执行异步代码
    let rt = tokio::runtime::Runtime::new().unwrap();
    let json = rt.block_on(async {
        response.json::<serde_json::Value>().await
    }).map_err(|e| e.to_string())?;
    log::info!("获取账号信息: {:?}", json);
    if let Some(data) = json.get("data") {
        let account = Account {
            uid: data["mid"].as_i64().unwrap_or(0),
            name: data["uname"].as_str().unwrap_or("账号信息获取失败，请删除重新登录").to_string(),
            level: data["level_info"]["current_level"].as_i64().unwrap_or(0).to_string(),
            cookie: cookie.to_string(),
            csrf: extract_csrf(cookie),
            is_logged: true,
            account_status: "空闲".to_string(),
            vip_label: data["vip_label"]["text"].as_str().unwrap_or("").to_string(),
            is_active: false,
        };
        Ok(account)
    } else {
        Err("无法获取用户信息".to_string())
    }
}

//提取 csrf
fn extract_csrf(cookie: &str) -> String {
    cookie.split(";")
        .find(|s| s.contains("bili_jct="))
        .map(|s| s.trim().replace("bili_jct=", ""))
        .unwrap_or_default()
}