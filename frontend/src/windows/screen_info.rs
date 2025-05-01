use eframe::egui;
use crate::app::Myapp;
use chrono::TimeZone;
use serde_json::Value;

pub fn show(
    app: &mut Myapp,
    ctx: &egui::Context,
    uid: i64,
){
    let bilibili_ticket = app.bilibiliticket_list
                .iter_mut()
                .find(|ticket| ticket.uid == uid)
                .unwrap();
    let mut window_open = app.show_screen_info.is_some();

    let ticket_data = match bilibili_ticket.project_info.clone() {
        Some(ticket) => {
            app.is_loading = false;
            ticket
        }
        None => {
            app.is_loading = true;
            return;
        }
    };
    //默认选择第一个场次（如果尚未选择）
    if app.selected_screen_index.is_none() && !ticket_data.screen_list.is_empty() {
        app.selected_screen_index = Some(0);
    }

    egui::Window::new("项目详情")
    .open(&mut window_open)
    .default_height(600.0)
    .default_width(800.0)
    .resizable(true)
    .show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // 项目标题区
            ui.vertical_centered(|ui| {
                ui.heading(&ticket_data.name);
                ui.add_space(5.0);

                /* // 活动时间和地点
                if let Some(venue_info) = &ticket_data.venue_info {
                    ui.label(format!("{} | {}", ticket_data.project_label, venue_info.name));
                } */
                ui.label(format!("状态: {}", ticket_data.sale_flag));
                ui.add_space(10.0);
            });

            ui.separator();

            // 场次选择区
            ui.heading("选择场次");
            ui.add_space(5.0);

            // 场次选择栏
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (idx, screen) in ticket_data.screen_list.iter().enumerate() {
                        let is_selected = app.selected_screen_index == Some(idx);

                        let btn = ui.add(
                            egui::SelectableLabel::new(
                                is_selected,
                                format!("{} ({})",
                                    screen.name,
                                    &screen.sale_flag.display_name
                                )
                            )

                        );

                        if btn.clicked() {
                            app.selected_screen_index = Some(idx);
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // 显示选中场次的票种信息
            if let Some(idx) = app.selected_screen_index {
                if idx < ticket_data.screen_list.len() {
                    let selected_screen = &ticket_data.screen_list[idx];
                    // 场次信息卡片
                    let mut card=egui::Frame::none();

                    if !ctx.style().visuals.dark_mode {
                        card=card.fill(egui::Color32::from_rgb(245, 245, 250));
                    } else {
                        card=card.fill(egui::Color32::from_rgb(6,6,6));
                    }


                    card.rounding(8.0)
                        .inner_margin(10.0)
                        .outer_margin(10.0)
                        .show(ui, |ui| {
                            // 场次基本信息
                            ui.label(format!("开始时间: {}", format_timestamp(selected_screen.start_time)));
                            ui.label(format!("售票开始: {}", format_timestamp(selected_screen.sale_start)));
                            ui.label(format!("售票结束: {}", format_timestamp(selected_screen.sale_end)));
                            ui.label(format!("售票状态: {}", selected_screen.sale_flag.display_name));

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.heading("票种列表");

                            // 票种表格头
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("票种名称").strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new("操作").strong());
                                    ui.add_space(70.0);
                                    ui.label(egui::RichText::new("状态").strong());
                                    ui.add_space(70.0);
                                    ui.label(egui::RichText::new("价格").strong());
                                });
                            });

                            ui.separator();

                            // 票种列表
                            for ticket in &selected_screen.ticket_list {
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    ui.label(&ticket.desc);

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let button_text = if ticket.clickable { "选择" } else { "不可选" };
                                        let button_enabled = true/* ticket.clickable */;

                                        if ui.add_enabled(
                                            button_enabled,
                                            egui::Button::new(button_text)
                                        ).clicked() {
                                            // 使用正确的类型赋值
                                            if !ticket.clickable{
                                                log::error!("请注意！该票种目前不可售！但是会尝试下单，如果该票持续不可售，多次下单不可售票种可能会被b站拉黑")
                                            }
                                            app.selected_screen_id = Some(selected_screen.id as i64);
                                            app.selected_ticket_id = Some(ticket.id as i64);
                                            app.show_screen_info = None;
                                            bilibili_ticket.screen_id = selected_screen.id.to_string();
                                            log::debug!("{}, {} , {}",selected_screen.id,ticket.id,ticket.project_id);


                                            // 将选中的票种ID保存到项目ID中，准备抢票
                                            app.ticket_id = ticket.project_id.to_string();
                                            bilibili_ticket.select_ticket_id = Some(ticket.id.to_string());
                                            app.confirm_ticket_info= Some(bilibili_ticket.uid.to_string().clone());
                                            log::info!("已选择: {} [{}]", &ticket.desc, ticket.id);
                                        }

                                        ui.add_space(20.0);
                                        ui.label(&ticket.sale_flag.display_name);
                                        ui.add_space(20.0);

                                        // 票价格式化为元
                                        let price = format!("¥{:.2}", ticket.price as f64 / 100.0);
                                        ui.label(egui::RichText::new(price)
                                            .strong()
                                            .color(egui::Color32::from_rgb(245, 108, 108)));
                                    });
                                });
                                ui.separator();
                            }
                        });
                }
            }

            // 项目详细信息区
            ui.add_space(10.0);
            ui.collapsing("查看详细信息", |ui| {
                ui.label("基本信息:");
                ui.indent("basic_info", |ui| {
                    ui.label(format!("项目ID: {}", ticket_data.id));

                    // 检查performance_desc是否存在，并显示基础信息
                    if let Some(desc) = &ticket_data.performance_desc {
                        for item in &desc.list {
                            if item.module == "base_info" {
                                if let Some(array) = item.details.as_array() {
                                    for info_item in array {
                                        if let (Some(title), Some(content)) = (
                                            info_item.get("title").and_then(Value::as_str),
                                            info_item.get("content").and_then(Value::as_str)
                                        ) {
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(format!("{}:", title)).strong());
                                                ui.label(content);
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }


                });
            });
        });

        // 底部按钮
        ui.separator();
        /* ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.button("关闭窗口").clicked() {
                app.show_screen_info = None;
            }
        }); */
    });
    if !window_open{app.show_screen_info = None;
    bilibili_ticket.project_info = None;}

}

// 将时间戳转换为可读时间
// 将时间戳转换为可读时间 (接受usize类型)
fn format_timestamp(timestamp: usize) -> String {
    if timestamp <= 0 {
        return "未设置".to_string();
    }

    // 安全地将usize转为i64
    let timestamp_i64 = match i64::try_from(timestamp) {
        Ok(ts) => ts,
        Err(_) => return "时间戳溢出".to_string(), // 处理极端情况
    };

    match chrono::Local.timestamp_opt(timestamp_i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => "无效时间".to_string(),
    }
}
