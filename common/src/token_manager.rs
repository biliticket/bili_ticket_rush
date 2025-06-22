use std::sync::Arc;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE, REFERER, USER_AGENT};
use serde_json::Value;
use anyhow::{Result, anyhow};

use crate::cookie_manager::CookieManager;
use crate::utility::CustomConfig;

/// 获取ctoken和ptoken
/// 
/// # 参数
/// 
/// * `cookie_manager` - Cookie管理器
/// * `project_id` - 项目ID
/// * `config` - 自定义配置
/// 
/// # 返回值
/// 
/// 返回一个包含ctoken和ptoken的元组
pub async fn get_tokens(cookie_manager: Arc<CookieManager>, project_id: &str, config: &CustomConfig) -> Result<(String, String)> {
    // 如果禁用了token验证，返回空字符串
    if !config.enable_token_verify {
        log::info!("Token验证已禁用，跳过获取ctoken和ptoken");
        return Ok(("".to_string(), "".to_string()));
    }
    
    log::info!("开始获取ctoken和ptoken");
    
    let client = reqwest::Client::new();
    
    // 构建请求头
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    headers.insert(REFERER, HeaderValue::from_str(&format!("https://show.bilibili.com/platform/detail.html?id={}", project_id))?);
    
    // 添加Cookie
    let cookie_str = cookie_manager.get_all_cookies();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str)?);
    
    // 构建请求URL
    let url = format!("https://show.bilibili.com/api/ticket/project/get?id={}", project_id);
    
    // 发送请求
    let response = client.get(&url)
        .headers(headers)
        .send()
        .await?;
    
    // 检查响应状态
    if !response.status().is_success() {
        return Err(anyhow!("获取token失败，HTTP状态码: {}", response.status()));
    }
    
    // 解析响应
    let response_text = response.text().await?;
    let response_json: Value = serde_json::from_str(&response_text)?;
    
    // 检查响应码
    let code = response_json["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        let message = response_json["message"].as_str().unwrap_or("未知错误");
        return Err(anyhow!("获取token失败，错误码: {}，错误信息: {}", code, message));
    }
    
    // 提取ctoken和ptoken
    let ctoken = response_json["data"]["ctoken"].as_str()
        .ok_or_else(|| anyhow!("响应中没有ctoken"))?
        .to_string();
    
    let ptoken = response_json["data"]["ptoken"].as_str()
        .ok_or_else(|| anyhow!("响应中没有ptoken"))?
        .to_string();
    
    log::info!("成功获取ctoken和ptoken");
    
    Ok((ctoken, ptoken))
}

/// 刷新token
/// 
/// 当token过期时，可以调用此函数刷新token
/// 
/// # 参数
/// 
/// * `cookie_manager` - Cookie管理器
/// * `project_id` - 项目ID
/// * `config` - 自定义配置
/// 
/// # 返回值
/// 
/// 返回一个包含新的ctoken和ptoken的元组
pub async fn refresh_tokens(cookie_manager: Arc<CookieManager>, project_id: &str, config: &CustomConfig) -> Result<(String, String)> {
    log::info!("开始刷新ctoken和ptoken");
    
    // 实现刷新token的逻辑，这里简单地调用get_tokens
    get_tokens(cookie_manager, project_id, config).await
}