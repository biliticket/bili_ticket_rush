use crate::app::Myapp;
use common::ticket::BilibiliTicket;
use eframe::egui;

pub fn show(app: &mut Myapp,ctx:&egui::Context,uid:&i64){
    let biliticket = match app.bilibiliticket_list.iter()
    .find(|biliticket| biliticket.uid == *uid){
        Some(biliticket) => biliticket,
        None => {
            log::error!("没有找到uid为{}的抢票信息",uid);
            return;
        }
    };

}