use crate::app::Myapp;
use common::ticket::{*};
use common::taskmanager::{GrabTicketRequest, TaskStatus, TaskRequest};
use eframe::egui;
use egui::{Color32, RichText, Vec2, Stroke};

pub fn show(app: &mut Myapp,ctx:&egui::Context,uid:&i64){
    
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
    let biliticket_project_id;
    let biliticket_session;
    let id_bind;
    let screen_info: Option<ScreenInfo>;
    let ticket_info: Option<ScreenTicketInfo>;
    let buyers;
    

    app.is_loading = false;
    {
        let biliticket = &app.bilibiliticket_list[biliticket_index];
        
        biliticket_uid = biliticket.uid;
        biliticket_project_id = biliticket.project_info.as_ref().map(|p| p.id.to_string());
        biliticket_session = biliticket.session.clone();
        
        id_bind = match &biliticket.project_info {
            Some(project) => project.id_bind,
            None => 9,
        };
    
     // 查找当前选择的场次和票种信息
     let (screen, ticket) = match &biliticket.project_info {
        Some(project) => {
            let screen = project.screen_list.iter().find(|s| 
                s.id.to_string() == biliticket.screen_id);
            
            if let Some(screen) = screen {
                let ticket = screen.ticket_list.iter().find(|t| 
                    t.id == app.selected_ticket_id.unwrap_or(-1) as usize);
                
                (Some(screen.clone()), ticket.cloned())
            } else {
                (None, None)
            }
        },
        None => (None, None)
    };
    screen_info = screen;
    ticket_info = ticket;
    

    // 获取购票人列表
    let buyers_in = match &biliticket.all_buyer_info {
        Some(data) => &data.list,
        None => {
            //log::error!("购票人列表未加载，请先获取购票人信息");
            &Vec::new() // 返回空列表
        }
    };
    buyers = buyers_in.clone();
  }
    let screen_info_display = screen_info.clone();
    let screen_info_button = screen_info.clone();
    let ticket_info_display = ticket_info.clone();

    // 创建窗口
    egui::Window::new("确认购票信息")
        .open(&mut open)
        .collapsible(true)
        .resizable(true)
        .default_width(500.0)
        //.anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(10.0, 15.0);

            // 标题区域
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.heading("确认购票信息");
                ui.add_space(5.0);
            });
            ui.separator();

            // 票务信息部分
            ui.add_space(5.0);
            ui.heading("已选择票种");
            ui.add_space(5.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(245, 245, 250))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    // 显示项目名称
                    let biliticket = &app.bilibiliticket_list[biliticket_index];
                    if let Some(project) = &biliticket.project_info {
                        ui.label(RichText::new(&project.name).strong().size(16.0));
                    }

                    // 显示场次和票种信息
                    if let Some(screen) = screen_info_display {
                        ui.label(RichText::new(format!("场次: {}", &screen.name)).size(14.0));
                        
                        if let Some(ticket) = ticket_info_display {
                            ui.horizontal(|ui| {
                                ui.label(format!("票种: {}", &ticket.desc));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("¥{:.2}", ticket.price as f64 / 100.0))
                                        .color(Color32::from_rgb(239, 68, 68))
                                        .strong());
                                });
                            });
                        }
                    }
                });
            
            match id_bind{
                0 =>{
                    ui.add_space(10.0);
                    ui.heading("输入联系人");
                    ui.add_space(5.0);

                    ui.horizontal(|ui|{
                        let biliticket = &mut app.bilibiliticket_list[biliticket_index];
                        common_input(ui, "请输入联系人姓名", &mut biliticket.nobind_name, "请输入联系人姓名",false);
                        ui.add_space(10.0);
                        common_input(ui, "请输入联系人手机号", &mut biliticket.nobind_tel, "请输入联系人手机号",true);
                        ui.add_space(10.0);
                    });

                }
                1|2 =>{
                    
                
                ui.add_space(10.0);
                 
               if id_bind == 2 {
                          let selected_count = app.selected_buyer_list.as_ref().map_or(0, |list| list.len());
    
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
                } else {
                     ui.heading("选择购票人");
                }
                ui.add_space(5.0);

                if buyers.is_empty() {
               
                } else {
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
                                    // 判断是单选还是多选模式
                                    let is_multi_select = id_bind == 2;
                                    
                                    // 检查该购票人是否被选中 - 对单选和多选都使用 selected_buyer_list
                                    let is_selected = app.selected_buyer_list.as_ref()
                                        .map_or(false, |list| list.iter().any(|b| b.id == buyer.id));
                                    
                                    let card_color = if is_selected {
                                        Color32::from_rgb(236, 252, 243) // 选中状态的浅绿色
                                    } else {
                                        Color32::from_rgb(245, 245, 250) // 默认浅灰色
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
                                                    // 添加不同样式的选择按钮
                                                    let select_button = if is_multi_select {
                                                        // 多选模式：显示复选框样式
                                                        if is_selected {
                                                            ui.add(egui::Button::new("☑").fill(Color32::from_rgb(74, 222, 128)))
                                                        } else {
                                                            ui.add(egui::Button::new("☐").fill(Color32::TRANSPARENT))
                                                        }
                                                    } else {
                                                        // 单选模式：显示单选框样式
                                                        if is_selected {
                                                            ui.add(egui::Button::new("✓").fill(Color32::from_rgb(74, 222, 128)))
                                                        } else {
                                                            ui.add(egui::Button::new("○").fill(Color32::TRANSPARENT))
                                                        }
                                                    };
                                                    
                                                    // 处理选择按钮点击
                                                    if select_button.clicked() {
                                                        if is_multi_select {
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
                                                        } else {
                                                            // 单选模式：替换当前选择的购票人
                                                            log::debug!("选择购票人: {}", buyer.name);
                                                            //app.selected_buyer_id = Some(buyer.id); // 保持单选ID兼容
                                                            app.selected_buyer_list = Some(vec![buyer.clone()]); // 使用List，但只有一个
                                                            let biliticket = &mut app.bilibiliticket_list[biliticket_index];
                                                            biliticket.buyer_info = Some(vec![buyer.clone()]);
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
                }
                _ =>{
                    ui.add_space(10.0);
                    ui.label("该项目不支持选择购票人（未知状态码），请尝试直接购票！");
                }
            }
            

            // 底部按钮区域
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 根据模式决定按钮启用条件
                    let biliticket = &app.bilibiliticket_list[biliticket_index];
                    let button_enabled = match id_bind {
                        0 => biliticket.nobind_tel.is_some() && biliticket.nobind_name.is_some(),
                        1 => app.selected_buyer_list.as_ref().map_or(false, |list| !list.is_empty()),
                        2 => app.selected_buyer_list.as_ref().map_or(false, |list| !list.is_empty()),
                        _ => false,
                    };
                    
                    if ui.add_enabled(
                        button_enabled,
                        egui::Button::new("确认购票")
                            .fill(Color32::from_rgb(59, 130, 246))
                            .min_size(Vec2::new(100.0, 36.0))
                    ).clicked() {
                        match id_bind {
                            0 => {
                                // 已有代码...
                            }
                            1 | 2 => {
                                if let Some(ref buyer_list) = app.selected_buyer_list {
                                    if !buyer_list.is_empty() {
                                        let ids: Vec<i64> = buyer_list.iter().map(|b| b.id).collect();
                                        log::info!("确认购票，选择的购票人IDs: {:?}", ids);
                                        
                                        if let Some(screen) = screen_info_button {
                                            if let Some(ticket) = ticket_info {
                                                // 提交抢票任务
                                                let grab_ticket_request = GrabTicketRequest {
                                                    task_id: "".to_string(),
                                                    uid: biliticket_uid,
                                                    project_id: biliticket_project_id.clone().unwrap_or_default(),
                                                    screen_id: screen.id.to_string(),
                                                    ticket_id: ticket.id.to_string(),
                                                    buyer_info: buyer_list.clone(),
                                                    grab_mode: app.grab_mode,
                                                    status: TaskStatus::Pending,
                                                    start_time: None,
                                                    client: biliticket_session.unwrap(),
                                                    biliticket: biliticket.clone(),
                                                };
                                                log::debug!("提交抢票任务: {:?}", grab_ticket_request);
                                                // 提交到任务管理器
                                                match app.task_manager.submit_task(TaskRequest::GrabTicketRequest(grab_ticket_request)) {
                                                    Ok(task_id) => {
                                                        log::info!("提交抢票任务成功，任务ID: {}", task_id);
                                                        app.confirm_ticket_info = None;
                                                    },
                                                    Err(e) => {
                                                        log::error!("提交抢票任务失败: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        // 关闭窗口
                                        app.confirm_ticket_info = None;
                                    }
                                }
                            }
                            
                            _ => {
                                log::error!("未知的购票人绑定状态: {}", id_bind);
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

// 隐藏部分证件号码
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

pub fn common_input(
    ui: &mut egui::Ui, 
    title: &str,
    text: &mut Option<String>,
    hint: &str,
    open_filter: bool,


) -> bool{
    if text.is_none() {
        *text = Some(String::new());
    }
    let text_ref = text.as_mut().unwrap();
    ui.label(
        egui::RichText::new(title)
              .size(15.0)                               
              .color(egui::Color32::from_rgb(0,0,0))  

              
    );
    ui.add_space(8.0);
    let input = egui::TextEdit::singleline( text_ref)
                .hint_text(hint)//提示
                .desired_rows(1)//限制1行       
                .min_size(egui::vec2(120.0, 35.0));
                
                
    let response = ui.add(input);
    if response.changed(){
        if open_filter{
            *text_ref = text_ref.chars()//过滤非法字符
            .filter(|c| c.is_ascii_alphanumeric() || *c == '@' || *c == '.' || *c == '-' || *c == '_')
            .collect();
        }
        else{
            *text_ref = text_ref.chars()//过滤非法字符
            .collect();
        };
            
    }
    response.changed()

}