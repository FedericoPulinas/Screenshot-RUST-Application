mod imglib;
mod screenlib;
mod hotkeylib;
mod pathlib;
mod paintlib;
mod savelib;
mod cutlib;

use std::borrow::Cow;
use imglib::AllFormats;
use screenlib::MyScreenshot;
use hotkeylib::MyHotKey;
use pathlib::MyPath;
use paintlib::Painting;

use eframe::emath::Align;
use eframe::Frame;
use egui::{Ui, Separator, Context, CentralPanel, TopBottomPanel, Layout, Grid, menu, CollapsingHeader, Window, TextureHandle, Vec2};
use std::path::PathBuf;
use std::env;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use arboard::{Clipboard, ImageData};
use global_hotkey::hotkey::{Code as KeyCode, HotKey, Modifiers as KeyModifiers};
use image::{RgbaImage, imageops};
use serde::{Serialize, Deserialize};
use crate::myapp::cutlib::MyCut;
use crate::myapp::hotkeylib::STD_HOTKEYS;
use crate::myapp::imglib::{load_image_from_memory, restore_dim};
use crate::myapp::paintlib::Shapes;
use crate::myapp::savelib::MySave;

pub const PADDING: f32 = 5.0;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub take_screenshot: (u32, String, String),
    pub save_screenshot: (u32, String, String),
    pub format: String,
    pub path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let tsm = KeyModifiers::SHIFT;
        let tsc = KeyCode::KeyD;
        let idt = HotKey::new(Some(tsm), tsc).id();
        let ssm = KeyModifiers::CONTROL;
        let ssc = KeyCode::KeyS;
        let ids = HotKey::new(Some(ssm), ssc).id();
        Self {
            take_screenshot: (idt, KeyModifiersWrapper(KeyModifiers::SHIFT).to_string(), KeyCodeWrapper(KeyCode::KeyD).to_string()),
            save_screenshot: (ids, KeyModifiersWrapper(KeyModifiers::CONTROL).to_string(), KeyCodeWrapper(KeyCode::KeyS).to_string()),
            format: String::from("png"),
            path: env::current_dir().expect("Current directory not accessible"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Layouts{
    Home,
    Hotkey,
    Screenshot,
    Path,
    About
}

pub struct MyApp {
    pub config: Config,
    pub format: AllFormats,
    screen_ly: MyScreenshot,
    pub hotkey_ly: MyHotKey,
    path_ly: MyPath,
    pub layout: Layouts,
    pub open_settings: bool,
    pub open_save: bool,
    img: Option<RgbaImage>,
    //a prova potremmo cambiare nome
    prova: Option<RgbaImage>,
    pub texture: Option<TextureHandle>,
    painting: Option<Painting>,
    save_ly: Option<MySave>,
    clipboard: Option<Clipboard>,
    wait: bool,
    timeout: f64,
    pub(crate) disabled_time: f64,
    pub saving: bool,
    //usati per tokio
    pub tx: Sender<Duration>,
    pub rx: Receiver<Duration>,
    shape: Shapes,
    mycut: Option<MyCut>,
    copy: bool,
    pub save_by_hk: bool,
    pub dim: Option<(f32, f32)>,
}

impl Default for MyApp {
    fn default() -> Self {
        let config : Config = confy::load("screenshot", "screenshot").unwrap_or_default();
        //println!("{:?}", confy::get_configuration_file_path("screenshot", "screenshot").unwrap());
        let paths = config.path.clone();
        let format = imglib::format_from_string(config.format.as_str()).unwrap_or(AllFormats::PNG);
        let take_screenshot = config.take_screenshot.clone();
        let save_screenshot = config.save_screenshot.clone();
        let clipboard = Clipboard::new().ok();
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            config,
            format,
            screen_ly: MyScreenshot::default(),
            hotkey_ly: MyHotKey::new(take_screenshot, save_screenshot),
            path_ly: MyPath::new(paths.clone()),
            layout: Layouts::Home,
            open_settings: false,
            open_save: false,
            img: None,
            prova: None,
            texture: None,
            painting: None,
            save_ly: None,
            saving: false,
            clipboard,
            tx,
            rx,
            wait: false,
            timeout: 0.,
            disabled_time: f64::NEG_INFINITY,
            shape: Shapes::None,
            mycut: None,
            copy: false,
            save_by_hk: false,
            dim: None,
        }
    }
}

impl MyApp {
    /**schermata home**/
    pub fn home_layout(&mut self, ctx: &Context, _frame: &mut Frame){
        CentralPanel::default().show(ctx, |ui| {
            menu::bar(ui, |ui|{
                ui.horizontal(|ui|{
                    ui.with_layout(Layout::left_to_right(Align::TOP), |ui|{
                        if ui.button("+ New")
                            .on_hover_text(format!("{} + {}", self.config.take_screenshot.1, self.config.take_screenshot.2))
                            .clicked() {
                            _frame.set_visible(false);
                            self.open_settings = false;
                            self.disabled_time = ui.input(|i| i.time);
                            self.layout = Layouts::Screenshot;
                        }
                    });
                    egui::ComboBox::from_label("🕒")
                        .width(5.)
                        .selected_text(format!("{:?}", self.timeout as u32))
                        .show_ui(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.set_min_width(60.0);
                            if ui.selectable_value(&mut self.timeout, 0., "🕒 0 seconds").clicked() {
                                self.timeout = 0.;
                            };
                            if ui.selectable_value(&mut self.timeout, 3., "🕒 3 seconds").clicked() {
                                self.timeout = 3.;
                            };
                            if ui.selectable_value(&mut self.timeout, 5., "🕒 5 seconds").clicked() {
                                self.timeout = 5.;
                            };
                            if ui.selectable_value(&mut self.timeout, 10., "🕒 10 seconds").clicked() {
                                self.timeout = 10.;
                            };
                        });

                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui|{
                        self.render_settings(ui, _frame);
                    });
                });
            });

            ui.horizontal_centered(|ui|{
                self.render_body(ui);
            });

            if self.texture.is_some() && self.mycut.is_none() {
                ui.horizontal_wrapped(|ui| {
                    let painting = self.painting.as_mut().unwrap();
                    painting.stroke(ui);
                    egui::ComboBox::from_label("Shape")
                        .selected_text(format!("{}", self.shape.to_name()))
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.shape, Shapes::None, Shapes::None.to_name()).clicked() {
                                self.shape = Shapes::None;
                                painting.set_shape(Shapes::None);
                            }
                            if ui.selectable_value(&mut self.shape, Shapes::Rect, Shapes::Rect.to_name()).clicked() {
                                self.shape = Shapes::Rect;
                                painting.set_shape(Shapes::Rect);
                            }
                            if ui.selectable_value(&mut self.shape, Shapes::Circle, Shapes::Circle.to_name()).clicked() {
                                self.shape = Shapes::Circle;
                                painting.set_shape(Shapes::Circle);
                            }
                        });
                    if ui.button("✂")
                        .on_hover_text(format!("{} + {}", STD_HOTKEYS[4].0.to_string(), STD_HOTKEYS[4].1.to_string()))
                        .clicked() {
                        self.mycut = Some(MyCut::default());
                    }
                    if ui.button("💾")
                        .on_hover_text(format!("{} + {}", self.config.save_screenshot.1, self.config.save_screenshot.2))
                        .clicked() {
                        self.open_save = true;
                        let rgba = painting.edit_rgba(self.prova.clone().unwrap());
                        self.save_ly = Some(MySave::new(self.config.path.clone(),
                                                        imglib::format_from_string(self.config.format.as_str()).unwrap_or(AllFormats::PNG),
                                                        rgba.as_ref().unwrap(), self.tx.clone()));
                    }
                    ui.separator();
                    if ui.button("↩")
                        .on_hover_text(format!("{} + {}", STD_HOTKEYS[1].0.to_string(), STD_HOTKEYS[1].1.to_string()))
                        .clicked() {
                        painting.undo();
                    }
                    if ui.button("↪")
                        .on_hover_text(format!("{} + {}", STD_HOTKEYS[2].0.to_string(), STD_HOTKEYS[2].1.to_string()))
                        .clicked() {
                        painting.redo();
                    }
                    if ui.button("Clear")
                        .on_hover_text(format!("{} + {}", STD_HOTKEYS[3].0.to_string(), STD_HOTKEYS[3].1.to_string()))
                        .clicked() {
                        painting.clear();
                    }
                    if self.save_by_hk {
                        let rgba = painting.edit_rgba(self.prova.clone().unwrap());
                        MySave::save_image_tokio(rgba,
                                                 self.config.path.clone(),
                                                 "".to_string(),
                                                 self.format.clone(),
                                                 self.tx.clone(),
                                                 ui.ctx().clone());
                        self.save_by_hk = false;
                        self.saving = true;
                    }
                    if self.saving {
                        ui.spinner();
                        ui.label("Saving 😺 ...");
                    }
                    if self.copy {
                        let rgba = painting.edit_rgba(self.prova.clone().unwrap());
                        let img = rgba.as_ref().unwrap();
                        let img_data =  ImageData {
                            width: img.1 as usize,
                            height: img.2 as usize,
                            bytes: Cow::from(img.0.to_vec()),
                        };
                        if let Some(clip) = self.clipboard.as_mut() {
                            clip.set_image(img_data.to_owned_img()).unwrap_or(println!("Error in cpy on clipboard"));
                            println!("Image copied on clipboard");
                        }
                        self.copy = false;
                    }
                });
                self.hotkey_ly.edit_hotkeys(ui, self.painting.as_mut().unwrap(), &mut self.copy, &mut self.mycut);
            }
            else if self.mycut.is_some() {

                ui.horizontal(|ui| {
                    if ui.button("✔").clicked() {
                        let cutrect = self.mycut.clone().unwrap().get_cut_rect(Vec2::new(self.prova.clone().unwrap().width() as f32, self.prova.clone().unwrap().height() as f32));
                        self.img = Some(imageops::crop(&mut self.prova.clone().unwrap(),
                                                       cutrect.min.x.round() as u32,
                                                       cutrect.min.y.round() as u32,
                                                       cutrect.size().x.round() as u32,
                                                       cutrect.size().y.round() as u32)
                            .to_image());
                        if self.painting.is_some() {
                            self.painting.as_mut().unwrap().adapt_to_cut(self.mycut.clone().unwrap().get_rect());
                        }

                    } else if ui.button("✖").clicked() {
                        self.mycut = None;
                    }
                });

            }
            self.render_save(ui);
        });
    }

    pub fn render_settings(&mut self, ui: &mut Ui, frame: &mut Frame) {
        if ui.button("⚙").clicked() {
            self.open_settings = !self.open_settings;
        }
        if self.open_settings {
            Window::new("SETTINGS")
                .show(ui.ctx(), |ui| {
                    Grid::new("settings_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("Default Hot Key");
                            ui.menu_button("Hot Keys", |ui| {
                                CollapsingHeader::new("Default Hot Keys").show(ui, |ui| {
                                    ui.label(format!("Take Screenshot: {} + {}", self.config.take_screenshot.1, self.config.take_screenshot.2));
                                    ui.label(format!("Save Screenshot: {} + {}", self.config.save_screenshot.1, self.config.save_screenshot.2));
                                });
                                if ui.button("Change HotKey").clicked() {
                                    self.dim = Some((frame.info().window_info.size.x, frame.info().window_info.size.y));
                                    self.open_settings = false;
                                    restore_dim(&None, frame, Some(Layouts::Hotkey));
                                    self.layout = Layouts::Hotkey;
                                }
                            });
                            ui.end_row();

                            ui.label("Default Format");
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", self.format))
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.set_min_width(60.0);
                                    if ui.selectable_value(&mut self.format, AllFormats::PNG, "PNG").clicked() {
                                        self.config.format = "png".to_string();
                                        confy::store("screenshot", "screenshot", &self.config).unwrap();
                                    };
                                    if ui.selectable_value(&mut self.format, AllFormats::JPEG, "JPEG").clicked() {
                                        self.config.format = "jpeg".to_string();
                                        confy::store("screenshot", "screenshot", &self.config).unwrap();
                                    };
                                    if ui.selectable_value(&mut self.format, AllFormats::GIF, "GIF").clicked() {
                                        self.config.format = "gif".to_string();
                                        confy::store("screenshot", "screenshot", &self.config).unwrap();
                                    };
                                });
                            ui.end_row();
                            ui.label("Default Path");
                            ui.menu_button("Path", |ui| {
                                CollapsingHeader::new("Default Path").show(ui, |ui| {
                                    let path = self.config.path.clone().into_os_string().into_string().expect("Invalid path");
                                    ui.label(path);
                                });
                                if ui.button("Change Path").clicked() {
                                    self.open_settings = false;
                                    self.dim = Some((frame.info().window_info.size.x, frame.info().window_info.size.y));
                                    restore_dim(&None, frame, Some(Layouts::Path));
                                    self.layout = Layouts::Path;
                                }
                            });
                            ui.end_row();
                            if ui.button("About").clicked() {
                                self.open_settings = false;
                                self.dim = Some((frame.info().window_info.size.x, frame.info().window_info.size.y));
                                restore_dim(&None, frame, Some(Layouts::About));
                                self.layout = Layouts::About;
                            }
                            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui|{
                                if ui.button("↩").clicked() {
                                    self.open_settings = false;
                                }
                            });
                        });
                });
        }
    }

    pub fn render_save(&mut self, ui: &mut Ui){
        if self.open_save {
            Window::new("SAVE TO FILE").show(ui.ctx(), |ui| {
                if let Some(save_ly) = &mut self.save_ly {
                    save_ly.save_body(ui, &mut self.open_save, &mut self.saving);
                }
            });
        }
    }

    pub fn render_body(&mut self, ui: &mut Ui) {
        if let Some(buff) = self.img.take() {
            self.prova = Some(buff.clone());
            if self.mycut.is_none() {
                self.painting = Some(Painting::default());
            } else {
                self.mycut = None;
            }
            //renderizza immagine + tast
            //self.clicked = Some(ButtonClicked::Paint);
            //render_top_panel(ctx, _frame);
            self.texture = Some(ui.ctx().load_texture(
                "my-image",
                load_image_from_memory(buff),
                Default::default(),
            ));
        }
        if self.texture.is_none() {
            ui.centered_and_justified(|ui|{
                ui.group(|ui|{
                    ui.heading(format!("Take a screenshot:\t {} + {}", self.config.take_screenshot.1, self.config.take_screenshot.2));
                });
            });
        }
        if self.texture.is_some() {
            let painting = self.painting.as_mut().unwrap();
            ui.centered_and_justified(|ui|{
                ui.with_layout(Layout::default(), |ui| {
                    egui::Frame::canvas(ui.style())
                        .show(ui, |ui| {
                            if let Some(texture) = self.texture.as_ref() {
                                painting.ui_content(ui, &texture, &mut self.mycut);
                            }
                        });
                });
            });
        }
    }

    pub fn screen_layout(&mut self, ctx: &Context, _frame: &mut Frame) {
        let enabled = ctx.input(|i| i.time) - self.disabled_time > self.timeout;
        if enabled {
            _frame.set_visible(true);
            self.wait = false;
            self.screen_ly.screen_layout(ctx, _frame, &mut self.layout, &mut self.img,
                                         &mut self.clipboard);
        }
    }

    pub fn hotkey_layout(&mut self, ctx: &Context, _frame: &mut Frame){
        self.hotkey_ly.hotkey_layout(ctx,
                                     _frame,
                                     &mut self.config,
                                     &mut self.layout,
                                     &self.dim);
    }

    pub fn path_layout(&mut self, ctx: &Context, _frame: &mut Frame){
        self.path_ly.path_layout(ctx, _frame, &mut self.layout, &mut self.config, &self.dim);
    }

    pub fn about_layout(&mut self, ctx: &Context, _frame: &mut Frame){
        CentralPanel::default().show(ctx, |ui| {
            render_header(ui, "ABOUT");
            ui.vertical_centered(|ui|{
                ui.add_space(10.);
                ui.monospace("Screenshot App");
                ui.monospace("Version 1.0");
                ui.monospace("Developed by");
                //ui.indent("tab", |ui| {
                    ui.monospace("Fezza Lorenzo");
                    ui.monospace("Gruber Aurora");
                    ui.monospace("Pulinas Federico");
                //});
            });
            render_footer(ctx);
            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui|{
                if ui.button("↩").clicked() {
                    restore_dim(&self.dim, _frame, Some(Layouts::Home));
                    self.layout = Layouts::Home;
                }
            });
        });
    }
}

