use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use common::taskmanager::{
    TaskManager, TaskStatus, TaskRequest, TicketRequest, QrCodeLoginRequest,
    TaskResult, TaskTicketResult, TaskQrCodeLoginResult, 
    TicketTask, QrCodeLoginTask, TicketResult, Task
};
use crate::api::{*};


pub struct TaskManagerImpl {
    task_sender: mpsc::Sender<TaskMessage>,
    result_receiver: mpsc::Receiver<TaskResult>,
    running_tasks: HashMap<String, Task>, // 使用 Task 枚举
    runtime: Arc<Runtime>,
    _worker_thread: Option<thread::JoinHandle<()>>,
}

enum TaskMessage {
    SubmitTask(TaskRequest),
    CancelTask(String),
    Shutdown,
}

impl TaskManager for TaskManagerImpl {
    fn new() -> Self {
        // 创建通道
        let (task_tx, mut task_rx) = mpsc::channel(100);
        let (result_tx, result_rx) = mpsc::channel(100);
        
        // 创建tokio运行时
        let runtime = Arc::new(Runtime::new().unwrap());
        let rt = runtime.clone();
        
        // 启动工作线程
        let worker = thread::spawn(move || {
            rt.block_on(async {
                while let Some(msg) = task_rx.recv().await {
                    match msg {
                        TaskMessage::SubmitTask(request) => {
                            let task_id = uuid::Uuid::new_v4().to_string();
                            let result_tx = result_tx.clone();
                            
                            // 根据任务类型处理
                            match request {
                                TaskRequest::TicketRequest(ticket_req) => {
                                    // 注意: 从 ticket_req 获取 account_id (不是从request)
                                    let account_id = ticket_req.account_id.clone();
                                    
                                    tokio::spawn(async move {
                                        // 传递 &ticket_req 而不是 &request
                                        let result = match perform_ticket_grab(&ticket_req).await {
                                            Ok(ticket_result) => Ok(ticket_result),
                                            Err(e) => Err(e.to_string()),
                                        };
                                        
                                        log::info!("任务完成 ID: {}, 结果: {}", 
                                            task_id, 
                                            if let Ok(ref r) = result { "成功" } else { "失败" }
                                        );
                                        
                                        // 使用正确的 TaskResult 枚举变体
                                        let task_result = TaskResult::TicketResult(TaskTicketResult {
                                            task_id,
                                            account_id,
                                            result,
                                        });
                                        
                                        let _ = result_tx.send(task_result).await;
                                    });
                                },
                                TaskRequest::QrCodeLoginRequest(qrcode_req) => {
                                    tokio::spawn(async move {
                                        // 二维码登录逻辑
                                        let status = poll_qrcode_login(&qrcode_req.qrcode_key,qrcode_req.user_agent.as_deref()).await;
                                        
                                        let (cookie, error) = match &status {
                                            common::login::QrCodeLoginStatus::Success(cookie) => 
                                                (Some(cookie.clone()), None),
                                            common::login::QrCodeLoginStatus::Failed(err) => 
                                                (None, Some(err.clone())),
                                            _ => (None, None)
                                        };
                                        
                                        // 创建正确的结果类型
                                        let task_result = TaskResult::QrCodeLoginResult(TaskQrCodeLoginResult {
                                            task_id,
                                            status,
                                            cookie,
                                            error,
                                        });
                                        
                                        let _ = result_tx.send(task_result).await;
                                    });
                                }
                            }
                        },
                        TaskMessage::CancelTask(_task_id) => {
                            // 取消任务逻辑
                        },
                        TaskMessage::Shutdown => break,
                    }
                }
            });
        });
        
        Self {
            task_sender: task_tx,
            result_receiver: result_rx,
            running_tasks: HashMap::new(),
            runtime: runtime,
            _worker_thread: Some(worker),
        }
    }
    
    fn submit_task(&mut self, request: TaskRequest) -> Result<String, String> {
        // 生成任务ID
        let task_id = uuid::Uuid::new_v4().to_string();
        
        // 根据请求类型创建相应的任务
        match &request {
            TaskRequest::TicketRequest(ticket_req) => {
                log::info!("提交票务任务 ID: {}, 票ID: {}", task_id, ticket_req.ticket_id);
                // 创建票务任务
                let task = TicketTask {
                    task_id: task_id.clone(),
                    account_id: ticket_req.account_id.clone(),
                    ticket_id: ticket_req.ticket_id.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                    result: None,
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::TicketTask(task));
            },
            TaskRequest::QrCodeLoginRequest(qrcode_req) => {
                log::info!("提交二维码登录任务 ID: {}", task_id);
                // 创建二维码登录任务
                let task = QrCodeLoginTask {
                    task_id: task_id.clone(),
                    qrcode_key: qrcode_req.qrcode_key.clone(),
                    qrcode_url: qrcode_req.qrcode_url.clone(),
                    status: TaskStatus::Pending,
                    start_time: Some(std::time::Instant::now()),
                };
                
                // 保存任务
                self.running_tasks.insert(task_id.clone(), Task::QrCodeLoginTask(task));
            }
        }
        
        // 发送任务
        if let Err(e) = self.task_sender.blocking_send(TaskMessage::SubmitTask(request)) {
            return Err(format!("无法提交任务: {}", e));
        }
        
        Ok(task_id)
    }
    
    fn get_results(&mut self) -> Vec<TaskResult> {
        let mut results = Vec::new();
        
        // 非阻塞方式获取所有可用结果
        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }
        
        results
    }
    
    fn cancel_task(&mut self, task_id: &str) -> Result<(), String> {
        if !self.running_tasks.contains_key(task_id) {
            return Err("任务不存在".to_string());
        }
        
        if let Err(e) = self.task_sender.blocking_send(TaskMessage::CancelTask(task_id.to_owned())) {
            return Err(format!("无法取消任务: {}", e));
        }
        
        Ok(())
    }
    
    fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        if let Some(task) = self.running_tasks.get(task_id) {
            match task {
                Task::TicketTask(t) => Some(t.status.clone()),
                Task::QrCodeLoginTask(t) => Some(t.status.clone()),
            }
        } else {
            None
        }
    }
    
    fn shutdown(&mut self) {
        let _ = self.task_sender.blocking_send(TaskMessage::Shutdown);
        if let Some(handle) = self._worker_thread.take() {
            let _ = handle.join();
        }
    }
}

