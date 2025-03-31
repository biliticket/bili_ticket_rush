use serde::{Serialize, Deserialize};

//推送token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PushConfig{
    pub enabled: bool,
    pub bark_token: String,
    pub pushplus_token: String,
    pub fangtang_token: String,
    pub dingtalk_token: String,
    pub wechat_token: String,
    pub smtp_config: SmtpConfig,

}

//邮箱配置(属于pushconfig)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtpConfig{
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub smtp_to: String,
    } 

impl PushConfig{
    pub fn new() -> Self{
        Self{
            enabled: false,
            bark_token: String::new(),
            pushplus_token: String::new(),
            fangtang_token: String::new(),
            dingtalk_token: String::new(),
            wechat_token: String::new(),
            smtp_config: SmtpConfig::new(),
        }
    }
    
}

impl SmtpConfig{
    pub fn new() -> Self{
        Self{
            smtp_server: String::new(),
            smtp_port: String::new(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_from: String::new(),
            smtp_to: String::new(),
        }
    }
    
}