use std::fs;
use std::path::PathBuf;
use eframe::Frame;
use egui::{CollapsingHeader, Ui, CentralPanel, Context, ScrollArea, Color32};
use crate::myapp::{Config, Layouts, PADDING, render_header};
use crate::myapp::imglib::restore_dim;


pub struct MyPath{
    pub path: PathBuf
}

impl MyPath {
    pub fn new(path: PathBuf) -> Self {
        Self{ path }
    }
    pub fn path_layout(&mut self, ctx: &Context, _frame: &mut Frame, layout: &mut Layouts, config: &mut Config, dim: &Option<(f32, f32)>){
        //_frame.set_window_size(egui::vec2(300.0, 360.0));
        //render_top_panel(ctx, _frame);
        CentralPanel::default().show(ctx, |ui| {
            render_header(ui, "PATH");
            self.render_path_body(_frame, ui, config, layout, dim);
        });
    }

    pub fn render_path_body(&mut self, _frame: &mut Frame, ui: &mut Ui, config: &mut Config, layout: &mut Layouts, dim: &Option<(f32, f32)>) {
        //let paths = self.path.clone();
        //ui.horizontal(|ui| {
        ui.add_space(2. * PADDING);
        ui.label(format!("Destination Path : {}", self.path.clone().into_os_string().into_string().unwrap()));
        ui.add_space(2. * PADDING);
        ui.separator();
        ui.add_space(2. * PADDING);
        self.directories_tree(ui);
        ui.add_space(2. * PADDING);
        ui.separator();
        ui.add_space(2. * PADDING);
        ui.horizontal(|ui|{
            if ui.button("Change Path").clicked() {
                config.path = self.path.clone();
                confy::store("screenshot", "screenshot", &config).unwrap();
                restore_dim(dim, _frame, Some(Layouts::Home));
                *layout = Layouts::Home;
            }
            if ui.button("↩").clicked() {
                self.path = config.path.clone();
                restore_dim(dim, _frame, Some(Layouts::Home));
                *layout = Layouts::Home;
            }
        });
    }
    pub fn directories_tree(&mut self, ui: &mut Ui) {
        let paths = self.path.clone();
        ScrollArea::new([false, true]).max_height(200.).show(ui, |ui|{
            //CollapsingHeader::new(format!("{}", parent_dir.file_name().unwrap().to_string_lossy()))
            CollapsingHeader::new("Change Path")
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(parent_dir) = paths.parent() {
                        if let Some(file_name) = parent_dir.file_name() {
                            if ui.button(format!("🗁 {}", file_name.to_string_lossy())).clicked() {
                                println!("Path: {}", parent_dir.display());
                                self.path = parent_dir.to_path_buf();
                            }
                        }
                        else {
                            ui.colored_label(Color32::LIGHT_RED, "You can't go back!");
                        }
                    }
                    if let Some(file_name) = paths.file_name() {
                        CollapsingHeader::new(format!("🗁 {} (Current Path)", file_name.to_string_lossy()))
                            .default_open(true)
                            .show(ui, |ui| {
                                if let Ok(entries) = fs::read_dir(&paths.clone()) {
                                    for entry in entries {
                                        if let Ok(entry) = entry {
                                            if entry.path().is_dir() {
                                                if ui.button(format!("🗁 {}", entry.file_name().to_string_lossy())).clicked() {
                                                    println!("Path: {}", entry.path().display());
                                                    self.path = entry.path().to_path_buf();
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    }
                });
        });

    }
}


