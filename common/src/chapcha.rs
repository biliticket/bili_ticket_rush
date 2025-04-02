/* use bili_ticket_gt_python::slide::Slide;
use bili_ticket_gt_python::click::Click;
use bili_ticket_gt_python::abstraction::{Api, GenerateW, Test};

 */
use reqwest::Client;
use serde_json::json;
use crate::utility::CustomConfig;

pub async fn chapcha(custom_config: CustomConfig, gt: &str, challenge: &str, referer: &str, chapcha_type:usize) -> Result<String, String> {
    // 0:本地打码  1：ttocr
    match custom_config.chapcha_mode {
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
                "itemid": chapcha_type,//33对应三代点字 32对应三代滑块
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

pub fn chapcha_sync(custom_config: &CustomConfig, gt: &str, challenge: &str, referer: &str, chapcha_type:usize) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("无法创建运行时: {}", e))?;
    
    rt.block_on(chapcha(custom_config.clone(), gt, challenge, referer, chapcha_type))
}

/* pub fn test(){
    // 使用 Slide
    let mut click = Click::default();
    
    // 例如:
    match click.register_test("https://passport.bilibili.com/x/passport-login/captcha?source=main_web") {
        Ok((gt, challenge)) => {
            println!("注册成功: gt={}, challenge={}", gt, challenge);
        },
        Err(e) => {
            println!("错误: {}", e);
        }
    }
} */