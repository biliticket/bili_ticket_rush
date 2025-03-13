use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use common::taskmanager::{TaskManager, TaskResult, TaskStatus, TicketRequest, TicketTask, TicketResult};
use crate::api::{*};


pub struct TaskManagerImpl {
    task_sender: mpsc::Sender<TaskMessage>,
    result_receiver: mpsc::Receiver<TaskResult>,
    running_tasks: HashMap<String, TicketTask>,
    runtime: Arc<Runtime>,
    _worker_thread: Option<thread::JoinHandle<()>>,
}

enum TaskMessage {
    SubmitTask(TicketRequest),
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
                            let account_id = request.account_id.clone();
                            
                            // 为每个请求创建异步任务
                            tokio::spawn(async move {
                                // 这里是实际业务逻辑
                                let result = match perform_ticket_grab(&request).await {
                                    Ok(ticket_result) => Ok(ticket_result),
                                    Err(e) => Err(e.to_string()),
                                };
                                log::info! (" 任务完成 ID: {}, 结果: {}", 
             
             task_id, 
             
             if let Ok(ref r) = result { "成功" } else { "失败" });
                                // 发送结果
                                let _ = result_tx.send(TaskResult {
                                    task_id,
                                    account_id,
                                    result,
                                }).await;
                            });
                        },
                        TaskMessage::CancelTask(_task_id) => {
                            // 取消任务逻辑
                            // 实际实现需要保存JoinHandle并使用abort()
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
    
    fn submit_task(&mut self, request: TicketRequest) -> Result<String, String> {
        // 生成任务ID
        let task_id = uuid::Uuid::new_v4().to_string();
        
        log::info!("提交任务 ID: {}, 票ID: {}", task_id, request.ticket_id);
        // 创建任务对象
        let task = TicketTask {
            task_id: task_id.clone(),
            account_id: request.account_id.clone(),
            ticket_id: request.ticket_id.clone(),
            status: TaskStatus::Pending,
            start_time: Some(std::time::Instant::now()),
            result: None,
        };
        
        // 保存任务
        self.running_tasks.insert(task_id.clone(), task);
        
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
        self.running_tasks.get(task_id).map(|task| task.status.clone())
    }
    
    fn shutdown(&mut self) {
        let _ = self.task_sender.blocking_send(TaskMessage::Shutdown);
        if let Some(handle) = self._worker_thread.take() {
            let _ = handle.join();
        }
    }
}

