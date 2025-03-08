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

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "等待中"),
            TaskStatus::Running => write!(f, "运行中"),
            TaskStatus::Completed(true) => write!(f, "已完成"),
            TaskStatus::Completed(false) => write!(f, "完成但未成功"),
            TaskStatus::Failed(err) => write!(f, "失败: {}", err),
        }
    }
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
pub struct TicketTask {
    pub task_id: String,
    pub account_id: String,
    pub ticket_id: String,
    pub status: TaskStatus,
    pub start_time: Option<Instant>,
    pub result: Option<TicketResult>,
}

// 抢票请求参数
pub struct TicketRequest {
    pub ticket_id: String,
    pub account_id: String,
    // 其他请求参数...
}

// 任务管理器接口
pub trait TaskManager: Send + 'static {
    // 创建新的任务管理器
    fn new() -> Self where Self: Sized;
    
    // 提交抢票任务
    fn submit_task(&mut self, request: TicketRequest) -> Result<String, String>;
    
    // 获取可用结果 (非阻塞)
    fn get_results(&mut self) -> Vec<TaskResult>;
    
    // 取消任务
    fn cancel_task(&mut self, task_id: &str) -> Result<(), String>;
    
    // 获取任务状态
    fn get_task_status(&self, task_id: &str) -> Option<TaskStatus>;
    
    // 关闭任务管理器
    fn shutdown(&mut self);
}

// 任务结果
pub struct TaskResult {
    pub task_id: String,
    pub account_id: String,
    pub result: Result<TicketResult, String>,
}