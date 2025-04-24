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

    //解析cookie字符串 
    //TODO：（ck登录待去多余字符）
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

    //现有client创建ck管理器 (已封进client的ck无法读取)
    pub fn from_client(client: Arc<Client>, original_cookie : &str) -> Self {
        let cookies = Self::parse_cookie_string(original_cookie);
        Self {
            client: client,
            cookies: Arc::new(Mutex::new(cookies))
        }
    }

    //更新单个字段
    pub fn update_cookie(&self, key:&str, value:&str){
        let mut cookies = self.cookies.lock().unwrap();
        cookies.insert(key.to_string(), value.to_string());
        log::debug!("更新Cookie: {}={}", key, value);
    }

    //移除某个键对应的值
    pub fn remove_cookie(&self, key:&str) -> bool {
        let mut cookies = self.cookies.lock().unwrap();
        let existed = cookies.remove(key).is_some();
        if existed {
            log::debug!("删除Cookie: {}", key);
        } else {
            log::debug!("Cookie不存在: {}", key);
        }
        existed
    }

    //更新大量ck
    pub fn update_cookies(&self, cookies_str: &str) {
        let new_cookies = Self::parse_cookie_string(cookies_str);
        let mut cookies = self.cookies.lock().unwrap();
        for(key,value) in new_cookies{
            cookies.insert(key, value);
        }
        log::debug!("批量更新Cookie: {}", cookies_str);
    }

    //清除所有ck
    pub fn clear_all_cookies(&self) {
        let mut cookies = self.cookies.lock().unwrap();
        cookies.clear();
        log::debug!("清除所有Cookie");
    }

    //获取某个键的值
    pub fn get_cookie(&self, key: &str) -> Option<String> {
        let cookies = self.cookies.lock().unwrap();
        cookies.get(key).cloned()
    }
}
