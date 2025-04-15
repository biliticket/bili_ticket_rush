use crate::app::Myapp;
use common::ticket::BilibiliTicket;
use eframe::egui;
use egui::{Color32, RichText, Vec2, Stroke};

pub fn show(app: &mut Myapp,ctx:&egui::Context,uid:&i64){
    let mut open = app.confirm_ticket_info.is_some();
    if !open {
        return;
    }
    let biliticket = match app.bilibiliticket_list.iter_mut()
    .find(|biliticket| biliticket.uid == *uid){
        Some(biliticket) => biliticket,
        None => {
            log::error!("没有找到uid为{}的抢票信息",uid);
            app.confirm_ticket_info = None;
            return;
        }
    };
    app.is_loading = false;
     // 查找当前选择的场次和票种信息
     let (screen_info, ticket_info) = match &biliticket.project_info {
        Some(project) => {
            let screen = project.screen_list.iter().find(|s| 
                s.id.to_string() == biliticket.screen_id);
            
            if let Some(screen) = screen {
                let ticket = screen.ticket_list.iter().find(|t| 
                    t.id == app.selected_ticket_id.unwrap_or(-1) as usize);
                
                (Some(screen), ticket)
            } else {
                (None, None)
            }
        },
        None => (None, None)
    };

    // 获取购票人列表
    let buyers = match &biliticket.all_buyer_info {
        Some(data) => &data.list,
        None => {
            //log::error!("购票人列表未加载，请先获取购票人信息");
            &Vec::new() // 返回空列表
        }
    };

    // 创建窗口
    egui::Window::new("确认购票信息")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .min_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
                    if let Some(project) = &biliticket.project_info {
                        ui.label(RichText::new(&project.name).strong().size(16.0));
                    }

                    // 显示场次和票种信息
                    if let Some(screen) = screen_info {
                        ui.label(RichText::new(format!("场次: {}", &screen.name)).size(14.0));
                        
                        if let Some(ticket) = ticket_info {
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

            // 购票人选择部分
            ui.add_space(10.0);
            ui.heading("选择购票人");
            ui.add_space(5.0);

            if buyers.is_empty() {
                // 空列表处理代码不变...
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
                                    let is_selected = app.selected_buyer_id == Some(buyer.id);
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
                                                    // 添加一个明确的选择按钮
                                                    let select_button = if is_selected {
                                                        ui.add(egui::Button::new("✓").fill(Color32::from_rgb(74, 222, 128)))
                                                    } else {
                                                        ui.add(egui::Button::new("○").fill(Color32::TRANSPARENT))
                                                    };
                                                    
                                                    if select_button.clicked() {
                                                        log::debug!("选择购票人: {}", buyer.name);
                                                        app.selected_buyer_id = Some(buyer.id);
                                                        biliticket.buyer_info = Some(buyer.clone());
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
            
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(
                        app.selected_buyer_id.is_some(),
                        egui::Button::new("确认购票")
                            .fill(Color32::from_rgb(59, 130, 246))
                            .min_size(Vec2::new(100.0, 36.0))
                    ).clicked() {
                        if let Some(buyer_id) = app.selected_buyer_id {
                            log::info!("确认购票，选择的购票人ID: {}", buyer_id);
                            
                            /* // 将此账号加入抢票队列
                            app.grab_tickets.push(biliticket.uid); */
                            
                            // 关闭窗口
                            app.confirm_ticket_info = None;
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