use chrono::{Local, DateTime};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;
use egui::{Color32, Grid};
use image::RgbaImage;
use crate::myapp::imglib::AllFormats;
use crate::myapp::PADDING;
use crate::myapp::pathlib::MyPath;

pub struct MySave {
    path: MyPath,
    name: String,
    format: AllFormats,
    rgba: Option<(Vec<u8>, u32, u32)>,
    name_error: bool,
    tx: Sender<Duration>,
}

impl MySave {
    pub fn new(path: PathBuf, format: AllFormats, rgba: &(Vec<u8>, u32, u32), tx: Sender<Duration>) -> Self {
        let current_datetime: DateTime<Local> = Local::now();
        let formatted_datetime = current_datetime.format("%Y-%m-%d_%H%M%S");
        let name = formatted_datetime.to_string();
        let path = MyPath::new(path);
        Self {
            path,
            name,
            format,
            rgba: Some(rgba.clone()),
            name_error: false,
            tx,
        }
    }

    pub fn save_body(&mut self, ui: &mut egui::Ui, open_save: &mut bool, saving: &mut bool) {
        let paths = self.path.path.clone();
        if self.name_error {
            ui.colored_label(Color32::RED, "Name syntax error");
            if Self::is_file_name_valid(self.name.as_str()){
                self.name_error = false;
            }
        }
        ui.add_space(PADDING);
        Grid::new("settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.name);
                ui.end_row();
                ui.label("Format");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.format))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(&mut self.format, AllFormats::PNG, "PNG");
                        ui.selectable_value(&mut self.format, AllFormats::JPEG, "JPEG");
                        ui.selectable_value(&mut self.format, AllFormats::GIF, "GIF");
                    });
                ui.end_row();
                ui.label("Destination Path :");
                ui.label(paths.clone().into_os_string().into_string().unwrap());
            });
        ui.add_space(2. * PADDING);
        self.path.directories_tree(ui);
        ui.add_space(2. * PADDING);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                //controllo se il nome inserito é corretto:
                if Self::is_file_name_valid(self.name.as_str()){
                    //funzione per salvare
                    //self.save_image();
                    MySave::save_image_tokio(self.rgba.clone(),
                                             self.path.path.clone(),
                                             self.name.clone(),
                                             self.format.clone(),
                                             self.tx.clone(),
                                             ui.ctx().clone());
                    *saving = true;
                    *open_save = false;
                }
                else{
                    //banner di errore nel nome
                    self.name_error = true;
                }
            }
            if ui.button("Cancel").clicked() {
                *open_save = false;
            }
        });
    }

    pub fn is_file_name_valid(file_name: &str) -> bool {
        file_name.chars().all(|c| c.is_ascii_alphanumeric() || c=='-' || c=='_')
        //file_name.chars().all(|c| c.is_ascii_alphanumeric())
    }

    pub fn save_image_tokio(rgba: Option<(Vec<u8>, u32, u32)>, path: PathBuf,
                            name: String, format: AllFormats,
                            tx: Sender<Duration>, ctx: egui::Context) {// rgba, path, name, format, tx
        tokio::spawn(async move {
            let instant = std::time::Instant::now();
            let (rgba, w, h) = rgba.clone().take().unwrap();
            let mut p = path.clone();
            let mut name = name.clone();
            if name.trim() == "" {
                let current_datetime: DateTime<Local> = Local::now();
                let formatted_datetime = current_datetime.format("%Y-%m-%d_%H%M%S");
                name = formatted_datetime.to_string();
            }

            p.push(format!("{}.{}", name, format.to_string()));
            let p = Self::generate_unique_filename(&p);

            if format == AllFormats::GIF {
                let reuced_width = w / 4;
                let reuced_height = h / 4;
                let rgba_image: RgbaImage = image::ImageBuffer::from_raw(
                    w,
                    h,
                    rgba.to_owned(),
                )
                    .unwrap();
                let reduced_image = image::imageops::resize(&rgba_image, reuced_width, reuced_height, image::imageops::FilterType::Lanczos3);
                reduced_image.save_with_format(p, image::ImageFormat::Gif).unwrap();
            } else {
                image::save_buffer(p, rgba.as_slice(), w, h, image::ColorType::Rgba8).unwrap();
            }
            let duration = instant.elapsed();
            //println!("Time elapsed in expensive_function() is: {:?}", duration);
            tx.send(duration).unwrap();
            ctx.request_repaint();
        });
    }
