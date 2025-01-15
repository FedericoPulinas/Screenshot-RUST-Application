use eframe::Frame;
use egui::{CentralPanel, Context, Ui, Grid, Color32, Layout, Align};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{Modifiers as KeyModifiers, Code as KeyCode, HotKey}};
use crate::myapp::{Config, Layouts, KeyModifiersWrapper, KeyCodeWrapper, render_header, PADDING};
use crate::myapp::imglib::restore_dim;
use crate::myapp::cutlib::MyCut;
use crate::myapp::paintlib::Painting;

pub enum ChangeState {
    Unregistered,
    Registered,
    Saved,
}

pub struct HotKeyData {
    pub hk: HotKey,
    pub code: KeyCodeWrapper,
    pub modifiers: KeyModifiersWrapper,
    pub state: ChangeState,
}

impl HotKeyData {
    /** per cambiare hotkey **/
    pub fn modify_hk(&mut self, a_reg: &mut bool, diff: &mut bool, manager: &mut GlobalHotKeyManager,
                     ui: &mut Ui, c: &(u32, String, String), other: &HotKeyData, en: &bool, saving: &mut bool){
        match self.state {
            ChangeState::Registered => {
                ui.colored_label(Color32::LIGHT_YELLOW,"Type the hotkey again to save it");
                if ui.ctx().input(|i| { i.keys_down.len() > 0 && (i.modifiers.command | i.modifiers.ctrl | i.modifiers.alt | i.modifiers.shift) }) {
                    ui.colored_label(Color32::LIGHT_RED, format!("Attention: The hotkey you are typing cannot be saved:\n\
                        The keys you are using ({} + {}) might not work correctly or may not correspond\
                         to the selected ones.", ui.ctx().input(|i| {
                        if i.modifiers.ctrl {
                            "Ctrl"
                        } else if i.modifiers.command {
                            "Command"
                        } else if i.modifiers.alt {
                            "Alt"
                        } else if i.modifiers.shift {
                            "Shift"
                        } else {
                            ""
                        }
                    }), ui.ctx().input(|i| {
                        i.keys_down.iter().map(
                            |k| k.name().to_string()).collect::<Vec<String>>().join(", ")
                    })));
                    ui.add_space(2. * PADDING);
                }
            }
            ChangeState::Saved => {ui.colored_label(Color32::LIGHT_GREEN, format!("HOTKEY CORRECTLY SAVED"));}
            _ => {},
        }
        Grid::new(format!("Change HotKey {}", c.0))
            .num_columns(2)
            .max_col_width(150.0)
            .min_col_width(150.0)
            .spacing([40.0, 7.0])
            .striped(false)
            .show(ui, |ui| {
                match self.state {
                    ChangeState::Unregistered => {
                        ui.label("Change Modifier");
                        egui::ComboBox::new(format!("M{}", self.hk.id()), "")
                            .selected_text(format!("{}", self.modifiers.to_string()))
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(true);
                                for modifier in ALL_KEY_MODIFIERS.iter() {
                                    if ui.selectable_value(&mut self.modifiers.to_string(), KeyModifiersWrapper(*modifier).to_string(), KeyModifiersWrapper(*modifier).to_string()).clicked() {
                                        self.modifiers = KeyModifiersWrapper(*modifier);
                                    }
                                }
                            });
                        ui.end_row();
                        ui.label("Change Code");
                        if *en {
                            ui.ctx().input(|i| {
                                if i.keys_down.len() > 0 {
                                    self.code = KeyCodeWrapper::from(i.keys_down.iter().last().unwrap().name().to_string());
                                }
                            });
                        }
                        if ui.button(self.code.to_string()).clicked(){};
                        ui.end_row();
                        self.reder_progress(ui);
                        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui|{
                            if ui.button("Register").clicked() {
                                if !self.are_hotkeys_valid(other) || is_std_hk(&self.modifiers, &self.code) {
                                    *diff = false;
                                } else {
                                    //devo unregistrare l'hotkey gia registrata che nel caso della register
                                    //puo essere solo quella delle config, perche se ho fatto annulla rimetto
                                    //quella vecchia, che é sempre una che viene dalle config
                                    MyHotKey::register(manager, self, &c, a_reg);
                                    if !*a_reg { self.state = ChangeState::Registered; }
                                }
                            }
                        });
                    }
                    ChangeState::Registered => {
                        *saving = true;
                        ui.label("The hotkey you have registered is : ");
                        ui.label(format!("{} + {}", self.modifiers.to_string(), self.code.to_string()));
                        ui.end_row();
                        self.reder_progress(ui);
                        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                            if ui.button("Cancel").clicked() {
                                manager.unregister(
                                    HotKey::new(
                                        Some(self.modifiers.0),
                                        self.code.0
                                    )
                                ).expect("Unable to unregister hotkey");
                                let modifiers = KeyModifiersWrapper::from(c.1.to_string());
                                let code = KeyCodeWrapper::from(c.2.to_string());
                                let hotkey = HotKey::new(Some(modifiers.0), code.0);
                                manager.register(hotkey).expect("Unable to register hotkey");
                                *self = HotKeyData { hk: hotkey, code, modifiers, state: ChangeState::Unregistered };
                                *saving = false;
                            }
                        });
                    }
                    ChangeState::Saved => {
                        ui.label("Saved Modifier:");
                        if ui.button(self.modifiers.to_string()).clicked() {}
                        ui.end_row();
                        ui.label("Saved Code:");
                        if ui.button(self.code.to_string()).clicked() {}
                        ui.end_row();
                        *saving = false;
                        self.reder_progress(ui);
                        ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                            if ui.button("Edit").clicked() {
                                self.state = ChangeState::Unregistered;
                            }
                        });

                    }
                }
            });
    }

    pub fn reder_progress(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui|{
            egui::ScrollArea::horizontal().id_source(format!("S{}", self.hk.id())).max_width(100.)
                .horizontal_scroll_offset(match self.state {
                    ChangeState::Unregistered => {0.},
                    ChangeState::Registered => {100.},
                    ChangeState::Saved => {200.},
                })
                .show(ui, |ui| {
                    ui.allocate_space(egui::vec2(300., 4.));
                });
        });
    }

    pub fn are_hotkeys_valid(&self, other: &HotKeyData) -> bool {
        if self.modifiers == other.modifiers &&
            self.code == other.code {
            return false;
        }
        true
    }

    pub fn change_hotkey(&mut self, config: &mut (u32, String, String), other: &HotKeyData, diff: &mut bool) {
        if !self.are_hotkeys_valid(other) {
            *diff = false;
        } else {
            println!("Old: {:?} {:?}", config.1, config.2);
            config.1 = self.modifiers.to_string();
            config.2 = self.code.to_string();
            self.hk = HotKey::new(
                Some(KeyModifiersWrapper::from(config.1.clone()).0),
                KeyCodeWrapper::from(config.2.clone()).0
            );
            config.0 =  self.hk.id();
            println!("New Take Screenshot: {:?} {:?} {:?}", config.0, config.1, config.2);
        }
    }

    pub fn is_already_reg(&self, c: &(u32, String, String)) -> Option<HotKey> {
        let hk = HotKey::new(
            Some(self.modifiers.0),
            self.code.0
        );
        if c.0 != hk.id() {
            return Some(hk);
        }
        None
    }
}
#[derive(PartialEq)]
pub enum Radio{
    Take,
    Save,
}

