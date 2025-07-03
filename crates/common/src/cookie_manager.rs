use crate::web_ck_obfuscated::*;
use cookie::Cookie;
use rand::seq::SliceRandom;
use reqwest::cookie::Jar;
use std::collections::HashMap;
use std::sync::{Arc, Mutex}; //?有用到吗
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone)]
pub struct AppData {
    pub brand: String,
    pub model: String,
    pub buvid: String,
}

#[derive(Debug, Clone)]
pub struct WebData {
    pub browser_version: String,
    pub os_version: String,
    pub ua: String,
    pub buvid3: String,
    pub buvid4: String,
    pub b_nut: String,
    pub buvid_fp: String,
    pub _uuid: String,
    pub bili_ticket: String,
    pub bili_ticket_expires: String,
    pub msource: String,
}

#[derive(Debug, Clone)]
pub struct CookieManager {
    pub client: Arc<reqwest::Client>,
    pub create_type: usize,
    app_data: Option<AppData>,
    pub web_data: Option<WebData>,
    pub cookies: CookiesData,
}

#[derive(Debug, Clone)]
pub struct CookiesData {
    pub cookies_map: Arc<Mutex<HashMap<String, String>>>,
    pub cookie_jar: Arc<Mutex<Jar>>,
}

impl CookiesData {
    pub fn insert(&self, key: String, value: String) {
        self.cookies_map
            .lock()
            .unwrap()
            .insert(key.clone(), value.clone());
        let cookie = Cookie::build(&key, value)
            .domain(".bilibili.com")
            .path("/")
            .finish();
        self.cookie_jar.lock().unwrap().add_cookie_str(
            &cookie.to_string(),
            &"https://bilibili.com".parse().unwrap(),
        );
    }

    pub fn clear(&self) {
        self.cookies_map.lock().unwrap().clear();
        *self.cookie_jar.lock().unwrap() = Jar::default();
    }
}

impl CookieManager {
    pub async fn new(
        original_cookie: &str,
        user_agent: Option<&str>,
        create_type: usize, //0：默认网页浏览器 1：app
    ) -> Self {
        let mut cookies = Self::parse_cookie_string(original_cookie);

        match create_type {
            0 => {
                //UA部分

                //浏览器
                let browser_version_list = vec![
                    //chrome
                    "Chrome/126.0.6478.55",
                    "Chrome/127.0.6520.0",
                    "Chrome/125.0.6422.61",
                    "Chrome/124.0.6367.60",
                    "Chrome/135.0.0.0",
                    //edge
                    "Chrome/126.0.6478.55 Edg/126.0.2578.55",
                    "Chrome/127.0.6520.0 Edg/127.0.2610.0",
                    "Chrome/125.0.6422.61 Edg/125.0.2535.51",
                    "Chrome/124.0.6367.60 Edg/124.0.2478.67",
                    "Chrome/135.0.0.0, Edg/135.0.0.0",
                ];
                let os_version_list = vec!["(Windows NT 10.0; Win64; x64)"];
                let browser_version = browser_version_list
                    .choose(&mut rand::thread_rng())
                    .map(|&s| s.to_string())
                    .unwrap_or_else(|| "Chrome/135.0.0.0".to_string());
                let os_version = os_version_list
                    .choose(&mut rand::thread_rng())
                    .map(|&s| s.to_string())
                    .unwrap_or_else(|| "(Windows NT 10.0; Win64; x64)".to_string());
                let ua = match user_agent {
                    Some(ua) => ua.to_string(),
                    None => {
                        format!(
                            "Mozilla/5.0 {} AppleWebKit/537.36 (KHTML, like Gecko) {} Safari/537.36",
                            os_version, browser_version
                        )
                    }
                };

                log::debug!("UA: {}", ua);

                let client_builder = reqwest::Client::builder().cookie_store(true);
                let client = client_builder
                    .user_agent(ua.clone())
                    .build()
                    .unwrap_or_default();

                //ck部分
                let (buvid3, buvid4, b_nut) = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_buvid3 = cookies_map.get("buvid3").cloned();
                    let existing_buvid4 = cookies_map.get("buvid4").cloned();
                    let existing_b_nut = cookies_map.get("b_nut").cloned();
                    drop(cookies_map);

                    if existing_buvid3.is_some()
                        && existing_buvid4.is_some()
                        && existing_b_nut.is_some()
                    {
                        (
                            existing_buvid3.unwrap(),
                            existing_buvid4.unwrap(),
                            existing_b_nut.unwrap(),
                        )
                    } else {
                        gen_buvid3and4(client.clone())
                            .await
                            .unwrap_or_else(|err| ("".to_string(), "".to_string(), "".to_string()))
                    }
                };
                log::debug!("buvid3: {}, buvid4: {}, b_nut: {}", buvid3, buvid4, b_nut);
                let fp = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_fp = cookies_map.get("buvid_fp").cloned();
                    drop(cookies_map);

                    if let Some(fp_value) = existing_fp {
                        fp_value
                    } else {
                        let new_fp = gen_fp();

                        new_fp
                    }
                };

                log::debug!("fp: {}", fp);
                let _uuid = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_uuid = cookies_map.get("_uuid").cloned();
                    drop(cookies_map);

                    if let Some(uuid_value) = existing_uuid {
                        log::debug!("使用现有 _uuid: {}", uuid_value);
                        uuid_value
                    } else {
                        let new_uuid = gen_uuid_infoc();
                        log::debug!("生成新的 _uuid: {}", new_uuid);
                        new_uuid
                    }
                };
                log::debug!("_uuid: {}", _uuid);
                let (bili_ticket, bili_ticket_expires) = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_ticket = cookies_map.get("bili_ticket").cloned();
                    let existing_expires = cookies_map.get("bili_ticket_expires").cloned();
                    drop(cookies_map);

