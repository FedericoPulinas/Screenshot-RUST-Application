use image::{RgbaImage};
use egui::{ImageData};
use eframe::epaint::ColorImage;
use eframe::Frame;
use crate::myapp::Layouts;


#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AllFormats {
    PNG,
    JPEG,
    GIF
}

impl ToString for AllFormats {
    fn to_string(&self) -> String {
        match self {
            AllFormats::PNG => "png".to_string(),
            AllFormats::JPEG => "jpeg".to_string(),
            AllFormats::GIF => "gif".to_string(),
        }
    }
}

//non serve per ora, ma potrebbe servire dopo
/*
pub fn load_image_from_path(path: &Path) -> Result<ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}*/
/*
pub fn get_image(filepath: &str) -> ImageData {
    let fp = Path::new(filepath);
    let color_image = load_image_from_path(&fp).unwrap();
    let img = ImageData::from(color_image);
    img
}*/
/*
pub fn scale_vec2(vec: egui::Vec2, scale_factor: f32) -> egui::Vec2 {
    egui::Vec2::new(vec.x * scale_factor, vec.y * scale_factor)
}
 */

pub fn load_image_from_memory(image_data: RgbaImage) -> ImageData {
    // let image = image::load_from_memory(image_data).expect("Error in the buffer");
    let size = [image_data.width() as _, image_data.height() as _];
    // let image_buffer = image.to_rgba8();
    let pixels: image::FlatSamples<&[u8]> = image_data.as_flat_samples();
    let color_image = ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    );
    let img = ImageData::from(color_image);
    img
}

pub fn format_from_string(format: &str) -> Option<AllFormats> {
    match format {
        "png" => Some(AllFormats::PNG),
        "jpeg" => Some(AllFormats::JPEG),
        "gif" => Some(AllFormats::GIF),
        _ => None,
    }
}

pub fn restore_dim(dim: &Option<(f32, f32)>, frame: &mut Frame, ly: Option<Layouts>) {
    if ly.is_some(){
        match ly.unwrap() {
            Layouts::Home => {
                println!("{:?}", dim);
                if let Some((w, h)) = dim {
                    frame.set_window_size(egui::vec2(*w, *h));
                }
                else {
                    frame.set_window_size(egui::vec2(400., 200.))
                }
            },
            Layouts::Path => { frame.set_window_size(egui::vec2(400., 480.)) },
            Layouts::Hotkey => { frame.set_window_size(egui::vec2(400., 500.)) },
            Layouts::About => { frame.set_window_size(egui::vec2(400., 270.)) },
            _ => {}
        }
    }
}
