pub mod taskmanager;
pub mod record_log;
pub mod account;
pub mod utils;
pub mod push;
pub mod utility;
pub mod login;
pub mod http_utils;
pub mod captcha;
pub mod show_orderlist;
pub mod ticket;
// 重导出日志收集器
pub use record_log::LOG_COLLECTOR;
pub use record_log::init as init_logger;

#[derive(Clone)]
pub struct TicketInfo {
    pub id: String,
    pub name: String,
    pub price: f64,
}
