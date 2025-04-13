use std::str::FromStr;

use reqwest::header::HeaderValue;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Clone,Debug)]
pub struct TicketInfo {
    pub id: String,
    pub name: String,
    pub is_sale: usize,
    pub start_time: usize,
    pub end_time: usize,
    pub pick_seat: usize, //0:不选座 1:选座
    pub project_type: usize, //未知作用，bw2024是type1
    pub express_fee: usize, //快递费
    pub sale_begin: usize, //开售时间
    pub sale_end: usize, //截止时间
    pub count_down: usize, //倒计时
    pub screen_list: Vec<ScreenInfo>, //场次列表
    pub sale_flag_number: usize, //售票标志位
    pub sale_flag: String, //售票状态
    pub is_free: bool,
    pub performance_desc: Option<DescribeList>, //基础信息


}

#[derive(Clone,Debug)]
pub struct ScreenInfo {
    pub sale_flag: SaleFlag,
    pub id: usize,
    pub start_time: usize,
    pub name: String,
    pub ticket_type: usize,
    pub screen_type: usize,
    pub delivery_type: usize,
    pub pick_seat: usize,
    pub ticket_list: Vec<ScreenTicketInfo>, //当日票种类列表
    pub clickable: bool, //是否可点（可售）
    pub sale_end: usize, //截止时间
    pub sale_start: usize, //开售时间
    pub sale_flag_number: usize, //售票标志位
    pub show_date: String, //展示信息
}

#[derive(Clone,Debug)]
pub struct SaleFlag{
    pub number: usize, //售票标志位
    pub display_name: String, //售票状态
}

#[derive(Clone,Debug)]
pub struct ScreenTicketInfo{
    pub saleStart : usize, //开售时间(时间戳)   eg：1720260000
    pub saleEnd : usize, //截止时间(时间戳)
    pub id: usize, //票种id
    pub project_id: usize, //项目id
    pub price: usize, //票价(分)
    pub desc: String, //票种描述
    pub sale_start: String, //开售时间（字符串）    eg:2024-07-06 18:00:00
    pub sale_end: String, //截止时间（字符串）
    pub r#type: usize, //类型 关键词替换，对应”type“
    pub sale_type: usize, //销售状态
    pub is_sale: usize, //是否销售？0是1否
    pub num: usize, //数量
    pub sale_flag: SaleFlag, //售票状态
    pub clickable: bool, //是否可点（可售）
    pub sale_flag_number: usize, //售票标志位
    pub screen_name: String, //场次名称


}

#[derive(Clone,Debug)]
pub struct DescribeList{
    pub r#type: u8,  // 使用 r# 前缀处理 Rust 关键字
    pub list: Vec<ModuleItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleItem {
    pub module: String,
    
    // details 可能是字符串或数组，使用 serde_json::Value 处理多态
    #[serde(default)]
    pub details: Value,
    
    // 可选字段
    #[serde(default)]
    pub module_name: Option<String>,
}

// 为 base_info 模块中的详情项创建结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseInfoItem {
    pub title: String,
    pub content: String,
}
