use crate::app::Myapp;
use eframe::egui::{self, RichText};
use egui::{Image, TextureHandle};
use std::collections::HashMap;
use common::account::Account;
use common::http_utils::request_get_sync;

pub fn show(
    app: &mut Myapp,
    ctx: &egui::Context,
){
    let uid = app.show_orderlist_window.as_ref().unwrap().clone();
    let find_account = app.account_manager.accounts.iter().find(|account| account.uid.to_string() == uid);
    let select_account = match find_account {
        Some(account) => account,
        None => return,
    };
    let select_client = select_account.client.as_ref().unwrap();
    let mut window_open = app.show_orderlist_window.is_some();
    egui::Window::new("订单列表")
        .open(&mut window_open)
        .default_height(600.0)
        .default_width(800.0)
        .resizable(true) 
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {            
                ui.label(RichText::new("订单列表")
                    .size(20.0)                               
                    .color(egui::Color32::from_rgb(0, 0, 0))  
                    .strong()
                );
            });
            
            // 添加滚动区域
            egui::ScrollArea::vertical().show(ui, |ui| {
                // 示例订单数据 - 实际中应从API获取
                let orders = vec![
                    ("O2023123456", "五月天2023巡回演唱会", "北京鸟巢", "2023-12-30 20:00", "¥680", "已付款", "https://example.com/image1.jpg"),
                    ("O2023123457", "周杰伦2023地表最强", "上海体育场", "2023-12-25 19:30", "¥1080", "待付款", "https://example.com/image2.jpg"),
                    // ...更多订单
                ];
                
                // 显示订单卡片
                for (order_id, name, venue, time, price, status, image_url) in &orders {
                    ui.add_space(12.0);
                    
                    egui::Frame::none()
                        .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                        .rounding(8.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 220, 220)))
                        .shadow(egui::epaint::Shadow {
                            extrusion: 2.0,
                            color: egui::Color32::from_black_alpha(20),
                        })
                        .inner_margin(egui::vec2(12.0, 12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // 预留图片位置 (可从URL加载或显示默认图片)
                                // 这里只是示例，实际中应该异步加载图片
                                let image_size = egui::vec2(80.0, 80.0);
                                
                                // 图片加载逻辑 - 这里使用占位区域
                                ui.add_sized(image_size, |ui: &mut egui::Ui| {
                                    // 直接返回自己的响应
                                    if let Some(texture) = get_image_texture(ctx, image_url) {
                                        // 居中显示图片
                                        ui.centered_and_justified(|ui| {
                                            ui.add(Image::new(texture).fit_to_exact_size(image_size))
                                        }).inner
                                    } else {
                                        // 居中显示加载提示
                                        let inner_response = ui.centered_and_justified(|ui| {
                                            ui.label("加载中...")
                                        });
                                        // 触发图片加载
                                        request_image_async(ctx.clone(), image_url.to_string());
                                        inner_response.inner
                                    }
                                });
                                
                                ui.add_space(12.0);
                                
                                // 订单信息区域
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(*name).size(16.0).strong());
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(*status)
                                                .color(if *status == "已付款" { 
                                                    egui::Color32::from_rgb(0, 150, 0) 
                                                } else { 
                                                    egui::Color32::from_rgb(200, 80, 0) 
                                                })
                                                .strong());
                                        });
                                    });
                                    
                                    ui.add_space(4.0);
                                    
                                    // 订单详细信息
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("订单号:").color(egui::Color32::GRAY));
                                        ui.monospace(*order_id);
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("场馆:").color(egui::Color32::GRAY));
                                        ui.label(*venue);
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("场次:").color(egui::Color32::GRAY));
                                        ui.label(*time);
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("价格:").color(egui::Color32::GRAY));
                                        ui.label(RichText::new(*price).strong());
                                        
                                        // 操作按钮放在右侧
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let button = egui::Button::new(
                                                egui::RichText::new("查看详情").size(20.0).color(egui::Color32::WHITE)
                                            )
                                            .min_size(egui::vec2(120.0, 50.0))
                                            .fill(egui::Color32::from_rgb(102,204,255))
                                            .rounding(20.0);
                                            let response = ui.add(button);
                                            if response.clicked()
                                            {
                                                // 处理点击事件
                                                log::debug!("查看订单详情: {}", order_id);
                                            }
                                        });
                                    });
                                });
                            });
                        });
                }
                
                ui.add_space(10.0); // 底部留白
            });
        });
    
    if !window_open {
        app.show_orderlist_window = None;
    }
}

// 辅助函数：从缓存获取图片纹理
fn get_image_texture<'a>(ctx: &'a egui::Context, url: &str) -> Option<&'a TextureHandle> {
    ctx.memory(|mem| mem.data.get_temp(egui::Id::new(url)))
}

// 辅助函数：异步请求图片
fn request_image_async(ctx: egui::Context, url: String) {
    // 避免重复请求
    if ctx.memory(|mem| mem.data.get_temp::<bool>(egui::Id::new(format!("loading_{}", url))).is_some()) {
        return;
    }
    
    // 标记为正在加载
    ctx.memory_mut(|mem| mem.data.insert_temp(egui::Id::new(format!("loading_{}", url)), true));
    
    // 启动异步加载线程
    std::thread::spawn(move || {
        // 这里应该实现实际的图片加载逻辑
        // 示例：
        /*
        if let Ok(bytes) = download_image(&url) {
            if let Some(image) = load_image_from_bytes(&bytes) {
                ctx.memory_mut(|mem| {
                    let texture = ctx.load_texture(url.clone(), image);
                    mem.data.insert_temp(egui::Id::new(&url), texture);
                    mem.data.remove::<bool>(egui::Id::new(format!("loading_{}", url)));
                });
                ctx.request_repaint();
            }
        }
        */
        
        // 为了示例，假设加载成功
        std::thread::sleep(std::time::Duration::from_secs(1));
        ctx.request_repaint();
    });
}