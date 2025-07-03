use eframe::egui;
use std::fs::read;
use std::path::Path;

// 配置字体函数
pub fn configure_fonts(ctx: &egui::Context) {
    // 创建字体配置
    let mut fonts = egui::FontDefinitions::default();
    
    // 根据不同操作系统选择合适的字体路径
    let font_data = load_system_font();
    
    // 使用from_owned方法创建FontData
    fonts.font_data.insert(
        "chinese_font".to_owned(),
        egui::FontData::from_owned(font_data)
    );
    
    // 将中文字体添加到所有字体族中
    for family in fonts.families.values_mut() {
        family.insert(0, "chinese_font".to_owned());
    }
    
    // 应用字体
    ctx.set_fonts(fonts);
}

// 根据操作系统加载合适的字体
fn load_system_font() -> Vec<u8> {
    #[cfg(target_os = "windows")]
    {
        // 尝试多个Windows字体路径
        let font_paths = [
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/simhei.ttf",
            "C:/Windows/Fonts/simsun.ttc",
            "C:/Windows/Fonts/msyh.ttf"
        ];
        
        for path in font_paths {
            if Path::new(path).exists() {
                if let Ok(data) = read(path) {
                    log::info!("加载字体: {}", path);
                    return data;
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS系统字体路径
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc"
        ];
        
        for path in font_paths {
            if Path::new(path).exists() {
                if let Ok(data) = read(path) {
                    log::info!("加载字体: {}", path);
                    return data;
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux系统字体路径
        let font_paths = [
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc"
        ];
        
        for path in font_paths {
            if Path::new(path).exists() {
                if let Ok(data) = read(path) {
                    log::info!("加载字体: {}", path);
                    return data;
                }
            }
        }
    }
    
    // 如果所有系统字体都无法加载，使用内置的字体
    log::warn!("无法加载系统中文字体，使用内置字体");
    include_bytes!("../../../resources/fonts/NotoSansSC-Regular.otf").to_vec()
}
