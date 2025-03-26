use reqwest::{Client, header, Response, Error};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

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


pub async fn request_get(client: &Client, url: &str, ua: Option<String>, cookie: Option<&str>) -> Result<Response, Error> {
    let ua = ua.unwrap_or_else(get_random_ua);
    
    let mut req = client.get(url)
        .header(header::USER_AGENT, ua);
    
    if let Some(cookie_str) = cookie {
        req = req.header(header::COOKIE, cookie_str);
    }
    
    req.send().await
}


pub async fn request_post<T: serde::Serialize + ?Sized>(
    client: &Client, 
    url: &str, 
    ua: Option<String>,
    cookie: Option<&str>,
    json_data: Option<&T>
) -> Result<Response, Error> {
    let ua = ua.unwrap_or_else(get_random_ua);
    
    let mut req = client.post(url)
        .header(header::USER_AGENT, ua);
    
    if let Some(cookie_str) = cookie {
        req = req.header(header::COOKIE, cookie_str);
    }
    
    if let Some(data) = json_data {
        req = req.json(data);
    }
    
    req.send().await
}

