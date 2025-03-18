

//推送token
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
pub struct SmtpConfig{
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub smtp_to: String,
    }