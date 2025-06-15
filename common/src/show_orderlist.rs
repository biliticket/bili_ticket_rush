use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderResponse {
    pub errno: i32,
    pub errtag: i32,
    pub msg: String,
    pub data: OrderData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderData {
    pub total: i32,
    pub list: Vec<Order>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Order {
    pub order_id: String,
    pub order_type: i32,
    pub item_id: i64,
    pub item_info: ItemInfo,
    pub total_money: i64,
    pub count: i32,
    pub pay_money: i64,
    pub pay_channel: Option<String>,
    pub status: i32,
    pub sub_status: i32,
    pub ctime: String,
    pub img: ImageInfo,
    pub sub_status_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemInfo {
    pub name: String,
    pub image: Option<String>,
    pub screen_id: String,
    pub screen_name: String,
    pub screen_start_time: String,
    pub screen_end_time: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageInfo {
    pub url: String,
}