pub struct MyHotKey{
    manager: GlobalHotKeyManager,
    take_screenshot: HotKeyData,
    save_screenshot: HotKeyData,
    are_different: bool,
    already_reg: bool,
    radio: Radio,
    saving: bool,
    old_ly: Layouts,
    is_pressed: bool,
}

impl MyHotKey {
    pub fn new(take_screenshot: (u32, String, String), save_screenshot: (u32, String, String)) -> Self {
        let manager = GlobalHotKeyManager::new().unwrap();

        let modifiers = KeyModifiersWrapper::from(take_screenshot.1);
        let code = KeyCodeWrapper::from(take_screenshot.2);
        let hotkey = HotKey::new( Some(modifiers.0), code.0);
        manager.register(hotkey).expect("Unable to register hotkey");
        let take_screenshot = HotKeyData{hk: hotkey, code, modifiers, state: ChangeState::Unregistered};

        let modifiers = KeyModifiersWrapper::from(save_screenshot.1);
        let code = KeyCodeWrapper::from(save_screenshot.2);
        let hotkey = HotKey::new( Some(modifiers.0), code.0);
        manager.register(hotkey).expect("Unable to register hotkey");
        let save_screenshot = HotKeyData{hk: hotkey, code, modifiers, state: ChangeState::Unregistered};
        Self {
            manager,
            take_screenshot,
            save_screenshot,
            are_different: true,
            already_reg: false,
            radio: Radio::Take,
            saving: false,
            old_ly: Layouts::Home,
            is_pressed: false
        }
    }

