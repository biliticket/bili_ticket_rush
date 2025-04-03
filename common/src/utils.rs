use std::fs;
use std::io;
use std::ops::{Index, IndexMut};
use serde_json::{Value, json, Map};
use crate::account::Account;
use crate::push::PushConfig;
use crate::utility::CustomConfig;


#[derive(Clone,Debug)]
pub struct Config{
    data: Value,
}

impl Config{
    pub fn load_config() -> io::Result<Self>{
        let content = fs::read_to_string("./config.json")?;
        let data = serde_json::from_str(&content)?;
        Ok(Self{data})

    }

    pub fn new() -> Self{
        let data = json!({});
        Self{data}
    }

    pub fn save_config(&self) -> io::Result<()>{        //后续上加密
        let json_str= serde_json::to_string_pretty(&self.data)?;
        fs::write("./config.json",json_str)
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