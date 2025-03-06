use eframe::egui;
use crate::ui;
use crate::windows;

pub struct Myapp{
    //ui
    pub left_panel_width: f32,  //左面板宽度
    pub selected_tab: usize,    //左侧已选中标签
    //加载动画
    pub loading_angle: f32,
    pub is_loading: bool,
    //自定义背景图  （未启用，效果不好，预留暂时不用）
    pub background_texture: Option<egui::TextureHandle>,
    //日志记录
    pub logs: Vec<String>,
    pub show_log_window: bool,




}

impl Myapp{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        //中文字体
        ui::fonts::configure_fonts(&cc.egui_ctx);
        
        Self {
            left_panel_width: 250.0,
            selected_tab: 0,
            is_loading: false,
            loading_angle: 0.0,
            background_texture: None,
            show_log_window: false,
            logs: Vec::new(),
        }
    }

    pub fn add_log(&mut self, message: &str){
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(format!("[{}] {}",timestamp,message));
    }
}


impl eframe::App for Myapp{
    fn update(&mut self, ctx:&egui::Context, frame: &mut eframe::Frame){
        //侧栏
        ui::sidebar::render_sidebar(self,ctx);

        //主窗口
        egui::CentralPanel::default().show(ctx, |ui|{
            ui::tabs::render_tab_content(self, ui);
        } );


        //加载动画
        if self.is_loading{
            ui::loading::render_loading_overlay(self, ctx);
        }

        //日志
        if self.show_log_window{
            windows::log_windows::show(self, ctx);
        }
    }
}