    /**schermata hotkey**/
    pub fn hotkey_layout(&mut self, ctx: &Context, _frame: &mut Frame, config: &mut Config, layout: &mut Layouts, dim: &Option<(f32, f32)>){
        CentralPanel::default().show(ctx, |ui| {
            render_header(ui, "HOT KEY");
            self.render_hotkey_body(ui, config, layout, _frame, dim);
        });
    }

    /** per cambiare hotkey **/
    pub fn render_hotkey_body(&mut self, ui: &mut Ui, config: &mut Config, layout: &mut Layouts, _frame: &mut Frame, dim: &Option<(f32, f32)>) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui|{
                if !self.are_different {
                    ui.colored_label(Color32::LIGHT_RED, "You cannot choose the same hotkey used for other functions");
                    ui.end_row();
                    if self.take_screenshot.are_hotkeys_valid(&self.save_screenshot) {
                        match self.radio {
                            Radio::Take => {
                                if !is_std_hk(&self.take_screenshot.modifiers, &self.take_screenshot.code) {
                                    self.are_different = true;
                                }
                            }
                            Radio::Save => {
                                if !is_std_hk(&self.save_screenshot.modifiers, &self.save_screenshot.code) {
                                    self.are_different = true;
                                }
                            }
                        }
                    }
                }
                if self.already_reg {
                    ui.colored_label(Color32::LIGHT_RED, "The hotkey you want to register is already in use");
                    match self.radio {
                        Radio::Take => {
                            if self.take_screenshot.is_already_reg(&config.take_screenshot).is_some() {
                                self.already_reg = false;
                            }
                        }
                        Radio::Save => {
                            if self.save_screenshot.is_already_reg(&config.save_screenshot).is_some() {
                                self.already_reg = false;
                            }
                        }
                    }
                    ui.end_row();
                }
            });
            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                ui.set_enabled(!self.saving);
                if ui.button("↩").clicked() {
                    restore_dim(dim, _frame, Some(Layouts::Home));
                    *layout = Layouts::Home;
                }
            });
        });
        self.render_form(ui, config);

        ui.group(|ui|{
            Grid::new("OtherHK")
                .num_columns(2)
                .max_col_width(150.0)
                .min_col_width(150.0)
                .spacing([40.0, 7.0])
                .striped(false)
                .show(ui, |ui| {
                    ui.label("Other hot keys: ");
                    ui.end_row();
                    ui.label("- Copy :");
                    ui.label(format!("{} + {}", STD_HOTKEYS[0].0.to_string(), STD_HOTKEYS[0].1.to_string()));
                    ui.end_row();
                    ui.label("- Undo :");
                    ui.label(format!("{} + {}", STD_HOTKEYS[1].0.to_string(), STD_HOTKEYS[1].1.to_string()));
                    ui.end_row();
                    ui.label("- Redo :");
                    ui.label(format!("{} + {}", STD_HOTKEYS[2].0.to_string(), STD_HOTKEYS[2].1.to_string()));
                    ui.end_row();
                    ui.label("- Clear :");
                    ui.label(format!("{} + {}", STD_HOTKEYS[3].0.to_string(), STD_HOTKEYS[3].1.to_string()));
                    ui.end_row();
                    ui.label("- Cut :");
                    ui.label(format!("{} + {}", STD_HOTKEYS[4].0.to_string(), STD_HOTKEYS[4].1.to_string()));
                });
        });
        ui.add_space( 3. * PADDING);
    }

    pub fn render_form(&mut self , ui: &mut Ui, config: &mut Config){
        ui.group(|ui|{
            let enabled = self.radio == Radio::Take;
            ui.set_enabled(!self.saving || enabled);
            ui.radio_value(&mut self.radio, Radio::Take, "Take Screenshot");
            ui.end_row();
            ui.group(|ui|{
                ui.set_enabled(enabled);
                self.take_screenshot.modify_hk( &mut self.already_reg, &mut self.are_different,
                                                &mut self.manager, ui, &config.take_screenshot,
                                                &self.save_screenshot, &enabled, &mut self.saving);
            });
        });
        ui.end_row();
        ui.group(|ui| {
            let enabled = self.radio == Radio::Save;
            ui.set_enabled(!self.saving || enabled);
            ui.radio_value(&mut self.radio, Radio::Save, "Save Screenshot").enabled();
            ui.end_row();
            ui.group(|ui| {
                ui.set_enabled(enabled);
                self.save_screenshot.modify_hk(&mut self.already_reg, &mut self.are_different,
                                               &mut self.manager, ui, &config.save_screenshot,
                                               &self.take_screenshot, &enabled, &mut self.saving);
            });
        });
        ui.end_row();
    }

    pub fn register(manager: &mut GlobalHotKeyManager, hot_key_data: &mut HotKeyData, c: &(u32, String, String), already_reg: &mut bool) {
        if let Some(hk) = hot_key_data.is_already_reg(c) {
            *already_reg = false;   //cosi dico che non é quella vecchia
            let modifiers = KeyModifiersWrapper::from(c.1.to_string());
            let code = KeyCodeWrapper::from(c.2.to_string());
            manager.unregister(HotKey::new( Some(modifiers.0), code.0))
                .expect("Unable to unregister hotkey");
            hot_key_data.hk = hk.clone();
            match manager.register(hk) {
                Ok(_) => println!("Hotkey registered"),
                Err(e) => println!("Error registering hotkey: {:?}", e)
            }
            println!("New Take Screenshot: {:?} ", hk);
        }
        else {
            *already_reg = true;
        }
    }

    pub fn match_event(&mut self, event: GlobalHotKeyEvent, _frame: &mut Frame, layout: &mut Layouts, disabled_time: &mut f64,
                       dim: &Option<(f32, f32)>, is_taken: bool,
                       ctx: &Context, saving: &mut bool, config: &mut Config, save_by_hk :&mut bool) {

        if event.id == self.take_screenshot.hk.id() &&  self.take_screenshot.hk.id() == config.take_screenshot.0 {
            if *layout != Layouts::Hotkey {
                match layout {
                    Layouts::Screenshot => { *layout = self.old_ly },
                    _ => {
                        _frame.set_visible(false);
                        *disabled_time = ctx.input(|i| i.time);
                        self.old_ly = *layout;
                        *layout = Layouts::Screenshot;
                    }
                }
                _frame.set_minimized(false);
                _frame.set_decorations(true);
                restore_dim(dim, _frame, Some(*layout));
                _frame.focus();
            }
        }
        else {
            let enabled = self.radio == Radio::Take;
            if event.id == self.take_screenshot.hk.id() &&  self.take_screenshot.hk.id() != config.take_screenshot.0  && enabled {
                self.take_screenshot.change_hotkey(&mut config.take_screenshot,
                                                   &self.save_screenshot, &mut self.are_different);
                confy::store("screenshot", "screenshot", &config).unwrap();
                self.take_screenshot.state = ChangeState::Saved;
            }
            else {
                if event.id == self.save_screenshot.hk.id() &&  self.save_screenshot.hk.id() == config.save_screenshot.0 {
                    match layout {
                        Layouts::Home => {
                            if is_taken {
                                *saving = true;
                                *save_by_hk = true;
                            }
                            else { eprintln!("No screenshot taken"); }
                        },
                        _ => { }
                    }
                }
                else {
                    let enabled = self.radio == Radio::Save;
                    if event.id == self.save_screenshot.hk.id() && self.save_screenshot.hk.id() != config.save_screenshot.0 && enabled {
                        self.save_screenshot.change_hotkey(&mut config.save_screenshot,
                                                           &self.take_screenshot, &mut self.are_different);
                        confy::store("screenshot", "screenshot", &config).unwrap();
                        self.save_screenshot.state = ChangeState::Saved;
                    }
                }
            }
        }
    }
    pub fn edit_hotkeys(&mut self, ui: &mut Ui, painting: &mut Painting, copy: &mut bool, mycut: &mut Option<MyCut>){
        if ui.ctx().input(|i| {i.keys_down.len()>0}) {
            ui.ctx().input(|i| {
                for (_, keycode) in STD_HOTKEYS.iter() {
                    if i.keys_down.iter().find(|k| k.symbol_or_name() == keycode.to_string()).is_some() &&
                        (i.modifiers.ctrl | i.modifiers.command) {
                        if !self.is_pressed {
                            self.is_pressed = true;

                            match keycode {
                                KeyCodeWrapper(KeyCode::KeyC) => {
                                    *copy = true;
                                },
                                KeyCodeWrapper(KeyCode::KeyZ) => {
                                    painting.undo();
                                },
                                KeyCodeWrapper(KeyCode::KeyY) => {
                                    painting.redo();
                                },
                                KeyCodeWrapper(KeyCode::KeyD) => {
                                    painting.clear();
                                },
                                KeyCodeWrapper(KeyCode::KeyT) => {
                                    //TODO
                                    *mycut = Some(MyCut::default());
                                },
                                _ => {}
                            }
                        }
                    }
                }
            });
        }
        else{
            self.is_pressed = false;
        }
    }
}

