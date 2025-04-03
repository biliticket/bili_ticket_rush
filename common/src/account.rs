use serde::{Serialize, Deserialize};
use reqwest::Client;
use crate::http_utils::{request_get_sync,request_post_sync};
use serde_json;

#[derive(Clone, Serialize, Deserialize)]
pub struct Account{
    pub uid: i64,  //UID
    pub name: String,   //昵称
    pub level: String,
    pub cookie: String, //cookie
    pub csrf : String,  //csrf
    pub is_login: bool,    //是否登录
    pub account_status: String,  //账号状态
    pub vip_label: String, //大会员，对应/nav请求中data['vip_label']['text']
    pub is_active: bool, //该账号是否启动抢票
    #[serde(skip)] 
    pub client: Option<reqwest::Client>,
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
    log::debug!("获取账号信息: {:?}", json);
    match json.get("code") {
        Some(code) if code.as_i64() == Some(0) => {} // 成功
        _ => return Err("获取账号信息失败".to_string()),
    }
    if let Some(data) = json.get("data") {
        let mut account = Account {
            uid: data["mid"].as_i64().unwrap_or(0),
            name: data["uname"].as_str().unwrap_or("账号信息获取失败，请删除重新登录").to_string(),
            level: data["level_info"]["current_level"].as_i64().unwrap_or(0).to_string(),
            cookie: cookie.to_string(),
            csrf: extract_csrf(cookie),
            is_login: true,
            account_status: "空闲".to_string(),
            vip_label: data["vip_label"]["text"].as_str().unwrap_or("").to_string(),
            is_active: false,
            client: Some(client.clone()),
        };
        account.ensure_client();
        Ok(account)
    } else {
        Err("无法获取用户信息".to_string())
    }
}

pub fn signout_account(account: &Account) -> Result<bool, String> {
    let data = serde_json::json!({
        "biliCSRF" : account.csrf,

    });
    let response = request_post_sync(
        account.client.as_ref().unwrap(),
        "https://passport.bilibili.com/login/exit/v2",
        None,
        None,
        Some(&data),
    ).map_err(|e| e.to_string())?;
    log::debug!("退出登录响应： {:?}",response);
    Ok(response.status().is_success())
    
}


//提取 csrf
fn extract_csrf(cookie: &str) -> String {
    cookie.split(";")
        .find(|s| s.contains("bili_jct="))
        .map(|s| s.trim().replace("bili_jct=", ""))
        .unwrap_or_default()
}

impl Account {
    // 确保每个账号都有自己的 client
    pub fn ensure_client(&mut self) {
        if self.client.is_none() {
            self.client = Some(create_client_for_account(&self.cookie));
        }
    }

    // 刷新 client（如果需要重新创建）
    pub fn refresh_client(&mut self) {
        self.client = Some(create_client_for_account(&self.cookie));
    }
}

// 创建client
fn create_client_for_account(cookie: &str) -> reqwest::Client {
    use reqwest::header;
    
    
    let random_id = format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos());
    
    
    let user_agent = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 {}", 
        random_id
    );
    
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_str(&user_agent).unwrap_or_else(|_| {
            // 提供一个替代值，而不是使用 unwrap_or_default()
            header::HeaderValue::from_static("Mozilla/5.0")
        })
    );
    
    // 创建 client
    reqwest::Client::builder()
        .default_headers(headers)
        .cookie_store(true)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}