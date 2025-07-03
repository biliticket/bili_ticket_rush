use std::sync::mpsc;
use std::thread;

// 登录状态枚举
#[derive(Debug, Clone)]
pub enum SeleniumLoginStatus {
    NotStarted,
    Connecting,
    WaitingForLogin,
    LoggedIn(String), // 成功获取的 cookie
    Failed(String),   // 错误信息
}

pub struct SeleniumLogin {
    status: SeleniumLoginStatus,
    progress: f32,
    cookie: Option<String>,
}

impl SeleniumLogin {
    pub fn new() -> Self {
        Self {
            status: SeleniumLoginStatus::NotStarted,
            progress: 0.0,
            cookie: None,
        }
    }

    // 启动浏览器登录过程
    pub fn start_login(&mut self) -> Result<(), String> {
        self.status = SeleniumLoginStatus::Connecting;
        self.progress = 0.1;

        // 创建通道用于接收异步结果
        let (tx, rx) = mpsc::channel();

        // 在后台线程执行浏览器登录
        thread::spawn(move || {
            // 创建运行时
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(login_and_get_cookie());

            match result {
                Ok(cookie) => {
                    let _ = tx.send(SeleniumLoginStatus::LoggedIn(cookie));
                }
                Err(e) => {
                    let _ = tx.send(SeleniumLoginStatus::Failed(e.to_string()));
                }
            }
        });

        // 保存接收器以便稍后检查结果
        self.cookie = None;

        Ok(())
    }

    // 检查登录状态 - 在UI更新循环中调用
    pub fn check_status(&mut self, rx: &mpsc::Receiver<SeleniumLoginStatus>) -> bool {
        // 非阻塞检查是否有新状态
        if let Ok(status) = rx.try_recv() {
            self.status = status.clone();

            // 如果成功获取cookie
            if let SeleniumLoginStatus::LoggedIn(cookie) = status {
                self.cookie = Some(cookie);
                self.progress = 1.0;
                return true;
            }
        }

        // 更新进度
        match self.status {
            SeleniumLoginStatus::Connecting => {
                if self.progress < 0.2 {
                    self.progress += 0.01;
                }
            }
            SeleniumLoginStatus::WaitingForLogin => {
                if self.progress < 0.9 {
                    self.progress += 0.001;
                }
            }
            _ => {}
        }

        false
    }

    // 获取当前cookie
    pub fn take_cookie(&mut self) -> Option<String> {
        self.cookie.take()
    }
}

// 原有的登录逻辑保持不变
async fn login_and_get_cookie() -> Result<String, Box<dyn std::error::Error>> {
    // 实现保持不变...
    // ...
    Ok("cookie_value".to_string())
}

// 检查WebDriver是否可用
pub fn is_webdriver_available() -> bool {
    // 简单检测本地是否运行了WebDriver
    match std::net::TcpStream::connect("127.0.0.1:4444") {
        Ok(_) => true,
        Err(_) => false,
    }
}
