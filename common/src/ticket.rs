use std::str::FromStr;

use reqwest::header::HeaderValue;
use reqwest::{header, Client};

use crate::account::Account;
use crate::push::PushConfig;
use crate::utility::CustomConfig;

#[derive(Debug, Clone)]
pub struct BilibiliTicket{
    pub method : u8,
    pub ua : String,
    pub config: CustomConfig,
    pub account: Account,
    pub push_self : PushConfig,
    pub status_delay : usize,
    pub captcha_use_type: usize,    //选择的验证码方式
    pub session: Option<reqwest::Client>,

    //抢票相关
    pub project_id: String,
    pub screen_id: String,

}

impl BilibiliTicket{
    pub fn new(
        method: &u8,
        ua: &String,
        config: &CustomConfig,
        account: &Account,
        push_self: &PushConfig,
        status_delay: &usize,
        project_id : &str,


    ) -> Self{
        let mut headers = header::HeaderMap::new();
        match HeaderValue::from_str(&account.cookie){
            Ok(ck_value) => {
                headers.insert(header::COOKIE, ck_value);
                match HeaderValue::from_str(ua){
                    Ok(ua_value) => {
                        headers.insert(header::USER_AGENT,ua_value);
                    }
                    Err(e) => {
                        log::error!("client插入ua失败！原因：{}",e);
                    }
                }
                
            }
            Err(e) => {
                log::error!("cookie设置失败！原因：{:?}",e);
            }

        }
        

        let client = match Client::builder()
                                    .cookie_store(true)
                                    .user_agent(ua)
                                    .default_headers(headers)
                                    
                                    .build(){
                                        Ok(client) => client,
                                        Err(e) => {
                                            log::error!("初始化client失败！，原因：{:?}",e);
                                            Client::new()
                                        }
                                    };
        let captcha_type = config.captcha_mode;      
           
        let new = Self{
            method: method.clone(),
            ua: ua.clone(),
            config: config.clone(),
            account: account.clone(),
            push_self: push_self.clone(),
            status_delay: *status_delay,
            captcha_use_type: captcha_type,
            session: Some(client),
            project_id: project_id.to_string(),
            screen_id: String::new(),

        };
        log::debug!("新建抢票对象：{:?}",new);
        new

    }

}