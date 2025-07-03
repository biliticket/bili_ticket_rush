use common::cookie_manager::CookieManager;
use common::show_orderlist::*;
use serde_json;
use std::sync::Arc;

pub async fn get_orderlist(cookie_manager: Arc<CookieManager>) -> Result<OrderResponse, String> {
    match cookie_manager
        .get("https://show.bilibili.com/api/ticket/ordercenter/ticketList?page=0&page_size=10")
        .await
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                match tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(resp.text())
                }) {
                    Ok(text) => {
                        log::debug!("获取全部订单：{}", text);
                        match serde_json::from_str::<OrderResponse>(&text) {
                            Ok(order_resp) => {
                                return Ok(order_resp);
                            }
                            Err(e) => {
                                log::error!("获取全部订单json解析失败：{}", e);
                                return Err(format!("获取全部订单json解析失败：{}", e));
                            }
                        }
                    }
                    Err(e) => {
                        //log::error!("获取data失败： {}",e);
                        return Err(format!("获取data失败： {}", e));
                    }
                }
            } else {
                // log::error!("获取订单不期待响应：{}", resp.status());
                return Err(format!("获取订单不期待响应：{}", resp.status()));
            }
        }
        Err(err) => {
            //log::error!("请求失败: {}", err);
            return Err(err.to_string());
        }
    };
}
