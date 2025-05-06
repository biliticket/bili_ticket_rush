use std::{fs, process};
use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use serde_json::{Value, json, Map};
use crate::account::Account;
use crate::cookie_manager::CookieManager;
use crate::http_utils::request_get_sync;
use crate::push::PushConfig;
use crate::utility::CustomConfig;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use aes::Aes128;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::path::Path;
use std::time::{SystemTime, Duration};
use fs2::FileExt;

#[derive(Clone,Debug)]
pub struct Config{
    data: Value,
}

impl Config {
    pub fn delete_json_config() -> io::Result<()> {
        fs::remove_file("config.json")
    }
}

impl Config{
    pub fn load_config() -> io::Result<Self>{
        let raw_context = fs::read_to_string("./config")?;
        let content = raw_context.split("%").collect::<Vec<&str>>();
        // base64解码后解密
        let iv = BASE64.decode(content[0].trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let decoded = BASE64.decode(content[1].trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let decrypted = decrypt_data(iv, &decoded)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let plain_text = String::from_utf8(decrypted)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let data = serde_json::from_str(&plain_text)?;
        Ok(Self{data})

    }
    pub fn load_json_config() -> io::Result<Self>{
        let content = fs::read_to_string("./config.json")?;
        let data = serde_json::from_str(&content)?;
        Ok(Self{data})

    }

    pub fn new() -> Self{
        let data = json!({});
        Self{data}
    }

    pub fn save_config(&self) -> io::Result<()> {   //后续上加密
        let json_str = serde_json::to_string_pretty(&self.data)?;
        // 加密后base64编码
        let (iv,encrypted) = encrypt_data(json_str.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let encoded_iv = BASE64.encode(&iv);  
        let encoded_encrypted = BASE64.encode(&encrypted);
        fs::write("./config", encoded_iv+"%" + &*encoded_encrypted)
    }


    //添加账号
    pub fn add_account(&mut self, account: &Account) -> io::Result<()>{
        if !self["accounts"].is_array(){  //不存在则创建
            self["accounts"]= json!([]);
        }

        let account_json = serde_json::to_value(account)?;

        if let Value::Array(ref mut accounts)= self["accounts"]{
            accounts.push(account_json);
        }

        Ok(())
    }

    //加载账号
    pub fn load_accounts(&self) -> Result<Vec<Account>,serde_json::Error>{
        if self["accounts"].is_array(){
            let accounts_json = &self["accounts"];
            serde_json::from_value(accounts_json.clone())
        }
        else{
            Ok(Vec::new())
        }
    }

    //账号更新（Account更新后调用这个保存,uid唯一寻找标识）
    pub fn update_account(&mut self, account: &Account) ->io::Result<bool>{
        if !self["accounts"].is_array(){
            return Ok(false);
        }

        let account_json = serde_json::to_value(account)?;
        if let Value::Array(ref mut accounts) = self["accounts"]{
            for (index, acc) in accounts.iter_mut().enumerate() {
                if let Some(uid) = acc["uid"].as_i64(){
                    if uid == account.uid{
                        accounts[index] = account_json;
                        return Ok(true);
                    }
            }   }
        }
        Ok(false)

    }

    //删除账号，传uid
        pub fn delete_account(&mut self, uid: i64) ->bool{
        if !self["accounts"].is_array(){
            return false;
        }

        let mut remove_flag = false;
        if let Value::Array(ref mut accounts  )= self["accounts"]{
            let old_len = accounts.len();
            accounts.retain(|acc|{
                if let Some(account_uid) = acc["uid"].as_i64(){
                    account_uid != uid
                }
                else{
                    true
                }
            });
            remove_flag = accounts.len() != old_len;
        }
        match save_config(self, None, None, None){
            Ok(_) => {
                log::info!("删除账号成功");
            },
            Err(e) => {
                log::error!("删除账号失败: {}", e);
            }
        }
        remove_flag
    }

    pub fn load_all_accounts() -> Vec<Account> {
        match Self::load_config() {
            Ok(config) => {
                match config.load_accounts() {
                    Ok(accounts) => accounts,
                    Err(e) => {
                        log::error!("加载账号失败: {}", e);
                        Vec::new()
                    }
                }
            },
            Err(e) => {
                log::error!("加载配置文件失败: {}", e);
                Vec::new()
            }
        }
    }

}

impl Index<&str> for Config{
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output{

        match self.data.get(key){
            Some(value) => value,
            None => &Value::Null,
        }

    }
}

// 实现索引修改
impl IndexMut<&str> for Config {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        if let Value::Object(ref mut map) = self.data {
            map.entry(key.to_string()).or_insert(Value::Null)
        } else {
            // 如果当前不是对象，将其转换为对象
            let mut map = Map::new();
            map.insert(key.to_string(), Value::Null);
            self.data = Value::Object(map);

            if let Value::Object(ref mut map) = self.data {
                map.get_mut(key).unwrap()
            } else {
                unreachable!() // 理论上不可能到达这里
            }
        }
    }
}

pub fn save_config(config: &mut Config, push_config: Option<&PushConfig>, custon_config: Option<&CustomConfig>, account: Option<Account>) -> Result<bool, String> {
    if let Some(push_config) = push_config {
        config["push_config"] = serde_json::to_value(push_config).unwrap();
    }
    if let Some(custon_config) = custon_config {
        config["custom_config"] = serde_json::to_value(custon_config).unwrap();
    }
    if let Some(account) = account {
        config.add_account(&account).unwrap();
    }


    match config.save_config(){
        Ok(_) => {
            log::info!("配置文件保存成功");
            Ok(true)
        },
        Err(e) => {
            log::error!("配置文件保存失败: {}", e);
            Err(e.to_string())
        }
    }

}
pub fn load_texture_from_path(ctx: &eframe::egui::Context, path: &str, name: &str) -> Option<eframe::egui::TextureHandle> {
    use std::io::Read;


    match File::open(path) {

        Ok(mut file) => {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() {
                match image::load_from_memory(&bytes) {
                    Ok(image) => {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();

                        Some(ctx.load_texture(
                            name,
                            eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                            Default::default()
                        ))
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}


fn write_bytes_to_file(file_path: &str, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(file_path)?; // 创建文件
    file.write_all(bytes)?; // 写入字节流
    file.flush()?; // 确保数据写入磁盘
    Ok(())
}

pub fn load_texture_from_url(ctx: &eframe::egui::Context, cookie_manager: Arc<CookieManager>, url: &String, name: &str) -> Option<eframe::egui::TextureHandle> {
    let rt = tokio::runtime::Runtime::new().unwrap();


    let bytes = rt.block_on(async {
        // 发送请求
        let resp = match cookie_manager.get(url).await.send().await {
            Ok(resp) => resp,
            Err(err) => {
                log::error!("HTTP请求失败: {}", err);
                return None;
            }
        };

        // 读取响应体
        match resp.bytes().await {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                log::error!("读取响应体失败: {}", err);
                None
            }
        }
    });


    let bytes = match bytes {
        Some(b) => b,
        None => return None,
    };

    // 处理图像数据
    match image::load_from_memory(&bytes) {
        Ok(image) => {
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            Some(ctx.load_texture(
                name,
                eframe::egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                Default::default()
            ))
        }
        Err(err) => {
            log::warn!("加载图片至内存失败: {}，url:{}", err, url);
            None
        }
    }
}


fn gen_machine_id_bytes_128b()->Vec<u8> {
    let id: String = machine_uid::get().unwrap();
    println!("{}", id);
    id[..16].as_bytes().to_vec()

}
// 加密函数
fn encrypt_data(data: &[u8]) -> Result<(Vec<u8>,Vec<u8>), block_modes::BlockModeError> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let mut iv = [0u8; 16];
    rand::thread_rng()
        .fill(&mut iv[..]); // 填充 16 字节的随机数据
    let cipher = Aes128Cbc::new_from_slices(&gen_machine_id_bytes_128b(), &iv)
        .map_err(|_| block_modes::BlockModeError)?; // 将 InvalidKeyIvLength 转换为 BlockModeError

    Ok((iv.to_vec(), cipher.encrypt_vec(data)))
}

fn decrypt_data(iv:Vec<u8>,encrypted: &[u8]) -> Result<Vec<u8>, block_modes::BlockModeError> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let cipher = Aes128Cbc::new_from_slices(&gen_machine_id_bytes_128b(), &iv)
        .map_err(|_| block_modes::BlockModeError)?; // 将 InvalidKeyIvLength 转换为 BlockModeError

    cipher.decrypt_vec(encrypted)
}

// 单例锁实现，防止程序多开
pub struct SingleInstanceLock {
    lock_file_path: String,
    lock_file: Option<File>,
}

impl SingleInstanceLock {
    pub fn new(app_name: &str) -> Self {
        let lock_file = format!("{}_{}.lock", app_name, whoami::username());
        let temp_dir = std::env::temp_dir();
        let lock_path = temp_dir.join(lock_file);
        
        Self {
            lock_file_path: lock_path.to_string_lossy().to_string(),
            lock_file: None,
        }
    }
    
    // 尝试获取锁，如果成功返回true，如果程序已运行返回false
    pub fn try_lock(&mut self) -> bool {
        // log::debug!("检查程序是否已运行，锁文件路径: {}", self.lock_file_path);
        
        // 先检查锁文件是否存在
        if Path::new(&self.lock_file_path).exists() {
            // 检查现有锁文件里的PID是否有效
            if self.check_and_cleanup_stale_lock() {
                // 锁文件里的进程已经不存在，继续获取锁
                log::info!("发现孤立锁文件，已清理");
            } else {
                // 锁文件里的进程仍然存在，表示程序确实在运行
                log::warn!("程序已经在运行中！");
                return false;
            }
        }
        
        // 创建或打开锁文件
        let mut file = match File::options()
            .write(true)
            .create(true)
            .open(&self.lock_file_path) {
                Ok(file) => file,
                Err(e) => {
                    log::error!("无法创建/打开锁文件: {}", e);
                    return false;
                }
            };
            
        // 尝试获取独占锁，不阻塞
        match file.try_lock_exclusive() {
            Ok(_) => {
                // 成功获取锁
                log::info!("成功获取程序锁，确认为单实例运行");
                // 写入PID到锁文件便于调试和后续检测
                let pid = std::process::id();
                if let Err(e) = file.set_len(0)
                    .and_then(|_| file.write_all(format!("{}", pid).as_bytes()))
                    .and_then(|_| file.flush()) {
                    log::warn!("写入PID到锁文件失败: {}", e);
                }
                // 保存文件句柄，当对象销毁时会自动释放锁
                self.lock_file = Some(file);
                true
            },
            Err(_) => {
                // 获取锁失败，但尝试再次检查进程是否存在
                drop(file); // 先释放我们的文件句柄
                
                if self.check_and_cleanup_stale_lock() {
                    // 上一个进程已退出但锁文件未清理，递归再试一次
                    log::info!("检测到过期锁，重试获取锁");
                    return self.try_lock();
                }
                
                log::warn!("程序已经在运行中！无法获取文件锁");
                false
            }
        }
    }
    
    // 检查锁文件是否对应有效进程，如果无效则清理
    fn check_and_cleanup_stale_lock(&self) -> bool {
        // 读取锁文件中的PID
        let pid_str = match fs::read_to_string(&self.lock_file_path) {
            Ok(content) => content.trim().to_string(),
            Err(_) => return false, // 读取失败，当作锁有效处理
        };
        
        let pid = match pid_str.parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => return false, // 无法解析PID，当作锁有效处理
        };
        
        // 检查进程是否存在
        if !is_process_running(pid) {
            // 进程不存在，清理锁文件
            let _ = fs::remove_file(&self.lock_file_path);
            return true;
        }
        
        // 进程存在，锁有效
        false
    }
}

impl Drop for SingleInstanceLock {
    fn drop(&mut self) {
        // 明确释放文件锁
        if let Some(file) = self.lock_file.take() {
            // 尝试解锁文件
            let _ = file.unlock();
            // 文件会在这里关闭，锁也会自动释放
            drop(file);
            
            // 尝试删除锁文件
            if let Err(e) = fs::remove_file(&self.lock_file_path) {
                log::error!("无法删除锁文件: {}", e);
            } else {
                log::info!("已删除锁文件");
            }
        }
    }
}

// 检查程序是否可以运行，如果已有实例运行则退出
pub fn ensure_single_instance() -> bool {
    // 使用静态变量保存锁实例，确保锁在程序整个生命周期都存在
    use std::sync::Mutex;
    use std::sync::Once;
    use std::sync::OnceLock;
    
    static INSTANCE: OnceLock<Mutex<SingleInstanceLock>> = OnceLock::new();
    static INIT: Once = Once::new();
    
    let mut success = true;
    
    INIT.call_once(|| {
        let mut lock = SingleInstanceLock::new("bili_ticket_rush");
        let result = lock.try_lock();
        
        if !result {
            log::error!("程序已经在运行中，请勿重复启动！");
            eprintln!("程序已经在运行中，请勿重复启动！");
            success = false;
            std::thread::sleep(std::time::Duration::from_secs(2));
            std::process::exit(1);
        }
        
        // 初始化成功后，将锁存储在全局静态变量中
        INSTANCE.get_or_init(|| Mutex::new(lock));
        
        // 确保进程退出时清理锁文件
        std::panic::set_hook(Box::new(|_| {
            if let Some(lock) = INSTANCE.get() {
                if let Ok(mut guard) = lock.try_lock() {
                    // 显式释放锁
                    guard.lock_file = None;
                }
            }
            log::info!("程序异常退出，已释放锁");
        }));
    });
    
    success
}

// 检查指定PID的进程是否正在运行
#[cfg(target_os = "windows")]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;
    
    // 使用 tasklist 命令检查进程
    let output = Command::new("tasklist")
        .args(&["/NH", "/FI", &format!("PID eq {}", pid)])
        .output();
        
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            !stdout.contains("信息: 没有运行的任务匹配指定标准") && 
            !stdout.contains("No tasks") && 
            stdout.contains(&format!("{}", pid))
        },
        Err(_) => false, // 执行命令失败，假设进程不存在
    }
}

#[cfg(target_os = "linux")]
fn is_process_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

#[cfg(target_os = "macos")]
fn is_process_running(pid: u32) -> bool {
    use std::process::Command;
    
    // 使用 ps 命令检查进程
    let output = Command::new("ps")
        .args(&["-p", &format!("{}", pid)])
        .output();
        
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(&format!("{}", pid))
        },
        Err(_) => false, // 执行命令失败，假设进程不存在
    }
}

// 为不支持的平台提供默认实现
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn is_process_running(_pid: u32) -> bool {
    false // 不支持的平台，假设进程不存在
}
