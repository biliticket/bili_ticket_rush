use reqwest::{Client, Response ,header, RequestBuilder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; //?有用到吗
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CookieManager {
    client: Arc<Client>,
    cookies: Arc<Mutex<HashMap<String, String>>>,
}

impl CookieManager {
    pub fn new(original_cookie : &str , user_agent: Option<&str>) -> Self {
        let cookies = Self::parse_cookie_string(original_cookie);

        let client_builder = Client::builder()
            .cookie_store(true);
        let client = if let Some(ua) = user_agent {
            client_builder.user_agent(ua)
        }else{
            //理论上不应该到这里，到这里需检查是否漏参丢参
            client_builder.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36 Edg/135.0.0.6")
        }.build().unwrap_or_default();

        Self { client: (Arc::new(client)), cookies: (Arc::new(Mutex::new(cookies))) }
    }

    //解析cookie字符串（ck登录待去多于字符）
    fn parse_cookie_string(cookie_str: &str) -> HashMap<String,String> {
        let mut map = HashMap::new();
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(index) = cookie.find("=") {
                let (key , value) = cookie.split_at(index);
                if value.len() >1 {
                    map.insert(key.to_string(), value[1..].to_string());
                }
            }
        }
        map
    }
}
