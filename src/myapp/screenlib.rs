use std::borrow::Cow;
use std::io::{ Cursor };
use crate::myapp::imglib::{ restore_dim };
use egui::*;
use eframe::Frame;
use screenshots::{Image, Screen};
use crate::myapp::{Layouts};
use arboard::{Clipboard, ImageData};
use image::{GenericImage, RgbaImage};

struct ScreenImage {
    screen: Screen,
    image: Image,
}

pub struct MyScreenshot {
    clicked: Option<ButtonClicked>,
    screenshot: bool,
    count: i32,
    id: Option<LayerId>,
    started_selection: bool,
    starting_point: Pos2,
    ending_point: Pos2,
    middle_point: Pos2,
    dimensions_selected: Vec2,
}
#[derive(PartialEq)]
enum ButtonClicked {
    FullScreen,
    GrabbedScreen,
}
impl Default for MyScreenshot {
    fn default() -> Self {
        Self{
            clicked: None,
            screenshot: false,
            count: 0,
            id: None,
            started_selection: false,
            starting_point: Default::default(),
            ending_point: Default::default(),
            middle_point: Default::default(),
            dimensions_selected: Default::default(),
        }
    }
}
impl MyScreenshot {
    /**schermata screen**/
    pub fn screen_layout(&mut self, ctx: &Context, _frame: &mut Frame,
                         layout: &mut Layouts, img_: &mut Option<RgbaImage>,
                         clipboard: &mut Option<Clipboard>)
    {
        let width = _frame.info().window_info.monitor_size.unwrap().x;
        let height = _frame.info().window_info.monitor_size.unwrap().y;
        _frame.set_decorations(false);
        _frame.set_window_size(vec2(width + 1., height + 1.));
        _frame.set_window_pos(pos2(0., 0.));
        ctx.set_cursor_icon(CursorIcon::Default);
        if ctx.is_pointer_over_area() {
            ctx.set_cursor_icon(CursorIcon::Crosshair);
        }

        if self.screenshot {
            ctx.set_cursor_icon(CursorIcon::Wait);
            //println!("screenshot.is_some()");
            let screen_images = Screen::all().unwrap()
                .into_iter()
                .filter(|screen| {
                    if self.clicked == Some(ButtonClicked::GrabbedScreen){
                        let rect = Rect::from_two_pos(
                            Pos2::new(screen.display_info.x as f32, screen.display_info.y as f32),
                            Pos2::new(
                                (screen.display_info.x + screen.display_info.width as i32) as f32,
                                (screen.display_info.y + screen.display_info.height as i32) as f32,
                            ),
                        );
                        rect.contains(self.starting_point)
                    } else {
                        true
                    }
                })
                .map(|screen| {
                    let image;
                    match self.clicked.as_ref() {
                        Some(ButtonClicked::FullScreen) => {
                            image = screen.capture().unwrap();
                        }
                        Some(ButtonClicked::GrabbedScreen) => {
                            image = screen.capture_area((self.starting_point.x * ctx.pixels_per_point()) as i32, (self.starting_point.y * ctx.pixels_per_point()) as i32, (self.dimensions_selected.x * ctx.pixels_per_point()) as u32, (self.dimensions_selected.y * ctx.pixels_per_point()) as u32).unwrap(); //TODO: inserire sreen grab
                        }
                        _ => image = screen.capture().unwrap()
                    }
                    ScreenImage{ screen, image }
                })
                .collect::<Vec<_>>();
            // Compute coordinates of combined image
            let x_min = screen_images.iter().map(|s| s.screen.display_info.x).min().unwrap();
            let y_min = screen_images.iter().map(|s| s.screen.display_info.y).min().unwrap();
            let x_max = screen_images
                .iter()
                .map(|s| s.screen.display_info.x + s.image.width() as i32)
                .max()
                .unwrap();
            let y_max = screen_images
                .iter()
                .map(|s| s.screen.display_info.y + s.image.height() as i32)
                .max()
                .unwrap();

            // Compute size and offset of combined image
            let offset = (x_min, y_min);
            let size = ((x_max - x_min) as u32, (y_max - y_min) as u32);
            //println!("Total screenshot size: {:?}", size);
            //println!("Offset: {:?}", offset);

            // Allocate combined image
            let mut img = RgbaImage::new(size.0, size.1);

            for screen_image in screen_images {
                let screenshot = image::io::Reader::new(Cursor::new(screen_image.image.to_png().unwrap()))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap();
                img.copy_from(
                    &screenshot,
                    (screen_image.screen.display_info.x - offset.0) as u32,
                    (screen_image.screen.display_info.y - offset.1) as u32,
                )
                    .unwrap();
            }
            let img_data =  ImageData {
                width: img.width() as usize,
                height: img.height() as usize,
                bytes: Cow::from(img.to_vec()),
            };
            if let Some(clip) = clipboard.as_mut() {
                clip.set_image(img_data.to_owned_img()).unwrap();
            }
            *img_ = Some(RgbaImage::from(img));
            self.clicked = None;
            self.screenshot = false;
            self.count = 0;
            _frame.set_decorations(true);
            _frame.set_window_size(vec2(0.3 * width + 200., 500.));
            _frame.set_window_pos(pos2(0., 0.));
            *layout = Layouts::Home;
        }
        if self.clicked.is_some() {
            self.count += 1;
            if self.count > 1 {
                self.screenshot = true;
            }
        }

        Area::new("screen")
            .show(ctx, |ui| {

                let response = ui.allocate_response(ctx.available_rect().size(), Sense::drag());
                let bound = response.rect.size();
                if response.drag_started() {
                    self.starting_point = ctx.pointer_interact_pos().unwrap();
                    //println!("starting point {:?}", self.starting_point);
                    self.started_selection = true;
                    //println!("{:?}", response);
                }

                if response.dragged() {
                    self.middle_point = ctx.pointer_interact_pos().unwrap();

                    if self.middle_point != self.starting_point && self.started_selection {

                        let selected_area = Rect::from_two_pos(self.starting_point, self.middle_point);

                        let selected = ui.painter().add(Shape::Noop);

                        /*let where_is_selected = */ui.painter().set(
                            selected,
                            epaint::RectShape {
                                rounding: Rounding::none(),
                                fill: Color32::from_rgba_unmultiplied(255, 255, 255, 2),
                                stroke: Stroke::new(2.0, Color32::WHITE),
                                rect: selected_area,
                            },
                        );
                    }
                }

                if response.drag_released() {
                    self.ending_point = ctx.pointer_interact_pos().unwrap();
                    //println!("ending point {:?}", self.ending_point);

                    self.dimensions_selected.x = self.ending_point.x - self.starting_point.x;
                    self.dimensions_selected.y = self.ending_point.y - self.starting_point.y;

                    if self.dimensions_selected.x.is_sign_negative() || self.dimensions_selected.y.is_sign_negative() {
                        self.dimensions_selected.x = self.dimensions_selected.x.abs();
                        self.dimensions_selected.y = self.dimensions_selected.y.abs();

                        let tmp = self.starting_point;
                        self.starting_point = self.ending_point;
                        self.ending_point = tmp;
                    }

                    //check if the selection is not too small
                    if self.dimensions_selected.x > 50.0 && self.dimensions_selected.y > 50.0{

                        //check if bounds are respected
                        if self.dimensions_selected.x > bound.x {
                            self.dimensions_selected.x = bound.x;
                        }

                        if self.dimensions_selected.y > bound.y {
                            self.dimensions_selected.y = bound.y;
                        }
                        //println!("dim selected from points {:?}", self.dimensions_selected);
                        self.clicked = Some(ButtonClicked::GrabbedScreen);

                    } else {
                        println!("ups!");
                        self.started_selection = false;
                        ctx.move_to_top(self.id.unwrap());
                    }
                }
            });
        if self.clicked.is_none() {
            Window::new("TAKE A SCREENSHOT")
                .title_bar(false)
                .fixed_pos(pos2((width - 30.) / 2.0, 0.0))
                .show(ctx, |ui| {
                    self.id = Some(ui.layer_id());
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.button("⛶").request_focus();
                            if ui.button("🖵").clicked() {
                                self.clicked = Some(ButtonClicked::FullScreen);
                            }
                            ui.separator();
                            if ui.button("◀").clicked() {
                                _frame.set_decorations(true);
                                restore_dim(&None, _frame, Some(Layouts::Home));
                                *layout = Layouts::Home;
                            }
                            if self.clicked.is_some() {
                                ui.set_visible(false);
                                ctx.request_repaint();
                            }
                        });
                    });
                });
        }
    }
}
