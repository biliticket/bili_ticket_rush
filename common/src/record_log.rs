use std::sync::{Arc, Mutex};
use log::{Record, Level, Metadata, LevelFilter, SetLoggerError};
use once_cell::sync::Lazy;


//日志记录器
pub struct LogCollector{
    pub logs: Vec<String>,
}

impl LogCollector{
    pub fn new() -> Self{
        Self { logs: Vec::new() }
    }
    //添加日志
    pub fn add(&mut self, message: String){
        self.logs.push(message);
    }

    //获取日志
    pub fn get_logs(&mut self) -> Option<Vec<String>>{
        if self.logs.is_empty(){
            return None;
        }
        let logs = self.logs.clone();
        
        self.clear_logs();
        Some(logs)
    }

    //清空日志
    pub fn clear_logs(&mut self){
        self.logs.clear();
    }
}

pub static LOG_COLLECTOR: Lazy<Arc<Mutex<LogCollector>>> =   //?
    Lazy::new(|| Arc::new(Mutex::new(LogCollector::new())));


struct CollectorLogger;
impl log::Log for CollectorLogger{
    fn enabled(&self, metadata: &Metadata) -> bool{
        metadata.level() <= Level::Info
    }
    
    fn log(&self,record: &Record){
        if self.enabled(record.metadata()){
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_message = format!("[{}] {}: {}", 
                timestamp, record.level(), record.args());

            if let Ok(mut collector) = LOG_COLLECTOR.lock(){
                collector.add(log_message.clone());
            }

            println!("{}", log_message);
        }
    }

    fn flush(&self) {}

}

// 静态日志记录器
static LOGGER: CollectorLogger = CollectorLogger;

// 初始化日志系统
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}