use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomConfig {
    pub open_custom_ua: bool,    //是否开启自定义UA
    pub custom_ua: String,       //自定义UA
    pub captcha_mode: usize,     //验证码模式   //0:本地打码  1：ttocr
    pub ttocr_key: String,       //ttocr key
    pub preinput_phone1: String, //预填账号1手机号
    pub preinput_phone2: String, //预填账号2手机号
    pub enable_token_verify: bool, //是否启用token验证
    pub enable_random_interval: bool, //是否启用购票间隔时间随机性
    pub error_90002_action: usize, //遇到错误码90002时的处理方式 0:等待3秒后继续 1:直接结束
}

impl CustomConfig {
    pub fn new() -> Self {
        Self {
            open_custom_ua: true,
            custom_ua: String::from(
                "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Mobile Safari/537.36",
            ),
            captcha_mode: 0,
            ttocr_key: String::new(),
            preinput_phone1: String::new(),
            preinput_phone2: String::new(),
            enable_token_verify: true, // 默认启用token验证
            enable_random_interval: true, // 默认启用购票间隔时间随机性
            error_90002_action: 0, // 默认等待3秒后继续
        }
    }
}