/*
    pub fn save_image(&mut self) {
        let instant = std::time::Instant::now();
        let (rgba,w, h) = self.rgba.clone().take().unwrap();
        let mut p = self.path.path.clone();
        if self.name.trim() == "" {
            let current_datetime: DateTime<Local> = Local::now();
            let formatted_datetime = current_datetime.format("%Y-%m-%d_%H%M%S");
            self.name = formatted_datetime.to_string();
        }

        p.push(format!("{}.{}", self.name, self.format.to_string()));
        let p = Self::generate_unique_filename(&p);

        if self.format == AllFormats::GIF {
            let reuced_width = w / 4;
            let reuced_height = h / 4;
            let rgba_image: RgbaImage = image::ImageBuffer::from_raw(
                w,
                h,
                rgba.to_owned(),
            )
                .unwrap();
            let reduced_image = image::imageops::resize(&rgba_image, reuced_width, reuced_height, image::imageops::FilterType::Lanczos3);
            reduced_image.save_with_format(p, image::ImageFormat::Gif).unwrap();
        }
        else{
            image::save_buffer(p, rgba.as_slice(), w, h, image::ColorType::Rgba8).unwrap();
        }
        let duration = instant.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
    }
 */
/*
    pub fn save_image_by_hotkey_tokio(rgba: (Vec<u8>, u32, u32), format: String, path: PathBuf, tx: Sender<Duration>, ctx: egui::Context) {
        tokio::spawn(async move {
            let instant = std::time::Instant::now();
            let (rgba, w, h) = rgba;
            let mut p = path.clone();
            let current_datetime: DateTime<Local> = Local::now();
            let formatted_datetime = current_datetime.format("%Y-%m-%d_%H%M%S");
            let name = formatted_datetime.to_string();
            p.push(format!("{}.{}", name, format.to_string()));
            let p = Self::generate_unique_filename(&p);

            if format == "gif" {
                let reuced_width = w / 4;
                let reuced_height = h / 4;
                let rgba_image: RgbaImage = image::ImageBuffer::from_raw(
                    w,
                    h,
                    rgba.to_owned(),
                )
                    .unwrap();
                let reduced_image = image::imageops::resize(&rgba_image, reuced_width, reuced_height, image::imageops::FilterType::Lanczos3);
                reduced_image.save_with_format(p, image::ImageFormat::Gif).unwrap();
            } else {
                image::save_buffer(p, rgba.as_slice(), w, h, image::ColorType::Rgba8).unwrap();
            }
            let duration = instant.elapsed();
            //println!("Time elapsed in expensive_function() is: {:?}", duration);
            tx.send(duration).unwrap();
            ctx.request_repaint();
        });
    }
*/
/*
    pub fn save_image_by_hotkey(rgba: (Vec<u8>, u32, u32), format: AllFormats, path: PathBuf) {
        let instant = std::time::Instant::now();
        let (rgba,w, h) = rgba;
        let mut p = path.clone();
        let current_datetime: DateTime<Local> = Local::now();
        let formatted_datetime = current_datetime.format("%Y-%m-%d_%H%M%S");
        let name = formatted_datetime.to_string();
        p.push(format!("{}.{}", name, format.to_string()));
        let p = Self::generate_unique_filename(&p);

        if format == AllFormats::GIF {
            let reuced_width = w / 4;
            let reuced_height = h / 4;
            let rgba_image: RgbaImage = image::ImageBuffer::from_raw(
                w,
                h,
                rgba.to_owned(),
            )
                .unwrap();
            let reduced_image = image::imageops::resize(&rgba_image, reuced_width, reuced_height, image::imageops::FilterType::Lanczos3);
            reduced_image.save_with_format(p, image::ImageFormat::Gif).unwrap();
        }
        else{
            image::save_buffer(p, rgba.as_slice(), w, h, image::ColorType::Rgba8).unwrap();
        }
        let duration = instant.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
    }
*/
    fn generate_unique_filename(path: &PathBuf) -> PathBuf {
        let mut attempt = 0;
        let mut new_path = path.clone();

        while new_path.exists() {
            attempt += 1;
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            let extension = path.extension().unwrap().to_string_lossy();
            let new_filename = format!("{}_({}).{}", file_stem, attempt, extension);
            new_path.set_file_name(new_filename);
        }

        new_path
    }

}
