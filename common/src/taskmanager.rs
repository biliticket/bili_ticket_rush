use std::time::Instant;
use std::fmt;
use crate::TicketInfo;



// 任务状态枚举
#[derive(Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed(bool),
    Failed(String),
}

// 票务结果
#[derive(Clone)]
pub struct TicketResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub message: Option<String>,
    pub ticket_info: TicketInfo,
    pub timestamp: Instant,
}

// 任务信息
pub enum Task {
    TicketTask(TicketTask),
    QrCodeLoginTask(QrCodeLoginTask),
}

pub struct TicketTask {
    pub task_id: String,
    pub account_id: String,
    pub ticket_id: String,
    pub status: TaskStatus,
    pub start_time: Option<Instant>,
    pub result: Option<TicketResult>,
}

pub struct QrCodeLoginTask {
    pub task_id: String,
    pub qrcode_key: String,
    pub qrcode_url: String,
    pub status: TaskStatus,
    pub start_time: Option<Instant>,
    
}

// 任务请求枚举
pub enum TaskRequest {
    TicketRequest(TicketRequest),
    QrCodeLoginRequest(QrCodeLoginRequest),
}

pub struct TicketRequest {
    pub ticket_id: String,
    pub account_id: String,
    // 其他请求参数...
}

pub struct QrCodeLoginRequest {
    pub qrcode_key: String,
    pub qrcode_url: String,
    pub user_agent: Option<String>,
}

// 任务结果枚举
#[derive(Clone)]
pub enum TaskResult {
    TicketResult(TaskTicketResult),
    QrCodeLoginResult(TaskQrCodeLoginResult),
}

#[derive(Clone)]
pub struct TaskTicketResult {
    pub task_id: String,
    pub account_id: String,
    pub result: Result<TicketResult, String>,
}

#[derive(Clone)]
pub struct TaskQrCodeLoginResult {
    pub task_id: String,
    pub status: crate::login::QrCodeLoginStatus,
    pub cookie: Option<String>,
    pub error: Option<String>,
}

// 更新 TaskManager trait
pub trait TaskManager: Send + 'static {
    // 创建新的任务管理器
    fn new() -> Self where Self: Sized;
    
    // 提交任务
    fn submit_task(&mut self, request: TaskRequest) -> Result<String, String>;
    
    // 获取可用结果，返回 TaskResult 枚举
    fn get_results(&mut self) -> Vec<TaskResult>;
    
    // 取消任务
    fn cancel_task(&mut self, task_id: &str) -> Result<(), String>;

    // 获取任务状态
    fn get_task_status(&self, task_id: &str) -> Option<TaskStatus>;
     
     // 关闭任务管理器
    fn shutdown(&mut self);
}