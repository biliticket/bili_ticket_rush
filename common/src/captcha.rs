/* use bili_ticket_gt_python::slide::Slide;
use bili_ticket_gt_python::click::Click;
use bili_ticket_gt_python::abstraction::{Api, GenerateW, Test};

 */
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use crate::{account::Account, ticket::TokenRiskParam, utility::CustomConfig};

pub async fn captcha(custom_config: CustomConfig, gt: &str, challenge: &str, referer: &str, captcha_type:usize) -> Result<String, String> {
    // 0:本地打码  1：ttocr
    match custom_config.captcha_mode {
        0 => {
            // 本地打码
            
            Err("暂不支持本地打码，请使用其它方式".to_string())
        },
        1 => {
            // ttocr
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)  // 禁用证书验证
                .build()
                .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
            let form_data = json!({
                "appkey": custom_config.ttocr_key,
                "gt": gt,
                "challenge": challenge,
                "itemid": captcha_type,//33对应三代点字 32对应三代滑块
                "referer": referer,
            });
            log::info!("验证码请求参数: {:?}", form_data);
            let response = client.post("http://api.ttocr.com/api/recognize")
            .json(&form_data)
            .send()
            .await
            .map_err(|e| format!("发送请求失败: {}", e))?;
            log::info!("验证码请求响应: {:?}", response);
            let text = response.text()
            .await
            .map_err(|e| format!("读取响应内容失败: {}", e))?;
            
            // 打印文本内容
            log::info!("响应内容: {}", text);
            let json_response: serde_json::Value = serde_json::from_str(&text)
              .map_err(|e| format!("解析JSON失败: {}", e))?;

            if json_response["status"].as_i64() == Some(1){
                log::info!("验证码提交识别成功");
            }
            else{
                log::error!("验证码提交识别失败: {}", json_response["msg"].as_str().unwrap_or("未知错误"));
                return Err("验证码提交识别失败".to_string());
            }
            let result_id = json_response["resultid"].as_str().unwrap_or("");
            for _ in 0..20{
                let response = client.post("http://api.ttocr.com/api/results")
                .json(&json!({
                    "appkey": custom_config.ttocr_key,
                    "resultid": result_id,
                }))
                .send()
                .await
                .map_err(|e| format!("发送请求失败: {}", e))?;
            let text = response.text()
            .await
            .map_err(|e| format!("读取响应内容失败: {}", e))?;
            
            // 打印文本内容
            log::info!("响应内容: {}", text);
            let json_response: serde_json::Value = serde_json::from_str(&text)
              .map_err(|e| format!("解析JSON失败: {}", e))?;


                if json_response["status"].as_i64() == Some(1){
                    log::info!("验证码识别成功");
                    return Ok(serde_json::to_string(&json!({
                        "challenge": json_response["data"]["challenge"],
                        "validate": json_response["data"]["validate"],
                        "seccode": json_response["data"]["seccode"],
                    })).map_err(|e| format!("序列化JSON失败: {}", e))?);
                    
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            Err("验证码识别超时".to_string())
        },
        _ => Err("无效的验证码模式".to_string()),
    }
}

pub fn captcha_sync(custom_config: &CustomConfig, gt: &str, challenge: &str, referer: &str, captcha_type:usize) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("无法创建运行时: {}", e))?;
    
    rt.block_on(captcha(custom_config.clone(), gt, challenge, referer, captcha_type))
}


pub async fn handle_risk_verification(client: Arc<Client>,risk_param: TokenRiskParam,custom_config: &CustomConfig,csrf: &str) -> Result<(), String> {
    let risk_params_value = match &risk_param.risk_param {
        Some(value) => value,
        None => return Err("风控参数为空".to_string()),
    };
    log::debug!("风控参数: {:?}", risk_params_value);
    let url = "https://api.bilibili.com/x/gaia-vgate/v1/register";
    let response = client.post(url)
        .json(&json!(risk_params_value))
        .send()
        .await
        .map_err(|e| format!("发送风控请求失败: {}", e))?; 
    if !response.status().is_success() {
        return Err(format!("风控请求返回错误状态码: {}", response.status()));
    }
    
    let text = response.text().await
        .map_err(|e| format!("读取响应内容失败: {}", e))?;
    log::debug!("验证码请求响应: {}", text);
    
    let json_response: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    
    if json_response["code"].as_i64() != Some(0) {
        let message = json_response["message"].as_str().unwrap_or("未知错误");
        return Err(format!("风控请求失败: {} (code: {})", message, json_response["code"]));
    }
    
    
    let captcha_type = json_response["data"]["type"].as_str().unwrap_or("");
    
    match captcha_type {
        "geetest" => {
            log::info!("验证码类型: geetest");
            
            
            let gt = json_response["data"]["geetest"]["gt"].as_str().unwrap_or("");
            let challenge = json_response["data"]["geetest"]["challenge"].as_str().unwrap_or("");
            let token = json_response["data"]["geetest"]["token"].as_str().unwrap_or("");
            
            if gt.is_empty() || challenge.is_empty() || token.is_empty() {
                return Err("获取验证码参数失败".to_string());
            }
            
            
            let captcha_result = captcha(
                custom_config.clone(), 
                gt, 
                challenge, 
                "https://api.bilibili.com/x/gaia-vgate/v1/validate", 
                33 // 点选类型
            ).await?;
            
            
            let captcha_data: serde_json::Value = serde_json::from_str(&captcha_result)
                .map_err(|e| format!("解析验证码结果失败: {}", e))?;
            
            
            
            
            let params = json!({
                "buvid": risk_param.buvid.unwrap_or_default(),
                "csrf": csrf,
                "geetest_challenge": captcha_data["challenge"],
                "geetest_seccode": captcha_data["seccode"],
                "geetest_validate": captcha_data["validate"],
                "token": token
            });
            
            
            log::debug!("发送验证请求: {:?}", params);
            let validate_url = "https://api.bilibili.com/x/gaia-vgate/v1/validate";
            let validate_response = client.post(validate_url)
                .json(&params)
                .send()
                .await
                .map_err(|e| format!("发送验证请求失败: {}", e))?;
            
            if !validate_response.status().is_success() {
                return Err(format!("验证请求返回错误状态码: {}", validate_response.status()));
            }
            
            let validate_json = validate_response.json::<serde_json::Value>().await
                .map_err(|e| format!("解析验证响应失败: {}", e))?;
            
            
            if validate_json["code"].as_i64() != Some(0) {
                let message = validate_json["message"].as_str().unwrap_or("未知错误");
                return Err(format!("验证失败: {} (code: {})", message, validate_json["code"]));
            }
            
            let is_valid = validate_json["data"]["is_valid"].as_bool().unwrap_or(false);
            if !is_valid {
                return Err("验证未通过".to_string());
            }
            
            
            
            log::info!("验证码验证成功");
            Ok(())
        },
        _ => Err(format!("不支持的验证码类型: {}", captcha_type))
    }
}