/*
pub fn render_top_panel(ctx :&Context, frame: &mut Frame){
    TopBottomPanel::top("top_panel").show(ctx, |ui|{
        //ui.add_space(10.);
        menu::bar(ui, |ui|{
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui|{
                let close_btn = ui.add(Button::new("❌"));
                if close_btn.clicked() {
                    frame.close();
                }
                ui.add(Button::new("‗"));
                let reduce_btn = ui.add(Button::new("_"));
                if reduce_btn.clicked() {
                    frame.set_minimized(true);
                }
            })
        });
        //ui.add_space(10.);
    });
}
 */

pub fn render_header(ui: &mut Ui, header: &str){
    ui.add_space(PADDING);
    ui.vertical_centered(|ui| {
        ui.heading(header);
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}

pub fn render_footer(ctx: &Context){
    TopBottomPanel::bottom("footer").show(ctx, |ui|{
        ui.vertical_centered(|ui|{
            ui.add_space(10.);
            ui.monospace("PROJECT");
            ui.hyperlink("https://gitlab.com/progetto-m1/rust");
            ui.monospace("2022/2023");
            ui.add_space(10.);
        })
    });
}

// Wrapper per la struttura KeyModifiers che implementa i trait From<String> e ToString
#[derive(PartialEq)]
pub struct KeyModifiersWrapper(pub KeyModifiers);

impl From<String> for KeyModifiersWrapper {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ALT" => KeyModifiersWrapper(KeyModifiers::ALT),
            "ALT_GRAPH" => KeyModifiersWrapper(KeyModifiers::ALT_GRAPH),
            "CAPS_LOCK" => KeyModifiersWrapper(KeyModifiers::CAPS_LOCK),
            "CONTROL" => KeyModifiersWrapper(KeyModifiers::CONTROL),
            "FN" => KeyModifiersWrapper(KeyModifiers::FN),
            "FN_LOCK" => KeyModifiersWrapper(KeyModifiers::FN_LOCK),
            "META" => KeyModifiersWrapper(KeyModifiers::META),
            "NUM_LOCK" => KeyModifiersWrapper(KeyModifiers::NUM_LOCK),
            "SCROLL_LOCK" => KeyModifiersWrapper(KeyModifiers::SCROLL_LOCK),
            "SHIFT" => KeyModifiersWrapper(KeyModifiers::SHIFT),
            "SYMBOL" => KeyModifiersWrapper(KeyModifiers::SYMBOL),
            "SYMBOL_LOCK" => KeyModifiersWrapper(KeyModifiers::SYMBOL_LOCK),
            "HYPER" => KeyModifiersWrapper(KeyModifiers::HYPER),
            "SUPER" => KeyModifiersWrapper(KeyModifiers::SUPER),
            _ => KeyModifiersWrapper(KeyModifiers::SHIFT), // Valore predefinito in caso di stringa sconosciuta
        }
    }
}

