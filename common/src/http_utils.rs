use reqwest::{Client, header, Response, Error};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use serde_json;

// 随机UA生成

pub fn get_random_ua() -> String {
    let ua_list = [
        "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 Chrome/98.0.4758.101",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/99.0.4844.51",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Safari/605.1.15",
        "Mozilla/5.0 (Linux; Android 12; SM-S908B) Chrome/101.0.4951.41",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/102.0.0.0"
    ];
    
    let mut rng = thread_rng();
    ua_list.choose(&mut rng).unwrap_or(&ua_list[0]).to_string()
}


pub async fn request_get(client: &Client, url: &str, cookie: Option<&str>) -> Result<Response, Error> {
    
    
    let mut req = client.get(url);
        
    
    if let Some(cookie_str) = cookie {
        req = req.header(header::COOKIE, cookie_str);
    }
    
    req.send().await
}

pub async fn request_post<T: serde::Serialize + ?Sized>(
    client: &Client, 
    url: &str, 
    
    cookie: Option<&str>,
    json_data: Option<&T>
) -> Result<Response, Error> {
    
    
    let mut req = client.post(url);
        
    if let Some(cookie_str) = cookie {
        req = req.header(header::COOKIE, cookie_str);
    }
    
    if let Some(data) = json_data {
        if let Ok(json_value) = serde_json::to_value(data) {
            if let Some(obj) = json_value.as_object() {
                let mut form = std::collections::HashMap::new();
                for (key, value) in obj {
                    // 转换所有值为字符串
                    form.insert(key, value.to_string().trim_matches('"').to_string());
                }
                req = req.form(&form);
            }
        }
    }
    
    req.send().await
}


pub fn request_get_sync(client: &Client, url: &str, ua: Option<String>, cookie: Option<&str>) -> Result<Response, Error> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(request_get(client, url,  cookie))
    
}

pub fn request_post_sync(client: &Client, url: &str, ua: Option<String>, cookie: Option<&str>, json_data: Option<&serde_json::Value>) -> Result<Response, Error> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(request_post(client, url, cookie, json_data))
    
}

pub fn request_form_sync(
    client: &Client, 
    url: &str,
    ua: Option<String>,
    cookie: Option<&str>,
    form_data: &HashMap<String, String>
) -> Result<Response, Error> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut req = client.post(url);
        
        if let Some(cookie_str) = cookie {
            req = req.header(header::COOKIE, cookie_str);
        }

        if let Some(ua_str) = ua {
            req = req.header(header::USER_AGENT, ua_str);
        }
        
        req = req.form(&form_data);
        req.send().await
    })
}


pub fn request_json_form_sync(
    client: &Client, 
    url: &str,
    ua: Option<String>,
    referer: Option<String>,
    cookie: Option<&str>,
    json_form: &serde_json::Map<String, serde_json::Value>
) -> Result<Response, Error> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut req = client.post(url);
        
        if let Some(cookie_str) = cookie {
            req = req.header(header::COOKIE, cookie_str);
        }

        if let Some(ua_str) = ua {
            req = req.header(header::USER_AGENT, ua_str);
        }
        
        if let Some(referer_str) = referer {
            req = req.header(header::REFERER, referer_str);
        }
        // 创建一个特殊的表单，保留数字类型
        let mut form = std::collections::HashMap::new();
        for (key, value) in json_form {
            match value {
                // 字符串类型
                serde_json::Value::String(s) => {
                    form.insert(key.clone(), s.clone());
                },
                // 数字类型 - 直接转为字符串但不加引号
                serde_json::Value::Number(n) => {
                    form.insert(key.clone(), n.to_string());
                },
                // 布尔类型
                serde_json::Value::Bool(b) => {
                    form.insert(key.clone(), b.to_string());
                },
                // 其他类型
                _ => {
                    form.insert(key.clone(), value.to_string().trim_matches('"').to_string());
                }
            }
        }
        
        req = req.form(&form);
        req.send().await
    })
}