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

#[derive(Clone,Debug)]
pub struct Config{
    data: Value,
}

impl Config{
    pub fn load_config() -> io::Result<Self>{
        let content = fs::read_to_string("./config.json")?;
        // base64解码后解密
        let decoded = BASE64.decode(content.trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let decrypted = decrypt_data(&decoded)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let plain_text = String::from_utf8(decrypted)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let data = serde_json::from_str(&plain_text)?;
        Ok(Self{data})

    }

    pub fn new() -> Self{
        let data = json!({});
        Self{data}
    }

    pub fn save_config(&self) -> io::Result<()> {   //后续上加密
        let json_str = serde_json::to_string_pretty(&self.data)?;
        // 加密后base64编码
        let encrypted = encrypt_data(json_str.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let encoded = BASE64.encode(&encrypted);
        fs::write("./config.json", encoded)
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

// 加密函数
fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, block_modes::BlockModeError> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let key = [0x42; 16]; // 16字节的密钥
    let iv = [0x24; 16]; // 16字节的IV
    let cipher = Aes128Cbc::new_from_slices(&key, &iv)
        .map_err(|_| block_modes::BlockModeError)?; // 将 InvalidKeyIvLength 转换为 BlockModeError
    
    Ok(cipher.encrypt_vec(data))
}

fn decrypt_data(encrypted: &[u8]) -> Result<Vec<u8>, block_modes::BlockModeError> {
    type Aes128Cbc = Cbc<Aes128, Pkcs7>;
    let key = [0x42; 16];
    let iv = [0x24; 16];
    let cipher = Aes128Cbc::new_from_slices(&key, &iv)
        .map_err(|_| block_modes::BlockModeError)?; // 将 InvalidKeyIvLength 转换为 BlockModeError
    
    cipher.decrypt_vec(encrypted)
}