impl ToString for KeyModifiersWrapper {
    fn to_string(&self) -> String {
        match self.0 {
            KeyModifiers::ALT => "ALT".to_string(),
            KeyModifiers::ALT_GRAPH => "ALT_GRAPH".to_string(),
            KeyModifiers::CAPS_LOCK => "CAPS_LOCK".to_string(),
            KeyModifiers::CONTROL => "CONTROL".to_string(),
            KeyModifiers::FN => "FN".to_string(),
            KeyModifiers::FN_LOCK => "FN_LOCK".to_string(),
            KeyModifiers::META => "META".to_string(),
            KeyModifiers::NUM_LOCK => "NUM_LOCK".to_string(),
            KeyModifiers::SCROLL_LOCK => "SCROLL_LOCK".to_string(),
            KeyModifiers::SHIFT => "SHIFT".to_string(),
            KeyModifiers::SYMBOL => "SYMBOL".to_string(),
            KeyModifiers::SYMBOL_LOCK => "SYMBOL_LOCK".to_string(),
            KeyModifiers::HYPER => "HYPER".to_string(),
            KeyModifiers::SUPER => "SUPER".to_string(),
            _ => "SHIFT".to_string(), // Valore predefinito in caso di stringa sconosciuta

        }
    }
}

#[derive(PartialEq)]
pub struct KeyCodeWrapper(pub KeyCode);
impl From<String> for KeyCodeWrapper {
    fn from(s: String) -> Self {
        match s.as_str() {
            "~" => KeyCodeWrapper(KeyCode::Backquote),
            "\\" => KeyCodeWrapper(KeyCode::Backslash),
            "[" => KeyCodeWrapper(KeyCode::BracketLeft),
            "]" => KeyCodeWrapper(KeyCode::BracketRight),
            "," => KeyCodeWrapper(KeyCode::Comma),
            "0" => KeyCodeWrapper(KeyCode::Digit0),
            "1" => KeyCodeWrapper(KeyCode::Digit1),
            "2" => KeyCodeWrapper(KeyCode::Digit2),
            "3" => KeyCodeWrapper(KeyCode::Digit3),
            "4" => KeyCodeWrapper(KeyCode::Digit4),
            "5" => KeyCodeWrapper(KeyCode::Digit5),
            "6" => KeyCodeWrapper(KeyCode::Digit6),
            "7" => KeyCodeWrapper(KeyCode::Digit7),
            "8" => KeyCodeWrapper(KeyCode::Digit8),
            "9" => KeyCodeWrapper(KeyCode::Digit9),
            "=" => KeyCodeWrapper(KeyCode::Equal),
            "INTLBACKSLASH" => KeyCodeWrapper(KeyCode::IntlBackslash),
            "INTLRO" => KeyCodeWrapper(KeyCode::IntlRo),
            "INTLYEN" => KeyCodeWrapper(KeyCode::IntlYen),
            "A" => KeyCodeWrapper(KeyCode::KeyA),
            "B" => KeyCodeWrapper(KeyCode::KeyB),
            "C" => KeyCodeWrapper(KeyCode::KeyC),
            "D" => KeyCodeWrapper(KeyCode::KeyD),
            "E" => KeyCodeWrapper(KeyCode::KeyE),
            "F" => KeyCodeWrapper(KeyCode::KeyF),
            "G" => KeyCodeWrapper(KeyCode::KeyG),
            "H" => KeyCodeWrapper(KeyCode::KeyH),
            "I" => KeyCodeWrapper(KeyCode::KeyI),
            "J" => KeyCodeWrapper(KeyCode::KeyJ),
            "K" => KeyCodeWrapper(KeyCode::KeyK),
            "L" => KeyCodeWrapper(KeyCode::KeyL),
            "M" => KeyCodeWrapper(KeyCode::KeyM),
            "N" => KeyCodeWrapper(KeyCode::KeyN),
            "O" => KeyCodeWrapper(KeyCode::KeyO),
            "P" => KeyCodeWrapper(KeyCode::KeyP),
            "Q" => KeyCodeWrapper(KeyCode::KeyQ),
            "R" => KeyCodeWrapper(KeyCode::KeyR),
            "S" => KeyCodeWrapper(KeyCode::KeyS),
            "T" => KeyCodeWrapper(KeyCode::KeyT),
            "U" => KeyCodeWrapper(KeyCode::KeyU),
            "V" => KeyCodeWrapper(KeyCode::KeyV),
            "W" => KeyCodeWrapper(KeyCode::KeyW),
            "X" => KeyCodeWrapper(KeyCode::KeyX),
            "Y" => KeyCodeWrapper(KeyCode::KeyY),
            "Z" => KeyCodeWrapper(KeyCode::KeyZ),
            "-" => KeyCodeWrapper(KeyCode::Minus),
            "." => KeyCodeWrapper(KeyCode::Period),
            "'" => KeyCodeWrapper(KeyCode::Quote),
            ";" => KeyCodeWrapper(KeyCode::Semicolon),
            "/" => KeyCodeWrapper(KeyCode::Slash),
            _ => KeyCodeWrapper(KeyCode::KeyD), // Default value for unknown string
        }
    }
}