                    if existing_ticket.is_some() && existing_expires.is_some() {
                        //验证过期时间
                        if let Some(expires_str) = &existing_expires {
                            if let Ok(expires_time) = expires_str.parse::<i64>() {
                                let current_time = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs()
                                    as i64;

                                // 未过期，使用已有的
                                if current_time < expires_time {
                                    log::debug!(
                                        "使用现有 bili_ticket (有效期至: {})",
                                        expires_time
                                    );
                                    (existing_ticket.unwrap(), existing_expires.unwrap()) // 删除了return关键字
                                } else {
                                    log::debug!("bili_ticket 已过期，重新生成");
                                    // 生成新的
                                    gen_ckbili_ticket(client.clone())
                                        .await
                                        .unwrap_or_else(|err| {
                                            log::error!("生成bili_ticket失败: {}", err);
                                            ("".to_string(), "".to_string())
                                        })
                                }
                            } else {
                                // 解析失败
                                gen_ckbili_ticket(client.clone())
                                    .await
                                    .unwrap_or_else(|err| {
                                        log::error!("生成bili_ticket失败: {}", err);
                                        ("".to_string(), "".to_string())
                                    })
                            }
                        } else {
                            //无过期时间
                            gen_ckbili_ticket(client.clone())
                                .await
                                .unwrap_or_else(|err| {
                                    log::error!("生成bili_ticket失败: {}", err);
                                    ("".to_string(), "".to_string())
                                })
                        }
                    } else {
                        log::debug!("生成新的 bili_ticket");
                        gen_ckbili_ticket(client.clone())
                            .await
                            .unwrap_or_else(|err| {
                                log::error!("生成bili_ticket失败: {}", err);
                                ("".to_string(), "".to_string())
                            })
                    }
                };
                let msourse = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_msource = cookies_map.get("msource").cloned();
                    drop(cookies_map);

                    if let Some(msource_value) = existing_msource {
                        msource_value
                    } else {
                        let new_msource = "bilibiliapp".to_string();

                        new_msource
                    }
                };
                let _01x96 = {
                    let cookies_map = cookies.cookies_map.lock().unwrap();
                    let existing_01x96 = cookies_map.get("deviceFingerprint").cloned();
                    drop(cookies_map);

                    if let Some(_01x25) = existing_01x96 {
                        log::debug!("使用现有 01x96: {}", _01x25);
                        _01x25
                    } else {
                        let new_01x96 = gen_01x88();
                        new_01x96
                    }
                };
                let _obf_key = unsafe {
                    std::str::from_utf8_unchecked(&[
                        100, 101, 118, 105, 99, 101, 70, 105, 110, 103, 101, 114, 112, 114, 105,
                        110, 116,
                    ])
                };
                cookies.insert("buvid3".to_string(), buvid3.clone());
                cookies.insert("buvid4".to_string(), buvid4.clone());
                cookies.insert("b_nut".to_string(), b_nut.clone());
                cookies.insert("buvid_fp".to_string(), fp.clone());
                cookies.insert("_uuid".to_string(), _uuid.clone());
                cookies.insert("bili_ticket".to_string(), bili_ticket.clone());
                cookies.insert(
                    "bili_ticket_expires".to_string(),
                    bili_ticket_expires.clone(),
                );
                cookies.insert("header_theme_version".to_string(), "CLOSE".to_string());
                cookies.insert("enable_web_push".to_string(), "DISABLE".to_string());
                cookies.insert("enable_feed_channel".to_string(), "ENABLE".to_string());
                cookies.insert("msource".to_string(), msourse.clone());
                cookies.insert(_obf_key.to_string(), _01x96.clone());
                log::debug!(
                    "buvid3: {}, buvid4: {}, b_nut: {}, fp: {}, _uuid: {}, bili_ticket: {}, bili_ticket_expires: {}",
                    buvid3,
                    buvid4,
                    b_nut,
                    fp,
                    _uuid,
                    bili_ticket,
                    bili_ticket_expires
                );

                let web_data = WebData {
                    browser_version: browser_version,
                    os_version: os_version,
                    ua: ua,
                    buvid3: buvid3,
                    buvid4: buvid4,
                    b_nut: b_nut,
                    buvid_fp: fp,
                    _uuid: _uuid,
                    bili_ticket: bili_ticket,
                    bili_ticket_expires: bili_ticket_expires,
                    msource: msourse,
                };
                Self {
                    client: Arc::new(client),
                    create_type: create_type,
                    app_data: None,
                    web_data: Some(web_data),
                    cookies: cookies,
                }
            }

            _ => {
                //默认浏览器
                log::warn!("创建类型错误");
                Self {
                    client: Arc::new(
                        reqwest::Client::builder()
                            .cookie_store(true)
                            .build()
                            .unwrap_or_default(),
                    ),
                    create_type: 0,
                    app_data: None,
                    web_data: None,
                    cookies: cookies,
                }
            }
        }
    }

    pub fn get_all_cookies(&self) -> String {
        let cookies_map = self.cookies.cookies_map.lock().unwrap();
        let mut cookie_str = String::new();
        for (key, value) in cookies_map.iter() {
            cookie_str.push_str(&format!("{}={}; ", key, value));
        }
        cookie_str
    }
    //解析cookie字符串
    //TODO：（ck登录待去多余字符）
    fn parse_cookie_string(cookie_str: &str) -> CookiesData {
        let mut map = HashMap::new();
        let cookie_jar = Arc::new(Mutex::new(reqwest::cookie::Jar::default()));
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(index) = cookie.find("=") {
                let (key, value) = cookie.split_at(index);
                if value.len() > 1 {
                    map.insert(key.to_string(), value[1..].to_string());
                    let cookie = Cookie::build(key, value[1..].to_string())
                        .domain(".bilibili.com")
                        .path("/")
                        .finish();
                    cookie_jar.lock().unwrap().add_cookie_str(
                        &cookie.to_string(),
                        &"https://bilibili.com".parse().unwrap(),
                    );
                }
            }
        }

        CookiesData {
            cookies_map: Arc::new(Mutex::new(map)),
            cookie_jar: cookie_jar,
        }
    }

    //现有client创建ck管理器 (已封进client的ck无法读取)
    pub fn from_client(client: Arc<reqwest::Client>, original_cookie: &str) -> Self {
        let cookies = Self::parse_cookie_string(original_cookie);
        Self {
            client: client,
            create_type: 0,
            app_data: None,
            web_data: None,
            cookies: cookies,
        }
    }

    //更新单个字段
    pub fn update_cookie(&self, key: &str, value: &str) {
        self.cookies.insert(key.to_string(), value.to_string());
        log::debug!("更新Cookie: {}={}", key, value);
    }

    //移除某个键对应的值
    pub fn remove_cookie(&self, key: &str) -> bool {
        let existed = self
            .cookies
            .cookies_map
            .lock()
            .unwrap()
            .remove(key)
            .is_some();
        if existed {
            let expire_cookie = Cookie::build(key, "")
                .domain(".bilibili.com")
                .path("/")
                .max_age(cookie::time::Duration::seconds(-1))
                .finish();
            self.cookies.cookie_jar.lock().unwrap().add_cookie_str(
                &expire_cookie.to_string(),
                &"https://bilibili.com".parse().unwrap(),
            );
            log::debug!("删除Cookie: {}", key);
        } else {
            log::debug!("Cookie不存在: {}", key);
        }
        existed
    }

    //更新大量ck
    pub fn update_cookies(&self, cookies_str: &str) {
        let new_cookies = Self::parse_cookie_string(cookies_str);

        for (key, value) in new_cookies.cookies_map.lock().unwrap().iter() {
            self.cookies.insert(key.clone(), value.clone());
        }
        log::debug!("批量更新Cookie: {}", cookies_str);
    }

    //清除所有ck
    pub fn clear_all_cookies(&self) {
        self.cookies.clear();
        log::debug!("清除所有Cookie");
    }

    //获取某个键的值
    pub fn get_cookie(&self, key: &str) -> Option<String> {
        let cookies = self.cookies.cookies_map.lock().unwrap();
        cookies.get(key).cloned()
    }

    // 发送 GET 请求
    pub async fn get(&self, url: &str) -> reqwest::RequestBuilder {
        let builder = self.client.get(url);
        self.prepare_request(builder)
    }

    // 发送 POST 请求
    pub async fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let builder = self.client.post(url);
        self.prepare_request(builder)
    }

    // 创建具有自定义标头的请求
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: HashMap<&str, &str>,
    ) -> reqwest::RequestBuilder {
        let mut builder = self.get(url).await;
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
        builder
    }

    pub async fn post_with_headers(
        &self,
        url: &str,
        headers: HashMap<&str, &str>,
    ) -> reqwest::RequestBuilder {
        let mut builder = self.post(url).await;
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
        builder
    }

    // 临时覆盖 UA
    pub async fn with_custom_ua(
        &self,
        builder: reqwest::RequestBuilder,
        ua: &str,
    ) -> reqwest::RequestBuilder {
        builder.header(reqwest::header::USER_AGENT, ua)
    }

    fn prepare_request(&self, mut builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // 1. 先添加所有 cookie - 覆盖内部 cookie_store 的值
        let cookies = self.cookies.cookies_map.lock().unwrap();

        if !cookies.is_empty() {
            let cookie_header = cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");

            builder = builder.header(reqwest::header::COOKIE, cookie_header);
        }

        // 2. 根据创建类型添加其他头
        let builder = match self.create_type {
            0 => {
                // Web 请求头
                if let Some(web_data) = &self.web_data {
                    builder
                        .header("User-Agent", &web_data.ua)
                        .header("Referer", "https://show.bilibili.com/")
                        .header("Origin", "https://show.bilibili.com")
                } else {
                    builder
                }
            }
            1 => {
                // App 请求头
                if let Some(app_data) = &self.app_data {
                    builder.header("x-bili-aurora-zone", "sh")
                    // 其他 app 相关头
                } else {
                    builder
                }
            }
            _ => builder,
        };

        builder
    }

    //处理响应中的 cookie
    pub async fn execute(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let response = request.send().await?;

        // 从响应中提取并更新 cookies
        let cookies = response.headers().get_all(reqwest::header::SET_COOKIE);
        for cookie_header in cookies {
            if let Ok(cookie_str) = cookie_header.to_str() {
                log::debug!("从响应中获取到 cookie: {}", cookie_str);
                self.update_cookies(cookie_str);
            }
        }

        Ok(response)
    }
}