pub const ALL_KEY_MODIFIERS: &'static [KeyModifiers] = &[
    KeyModifiers::ALT,
    KeyModifiers::CONTROL,
    KeyModifiers::FN,
    KeyModifiers::SHIFT,
];

pub const STD_HOTKEYS: &'static [(KeyModifiersWrapper, KeyCodeWrapper)] = &[
    (KeyModifiersWrapper(KeyModifiers::CONTROL), KeyCodeWrapper(KeyCode::KeyC)),  //copy
    (KeyModifiersWrapper(KeyModifiers::CONTROL), KeyCodeWrapper(KeyCode::KeyZ)),  //undo
    (KeyModifiersWrapper(KeyModifiers::CONTROL), KeyCodeWrapper(KeyCode::KeyY)),  //redo
    (KeyModifiersWrapper(KeyModifiers::CONTROL), KeyCodeWrapper(KeyCode::KeyD)),  //clear
    (KeyModifiersWrapper(KeyModifiers::CONTROL), KeyCodeWrapper(KeyCode::KeyT)),  //cut
];

pub fn is_std_hk(modifiers: &KeyModifiersWrapper, code: &KeyCodeWrapper) -> bool {
    for hk in STD_HOTKEYS.iter() {
        if *modifiers == hk.0 && *code == hk.1 {
            return true;
        }
    }
    false
}
