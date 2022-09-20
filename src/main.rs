#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_tools;

use std::process::Command;
use std::{env, path::PathBuf};
// hide console window on Windows in release
use eframe::egui;
use egui::*;
use file_tools::{
    check_installed_liteloader, check_installed_trapdoor, check_is_bds_root, extract_file_to,
};
use poll_promise::Promise;

fn main() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
   // resizable : false        ,
        ..Default::default()
    };


    eframe::run_native(
        "Trapdoor 安装器 v0.2",
        options,
        Box::new(|_cc| Box::new(MyApp::new(_cc))),
    );
}


struct MyApp {
    bds_path_vaild: bool,
    lieloader_installed: bool,
    trapdoor_installed: bool,
    bds_root_path: String,
    dropped_files: Vec<egui::DroppedFile>,
    promise: Option<Promise<bool>>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //set style

        use FontFamily::Proportional;
        let ctx = &cc.egui_ctx;

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "msyhl".to_owned(),
            FontData::from_static(include_bytes!(r"C:\WINDOWS\FONTS\MSYHL.TTC")),
        );

        fonts
            .families
            .insert(FontFamily::Monospace, vec!["msyhl".to_owned()]);
        fonts
            .families
            .insert(FontFamily::Proportional, vec!["msyhl".to_owned()]);

        //fonts.families.append(;);
        ctx.set_fonts(fonts);
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Heading, FontId::new(28.0, Proportional)),
            (TextStyle::Body, FontId::new(20.0, Proportional)),
            (TextStyle::Monospace, FontId::new(20.0, Proportional)),
            (TextStyle::Button, FontId::new(20.0, Proportional)),
            (TextStyle::Small, FontId::new(16.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        let defalut_path = env::current_dir().unwrap().to_str().unwrap().to_string();

        Self {
            bds_path_vaild: check_is_bds_root(&defalut_path),
            lieloader_installed: check_installed_liteloader(&defalut_path),
            trapdoor_installed: check_installed_trapdoor(&defalut_path),
            bds_root_path: defalut_path,
            dropped_files: vec![],
            promise: None,
        
        }
    }

    fn refresh_status(&mut self) {
        self.bds_path_vaild = check_is_bds_root(&self.bds_root_path);
        if self.bds_path_vaild {
            self.lieloader_installed = check_installed_liteloader(&self.bds_root_path);
        } else {
            self.lieloader_installed = false;
        }

        if self.lieloader_installed {
            self.trapdoor_installed = check_installed_trapdoor(&self.bds_root_path);
        } else {
            self.trapdoor_installed = false;
        }
    }
}

pub fn get_color(status: bool) -> egui::Color32 {
    if status {
        egui::Color32::from_rgb(0, 204, 102)
    } else {
        egui::Color32::from_rgb(255, 51, 51)
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        _frame.set_window_size(egui::vec2(540.0, 380.0));



//        painter.rect_filled(screen_rect, 0.0, Color32::from_rgba_premultiplied(100,10, 20, 255));


        egui::CentralPanel::default().show(ctx, |ui| {

       


            ui.heading("状态信息");

            ui.horizontal_wrapped(|ui| {
                // Trick so we don't have to add spaces in the text below:
                let width = ui
                    .fonts()
                    .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
                ui.spacing_mut().item_spacing.x = width;
                ui.label(&format!("当前BDS路径: {}\n", self.bds_root_path));
                ui.label("BDS路径合法状态:  ");
                ui.colored_label(
                    get_color(self.bds_path_vaild),
                    self.bds_path_vaild.to_string(),
                );

                ui.label("\nLiteLoader 安装状态:  ");
                ui.colored_label(
                    get_color(self.lieloader_installed),
                    self.lieloader_installed.to_string(),
                );

                ui.label("\nTrapdoor 安装状态:  ");
                ui.colored_label(
                    get_color(self.trapdoor_installed),
                    self.trapdoor_installed.to_string(),
                );
            });

            if ui.button("修改BDS根目录").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.bds_root_path = path.display().to_string();
                    self.refresh_status();
                }
            }

            ui.horizontal_wrapped(|ui| {
                if let Some(promise) = &self.promise {
                    if let Some(result) = promise.ready() {
                        ui.label("任务状态:  ");
                        ui.colored_label(get_color(*result), result.to_string());
                        self.refresh_status();
                    } else {
                        ui.spinner();
                    }
                }
            });
            
       
            ui.heading("使用教程");
            ui.label("在任意路径创建空文件夹，点击修改BDS根目录并选中,依次将你下载的BDS(bedorck_server_xxx.zip),ll(Liteloader-2.xx.zip)以及 trapdoor(release.zip)拖入这里即可");
            
            ui.heading("下载链接");
            ui.hyperlink_to("下载BDS", "https://www.minecraft.net/en-us/download/server/bedrock");
            ui.hyperlink_to("下载LiteloaderBDS", "https://github.com/LiteLDev/LiteLoaderBDS/releases");
            ui.hyperlink_to("下载Trapdoor-ll", "https://github.com/bedrock-dev/trapdoor-ll/releases");

            
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            self.dropped_files = ctx.input().raw.dropped_files.clone();
            let target_file = self.dropped_files[0]
                .clone()
                .path
                .unwrap()
                .display()
                .to_string();

            if target_file.ends_with(".zip") {
                let out_dir = self.bds_root_path.clone();
                let promise = poll_promise::Promise::spawn_thread("slow_operation", move || {
                    extract_file_to(&target_file, &out_dir);

                    if target_file.contains("LiteLoader") {
                        //RUN COMD

                        let pbf = PathBuf::from(out_dir);

                        let dir = format!("\"{}\"", pbf.to_str().unwrap());
                        let exe = format!("\"{}\\LLPeEditor.exe\"", pbf.to_str().unwrap());

                        println!("cmd is {} dir is {}", exe, dir);
                        if let Err(e) = Command::new("powershell")
                            .args(&["start", "-NoNewWindow", &exe, "-WorkingDirectory", &dir])
                            .output()
                        {
                            println!("{:?}", e);
                        } else {
                            println!("Command run succseefully")
                        }
                    }

                    return true;
                });

                self.promise = Some(promise);
            }
        }
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;

    if !ctx.input().raw.hovered_files.is_empty() {
        let target_file = ctx.input().raw.hovered_files[0]
            .clone()
            .path
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        let valid = target_file.ends_with(".zip");

        let text = if valid {
            "即将安装 :\n".to_owned() + &target_file
        } else {
            "非法的文件类型 :\n".to_owned() + &target_file
        };

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.input().screen_rect();

        let text_color = if valid {
            Color32::WHITE
        } else {
            Color32::from_rgb(250, 128, 114)
        };

        painter.rect_filled(screen_rect, 0.0, text_color);
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::BLACK,
        );
    }
}
