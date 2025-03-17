use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct Account{
    pub uid: i64,  //UID
    pub name: String,   //昵称
    pub level: String,
    pub cookie: String, //cookie
    pub csrf : String,  //csrf
    pub is_logged: bool,    //是否登录
    pub account_status: String,  //账号状态
    pub vip_label: String, //大会员，对应/nav请求中data['vip_label']['text']
    pub is_active: bool, //该账号是否启动抢票
}