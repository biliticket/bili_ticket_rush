use serde_json::{Value, json, Map};


#[derive(Clone)]
pub struct CustomConfig{
    pub open_custom_ua: bool, //是否开启自定义UA
    pub custom_ua: String,      //自定义UA
    pub chapcha_mode: usize,     //验证码模式   //0:本地打码  1：ttocr
    pub ttocr_key: String,      //ttocr key
    pub preinput_phone: String, //预填手机号
    

}