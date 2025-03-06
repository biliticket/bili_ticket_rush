use eframe::egui;
use std::fs::read;

// 配置字体函数
pub fn configure_fonts(ctx: &egui::Context) {
    // 创建字体配置
    let mut fonts = egui::FontDefinitions::default();
    
    // 使用std::fs::read读取字体文件
    let font_data = read("C:/Windows/Fonts/msyh.ttc").unwrap_or_else(|_| {
        // 备用字体
        read("C:/Windows/Fonts/simhei.ttf").unwrap()
    });
    
    // 使用from_owned方法创建FontData
    fonts.font_data.insert(
        "microsoft_yahei".to_owned(),
        egui::FontData::from_owned(font_data)
    );
    
    // 将中文字体添加到所有字体族中
    for family in fonts.families.values_mut() {
        family.insert(0, "microsoft_yahei".to_owned());
    }
    
    // 应用字体
    ctx.set_fonts(fonts);
}