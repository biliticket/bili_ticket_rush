use crate::app::Myapp;
use std::sync::Arc;
use common::cookie_manager::CookieManager;
use common::ticket::{BuyerInfo};
use common::taskmanager::{GrabTicketRequest, TaskStatus, TaskRequest};
use eframe::egui;
use egui::{Color32, RichText, Vec2, Stroke};

/// 显示捡漏模式的确认窗口
/// 只需要选择购票人，其他信息都使用默认值
pub fn show(app: &mut Myapp, ctx: &egui::Context, uid: &i64) {
    let mut open = app.confirm_ticket_info.is_some();
    if !open {
        return;
    }
    
    let biliticket_index = match app.bilibiliticket_list.iter().position(|bt| bt.uid == *uid) {
        Some(index) => index,
        None => {
            log::error!("没有找到uid为{}的抢票信息", uid);
            app.confirm_ticket_info = None;
            return;
        }
    };
    
    let biliticket_uid;
    let cookie_manager: Arc<CookieManager>;
    let buyers;
    
    {
        let biliticket = &app.bilibiliticket_list[biliticket_index];
        
        biliticket_uid = biliticket.uid;
        cookie_manager = match &biliticket.account.cookie_manager {
            Some(cm) => cm.clone(),
            None => {
                log::error!("账号未登录或cookie管理器未初始化");
                app.confirm_ticket_info = None;
                return;
            }
        };
        
        // 获取购票人列表
        let buyers_in = match &biliticket.all_buyer_info {
            Some(data) => &data.list,
            None => {
                log::warn!("购票人列表未加载");
                &Vec::new()
            }
        };
        buyers = buyers_in.clone();
    }
    
    // 创建窗口
    egui::Window::new("捡漏模式 - 选择购票人")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(10.0, 15.0);
            
            // 标题区域
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.heading("捡漏模式 - 选择购票人");
                ui.add_space(5.0);
                ui.label(RichText::new("系统会自动监控可用票种并尝试抢票").color(Color32::DARK_GRAY));
            });
            ui.separator();
            
            // 显示一个简单的模式说明
            ui.add_space(10.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(253, 246, 227))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("捡漏模式说明：").strong());
                    ui.label("1. 系统将持续监控所有可能的场次和票种");
                    ui.label("2. 一旦发现可购买票种，会立即尝试下单");
                    /* ui.label("3. 由于速度原因，可能会遇到更多的风控验证"); */
                    ui.label("3. 暂时只支持实名制票捡漏，请务必选择购票人，否则无法进行购票");
                });
            ui.add_space(10.0);
            
            // 计算已选择的购票人数量
            let selected_count = app.selected_buyer_list.as_ref().map_or(0, |list| list.len());
            
            // 购票人选择标题
            ui.horizontal(|ui| {
                ui.heading("选择购票人");
                ui.add_space(5.0);
                ui.label(RichText::new(format!("(已选 {} 人)", selected_count))
                    .color(if selected_count > 0 {
                        Color32::from_rgb(74, 222, 128)
                    } else {
                        Color32::DARK_GRAY
                    }));
            });
            ui.add_space(5.0);
            
            // 购票人列表
            if buyers.is_empty() {
                ui.label(RichText::new("暂无购票人信息，请先添加购票人").color(Color32::DARK_RED));
            } else {
                app.is_loading = false;
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        // 计算可用宽度和每列宽度
                        let available_width = ui.available_width();
                        let card_width = 230.0; // 每个卡片的宽度
                        let columns = (available_width / card_width).max(1.0).floor() as usize;
                        
                        // 创建网格布局
                        egui::Grid::new("buyers_grid")
                            .num_columns(columns)
                            .spacing([10.0, 10.0])
                            .show(ui, |ui| {
                                for (index, buyer) in buyers.iter().enumerate() {
                                    // 检查该购票人是否被选中
                                    let is_selected = app.selected_buyer_list.as_ref()
                                        .map_or(false, |list| list.iter().any(|b| b.id == buyer.id));

                                    let card_color=if !ctx.style().visuals.dark_mode {
                                        if is_selected {
                                            Color32::from_rgb(236, 252, 243) // 选中状态的浅绿色
                                        } else {
                                            Color32::from_rgb(245, 245, 250) // 默认浅灰色
                                        }
                                    } else {
                                        //深色模式
                                        if is_selected {
                                            Color32::from_rgb(6, 20, 6) // 选中状态的黑底浅绿色
                                        } else {
                                            Color32::from_rgb(6, 6, 6) // 默认深黑色
                                        }
                                    };
                                    
                                    // 创建固定宽度的卡片
                                    ui.scope(|ui| {
                                        ui.set_width(card_width - 10.0); // 减去间距
                                        
                                        egui::Frame::none()
                                            .fill(card_color)
                                            .stroke(Stroke::new(
                                                1.0, 
                                                if is_selected { Color32::from_rgb(74, 222, 128) } else { Color32::from_gray(220) }
                                            ))
                                            .rounding(8.0)
                                            .inner_margin(10.0)
                                            .show(ui, |ui| {
                                                let id_type_text = match buyer.id_type {
                                                    0 => "身份证",
                                                    1 => "护照",
                                                    2 => "港澳通行证",
                                                    3 => "台湾通行证",
                                                    _ => "其他证件"
                                                };
                                                
                                                ui.horizontal(|ui| {
                                                    // 多选复选框
                                                    let select_button = if is_selected {
                                                        ui.add(egui::Button::new("☑").fill(Color32::from_rgb(74, 222, 128)))
                                                    } else {
                                                        ui.add(egui::Button::new("☐").fill(Color32::TRANSPARENT))
                                                    };
                                                    
                                                    // 处理选择按钮点击
                                                    if select_button.clicked() {
                                                        // 多选模式：切换选中状态
                                                        if app.selected_buyer_list.is_none() {
                                                            app.selected_buyer_list = Some(Vec::new());
                                                        }
                                                        
                                                        let buyer_list = app.selected_buyer_list.as_mut().unwrap();
                                                        
                                                        // 如果已经选中，则移除；否则添加
                                                        if let Some(pos) = buyer_list.iter().position(|b| b.id == buyer.id) {
                                                            buyer_list.remove(pos);
                                                            log::debug!("移除购票人: {}", buyer.name);
                                                        } else {
                                                            buyer_list.push(buyer.clone());
                                                            log::debug!("添加购票人: {}", buyer.name);
                                                        }
                                                    }
                                                    
                                                    ui.vertical(|ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label(RichText::new(&buyer.name).strong().size(16.0));
                                                            ui.label(RichText::new(id_type_text).weak().size(13.0));
                                                        });
                                                        
                                                        ui.horizontal(|ui| {
                                                            ui.label(format!("证件号: {}", mask_id(&buyer.personal_id)));
                                                        });
                                                        
                                                        ui.horizontal(|ui| {
                                                            ui.label(format!("手机号: {}", buyer.tel));
                                                        });
                                                    });
                                                });
                                            });
                                    });
                                    
                                    // 控制换行
                                    if (index + 1) % columns == 0 && index < buyers.len() - 1 {
                                        ui.end_row();
                                    }
                                }
                            });
                        
                        // 添加购票人按钮
                        ui.add_space(10.0);
                        if ui.button("添加新购票人").clicked() {
                            app.show_add_buyer_window = Some(uid.to_string());
                            app.confirm_ticket_info = None;
                        }
                    });
            }
            
            // 底部按钮区域
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.heading("关键词过滤");
            ui.label(RichText::new("输入需要过滤的关键词，多个关键词用空格分隔。当捡漏到包含这些关键词的标题时将自动跳过。").color(Color32::DARK_GRAY));
            ui.add_space(5.0);
            
            // 文本输入框
            ui.horizontal(|ui| {
                ui.label("过滤关键词：");
                let text_edit = ui.text_edit_singleline(&mut app.skip_words_input);
                
                if text_edit.changed() {
                    // 当文本输入改变时，更新关键词列表
                    let words: Vec<String> = app.skip_words_input
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    
                    if words.is_empty() {
                        app.skip_words = None;
                    } else {
                        app.skip_words = Some(words);
                    }
                }
            });
            
            // 显示当前过滤词列表
            if let Some(words) = &app.skip_words {
                if !words.is_empty() {
                    // 用于记录需要删除的词
                    let mut word_to_delete: Option<String> = None;
                    
                    ui.horizontal_wrapped(|ui| {
                        ui.label("当前过滤词：");
                        for word in words.iter() {
                            let chip = egui::Label::new(
                                RichText::new(format!(" {} ", word))
                                    .background_color(Color32::from_rgb(59, 130, 246))
                                    .color(Color32::WHITE)
                            )
                            .sense(egui::Sense::click());
                            
                            if ui.add(chip).clicked() {
                                // 只记录要删除的词，不立即修改
                                word_to_delete = Some(word.clone());
                            }
                            ui.add_space(5.0);
                        }
                    });
                    
                    // 在闭包外处理删除逻辑
                    if let Some(word) = word_to_delete {
                        if let Some(words_mut) = &mut app.skip_words {
                            if let Some(pos) = words_mut.iter().position(|w| w == &word) {
                                words_mut.remove(pos);
                                
                                // 更新输入框内容
                                app.skip_words_input = words_mut.join(" ");
                                
                                // 如果关键词列表为空，设置为None
                                if words_mut.is_empty() {
                                    app.skip_words = None;
                                }
                            }
                        }
                    }
                }
            }
            
            

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 只有选择了购票人，按钮才可用
                    let has_buyers = app.selected_buyer_list.as_ref().map_or(false, |list| !list.is_empty());
                    
                    if ui.add_enabled(
                        has_buyers,
                        egui::Button::new("开始捡漏")
                            .fill(Color32::from_rgb(59, 130, 246))
                            .min_size(Vec2::new(100.0, 36.0))
                    ).clicked() {
                        if let Some(ref buyer_list) = app.selected_buyer_list {
                            if !buyer_list.is_empty() {
                                let ids: Vec<i64> = buyer_list.iter().map(|b| b.id).collect();
                                log::info!("开始捡漏模式，选择的购票人IDs: {:?}", ids);
                                
                                // 获取必要的数据
                                let mut biliticket = &mut app.bilibiliticket_list[biliticket_index];
                                let project_id = biliticket.project_id.clone();
                                let local_captcha = app.local_captcha.clone();
                                biliticket.id_bind = 1;
                                
                                // 创建捡漏模式的抢票请求
                                let grab_ticket_request = GrabTicketRequest {
                                    task_id: "".to_string(),
                                    uid: biliticket_uid,
                                    project_id,
                                    // 在捡漏模式下，这些值会被后端动态设置，所以这里设为空字符串
                                    screen_id: "".to_string(),
                                    ticket_id: "".to_string(),
                                    count: buyer_list.len() as i16,
                                    buyer_info: buyer_list.clone(),
                                    grab_mode: 2, // 使用捡漏模式
                                    status: TaskStatus::Pending,
                                    start_time: None,
                                    cookie_manager,
                                    biliticket: biliticket.clone(),
                                    local_captcha,
                                    skip_words: app.skip_words.clone(),
                                };
                                
                                log::debug!("提交捡漏模式任务: {:?}", grab_ticket_request);
                                
                                // 提交到任务管理器
                                match app.task_manager.submit_task(TaskRequest::GrabTicketRequest(grab_ticket_request)) {
                                    Ok(task_id) => {
                                        log::info!("提交捡漏模式任务成功，任务ID: {}", task_id);
                                        app.confirm_ticket_info = None;
                                    },
                                    Err(e) => {
                                        log::error!("提交捡漏模式任务失败: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    
                    if ui.button("取消").clicked() {
                        app.confirm_ticket_info = None;
                    }
                });
            });
        });
    
    // 更新窗口打开状态
    if !open {
        app.confirm_ticket_info = None;
    }
}

// 隐藏部分证件号码 - 复用已有函数
fn mask_id(id: &str) -> String {
    if id.len() <= 6 {
        return id.to_string();
    }
    let visible_prefix = &id[..3];
    let visible_suffix = &id[id.len() - 3..];
    let mask_len = id.len() - 6;
    let mask = "*".repeat(mask_len.min(6));
    
    format!("{}{}{}", visible_prefix, mask, visible_suffix)
}