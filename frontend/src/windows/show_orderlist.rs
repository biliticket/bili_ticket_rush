use crate::app::{Myapp, OrderData};
use eframe::egui::{self, RichText};
use egui::{Image, TextureHandle};
use serde::{Deserialize, Serialize};
use common::account::Account;
use common::http_utils::request_get_sync;



pub fn show(
    app: &mut Myapp,
    ctx: &egui::Context,
    
){
    let mut window_open = app.show_orderlist_window.is_some();
    
    let orders_data = match &app.total_order_data {
        Some(data) => {app.is_loading = false; data.clone()},
        None => {app.is_loading = true; return;}, 
    };
    
    
    // 显示窗口和订单数据
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
                // 使用从内存中获取的orders_data
                if let Some(order_data) = &orders_data.data {
                    // 显示订单数据
                    for order in &order_data.data.list {
                        // 原有的显示逻辑...
                        
                        // 注意：这里不要修改orders_data
                    }
                } else {
                    // 显示加载中状态
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("加载中...").size(16.0).color(egui::Color32::GRAY));
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