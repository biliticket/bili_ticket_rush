use eframe::egui;
use std::{fs, io, path::Path};

pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    let font_data = load_system_font().unwrap_or_else(|_| {
        log::warn!("使用内置中文字体");
        include_bytes!("../../../resources/fonts/NotoSansSC-Regular.otf").to_vec()
    });

    fonts.font_data.insert(
        "chinese_font".to_owned(),
        egui::FontData::from_owned(font_data),
    );

    fonts.families.values_mut().for_each(|family| {
        family.insert(0, "chinese_font".to_owned());
    });

    ctx.set_fonts(fonts);
}

fn load_system_font() -> io::Result<Vec<u8>> {
    let paths = get_system_font_paths();
    
    paths.into_iter()
        .find(|path| Path::new(path).exists())
        .and_then(|path| {
            log::info!("加载字体: {}", path);
            fs::read(path).ok()
        })
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "No suitable font found"))
}

fn get_system_font_paths() -> Vec<&'static str> {
    if cfg!(target_os = "windows") {
        vec![
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/simhei.ttf",
            "C:/Windows/Fonts/simsun.ttc",
            "C:/Windows/Fonts/msyh.ttf",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ]
    } else {
        vec![
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ]
    }
}