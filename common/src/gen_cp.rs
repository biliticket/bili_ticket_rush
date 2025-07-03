use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::{Rng, thread_rng};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

//感谢https://github.com/mikumifa/biliTickerBuy/pull/726/commits/0ff6218da458c41df89956384b8f192c7e7eae20
// 提供的CToken生成代码
//本代码依据上述代码

pub struct CTokenGenerator {
    touch_event: i32,
    isibility_change: i32,
    page_unload: i32,
    timer: i32,
    time_difference: i32,
    scroll_x: i32,
    scroll_y: i32,
    inner_width: i32,
    inner_height: i32,
    outer_width: i32,
    outer_height: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    screen_avail_width: i32,
    pub ticket_collection_t: i64,
    pub time_offset: i64,
    pub stay_time: i32,
}

impl CTokenGenerator {
    pub fn new(ticket_collection_t: i64, time_offset: i64, stay_time: i32) -> Self {
        CTokenGenerator {
            touch_event: 0,
            isibility_change: 0,
            page_unload: 0,
            timer: 0,
            time_difference: 0,
            scroll_x: 0,
            scroll_y: 0,
            inner_width: 0,
            inner_height: 0,
            outer_width: 0,
            outer_height: 0,
            screen_x: 0,
            screen_y: 0,
            screen_width: 0,
            screen_height: 0,
            screen_avail_width: 0,
            ticket_collection_t,
            time_offset,
            stay_time,
        }
    }

    fn encode(&self) -> String {
        let mut buffer = [0u8; 16];
        let mut data_mapping = HashMap::new();

        data_mapping.insert(0, (self.touch_event, 1));
        data_mapping.insert(1, (self.scroll_x, 1));
        data_mapping.insert(2, (self.isibility_change, 1));
        data_mapping.insert(3, (self.scroll_y, 1));
        data_mapping.insert(4, (self.inner_width, 1));
        data_mapping.insert(5, (self.page_unload, 1));
        data_mapping.insert(6, (self.inner_height, 1));
        data_mapping.insert(7, (self.outer_width, 1));
        data_mapping.insert(8, (self.timer, 2));
        data_mapping.insert(10, (self.time_difference, 2));
        data_mapping.insert(12, (self.outer_height, 1));
        data_mapping.insert(13, (self.screen_x, 1));
        data_mapping.insert(14, (self.screen_y, 1));
        data_mapping.insert(15, (self.screen_width, 1));

        let mut i = 0;
        while i < 16 {
            if let Some(&(data, length)) = data_mapping.get(&i) {
                if length == 1 {
                    let value = if data > 0 {
                        std::cmp::min(255, data)
                    } else {
                        data
                    };
                    buffer[i] = (value & 0xFF) as u8;
                    i += 1;
                } else if length == 2 {
                    let value = if data > 0 {
                        std::cmp::min(65535, data)
                    } else {
                        data
                    };
                    buffer[i] = ((value >> 8) & 0xFF) as u8;
                    buffer[i + 1] = (value & 0xFF) as u8;
                    i += 2;
                }
            } else {
                let condition_value = if (4 & self.screen_height) != 0 {
                    self.scroll_y
                } else {
                    self.screen_avail_width
                };
                buffer[i] = (condition_value & 0xFF) as u8;
                i += 1;
            }
        }

        let data_str: String = buffer.iter().map(|&b| b as char).collect();
        self.to_binary(data_str)
    }

    fn to_binary(&self, data_str: String) -> String {
        let mut uint16_data = Vec::new();
        let mut uint8_data = Vec::new();

        // 第一次转换：字符串转为Uint16Array等价物
        for char in data_str.chars() {
            uint16_data.push(char as u16);
        }

        // 第二次转换：Uint16Array buffer转为Uint8Array
        for val in uint16_data {
            uint8_data.push((val & 0xFF) as u8);
            uint8_data.push(((val >> 8) & 0xFF) as u8);
        }

        STANDARD.encode(&uint8_data)
    }

    pub fn generate_ctoken(&mut self, is_create_v2: bool) -> String {
        let mut rng = thread_rng();
        
        self.touch_event = 255; // 触摸事件数: 手机端抓包数据
        self.isibility_change = 2; // 可见性变化数: 手机端抓包数据
        self.inner_width = 255; // 窗口内部宽度: 手机端抓包数据
        self.inner_height = 255; // 窗口内部高度: 手机端抓包数据
        self.outer_width = 255; // 窗口外部宽度: 手机端抓包数据
        self.outer_height = 255; // 窗口外部高度: 手机端抓包数据
        self.screen_width = 255; // 屏幕宽度: 手机端抓包数据
        self.screen_height = rng.gen_range(1000..=3000); // 屏幕高度: 用于条件判断
        self.screen_avail_width = rng.gen_range(1..=100); // 屏幕可用宽度: 用于条件判断

        if is_create_v2 {
            // createV2阶段
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.time_difference =
                (current_time + self.time_offset - self.ticket_collection_t) as i32;
            self.timer = self.time_difference + self.stay_time;
            self.page_unload = 25; // 页面卸载数: 手机端抓包数据
        } else {
            // prepare阶段
            self.time_difference = 0;
            self.timer = self.stay_time;
            self.touch_event = rng.gen_range(3..=10);
        }

        self.encode()
    }
}