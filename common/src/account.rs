use serde::{Serialize, Deserialize};
use reqwest::Client;
use crate::{cookie_manager, http_utils::{request_get_sync,request_post_sync}};
use serde_json;
use std::sync::Arc;
use crate::cookie_manager::CookieManager;
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
    pub avatar_url: Option<String>, //头像地址
    #[serde(skip)]
    pub avatar_texture: Option<eframe::egui::TextureHandle>, //头像地址
    #[serde(skip)] 
    pub cookie_manager: Option<Arc<CookieManager>>, //cookie管理器
}
impl std::fmt::Debug for Account{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("uid", &self.uid)
            .field("name", &self.name)
            .field("level", &self.level)
            .field("cookie", &self.cookie)
            .field("csrf", &self.csrf)
            .field("is_login", &self.is_login)
            .field("account_status", &self.account_status)
            .field("vip_label", &self.vip_label)
            .field("is_active", &self.is_active)
            .field("avatar_url", &self.avatar_url)
            .field("avatar_texture", &"SKipped")
            .field("client", &self.cookie_manager)
            .finish()
    }
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
    let cookie_manager = Arc::new(rt.block_on(async{
        cookie_manager::CookieManager::new(cookie, Some(ua), 0).await
    }));
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
            cookie: cookie_manager.get_all_cookies(),
            csrf: extract_csrf(cookie),
            is_login: true,
            account_status: "空闲".to_string(),
            vip_label: data["vip_label"]["text"].as_str().unwrap_or("").to_string(),
            is_active: true,
            avatar_url: Some(data["face"].as_str().unwrap_or("").to_string()),
            avatar_texture: None,
            cookie_manager: Some(cookie_manager),
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
    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt.block_on(async{
        account.cookie_manager.clone().unwrap().post("https://passport.bilibili.com/login/exit/v2")
        .await
        .json(&data)
        .send()
        .await
    });
    
    let resp = match response {
        Ok(res) => res,
        Err(e) => return Err(format!("请求失败: {}", e)),
    };
    log::debug!("退出登录响应： {:?}",resp);
    Ok(resp.status().is_success())
    
}


//提取 csrf
fn extract_csrf(cookie: &str) -> String {
    // 打印原始cookie用于调试
    log::debug!("提取CSRF的原始cookie: {}", cookie);
    
    for part in cookie.split(';') {
        let part = part.trim();
        // 检查是否以bili_jct开头（不区分大小写）
        if part.to_lowercase().starts_with("bili_jct=") {
            // 找到等号位置
            if let Some(pos) = part.find('=') {
                let value = &part[pos + 1..];
                // 去除可能的引号
                let value = value.trim_matches('"').trim_matches('\'');
                log::debug!("成功提取CSRF值: {}", value);
                return value.to_string();
            }
        }
    }
    
    // 没找到，记录并返回空字符串
    log::warn!("无法从cookie中提取CSRF值");
    String::new()
}
impl Account {
    // 确保每个账号都有自己的 client
    pub fn ensure_client(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if self.cookie_manager.is_none() {
            rt.block_on(async{
            self.cookie_manager = Some(Arc::new(CookieManager::new(
                &self.cookie,
                None,
                0,
            ).await))
        });
        }
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