impl ToString for KeyCodeWrapper {
    fn to_string(&self) -> String {
        match self.0 {
            KeyCode::Backquote => "~".to_string(),
            KeyCode::Backslash => "\\".to_string(),
            KeyCode::BracketLeft => "[".to_string(),
            KeyCode::BracketRight => "]".to_string(),
            KeyCode::Comma => ",".to_string(),
            KeyCode::Digit0 => "0".to_string(),
            KeyCode::Digit1 => "1".to_string(),
            KeyCode::Digit2 => "2".to_string(),
            KeyCode::Digit3 => "3".to_string(),
            KeyCode::Digit4 => "4".to_string(),
            KeyCode::Digit5 => "5".to_string(),
            KeyCode::Digit6 => "6".to_string(),
            KeyCode::Digit7 => "7".to_string(),
            KeyCode::Digit8 => "8".to_string(),
            KeyCode::Digit9 => "9".to_string(),
            KeyCode::Equal => "=".to_string(),
            KeyCode::IntlBackslash => "INTLBACKSLASH".to_string(),
            KeyCode::IntlRo => "INTLRO".to_string(),
            KeyCode::IntlYen => "INTLYEN".to_string(),
            KeyCode::KeyA => "A".to_string(),
            KeyCode::KeyB => "B".to_string(),
            KeyCode::KeyC => "C".to_string(),
            KeyCode::KeyD => "D".to_string(),
            KeyCode::KeyE => "E".to_string(),
            KeyCode::KeyF => "F".to_string(),
            KeyCode::KeyG => "G".to_string(),
            KeyCode::KeyH => "H".to_string(),
            KeyCode::KeyI => "I".to_string(),
            KeyCode::KeyJ => "J".to_string(),
            KeyCode::KeyK => "K".to_string(),
            KeyCode::KeyL => "L".to_string(),
            KeyCode::KeyM => "M".to_string(),
            KeyCode::KeyN => "N".to_string(),
            KeyCode::KeyO => "O".to_string(),
            KeyCode::KeyP => "P".to_string(),
            KeyCode::KeyQ => "Q".to_string(),
            KeyCode::KeyR => "R".to_string(),
            KeyCode::KeyS => "S".to_string(),
            KeyCode::KeyT => "T".to_string(),
            KeyCode::KeyU => "U".to_string(),
            KeyCode::KeyV => "V".to_string(),
            KeyCode::KeyW => "W".to_string(),
            KeyCode::KeyX => "X".to_string(),
            KeyCode::KeyY => "Y".to_string(),
            KeyCode::KeyZ => "Z".to_string(),
            KeyCode::Minus => "-".to_string(),
            KeyCode::Period => ".".to_string(),
            KeyCode::Quote => "'".to_string(),
            KeyCode::Semicolon => ";".to_string(),
            KeyCode::Slash => "/".to_string(),
            _ => String::new(), // Default value for unknown KeyCode
        }
    }
}
