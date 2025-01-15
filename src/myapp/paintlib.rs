use egui::*;
use image::{RgbaImage};
use crate::myapp::cutlib::MyCut;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    // lines: Option<(Vec<Pos2>, Stroke)> ,
    // redo_lines: Vec<(Vec<Pos2>, Stroke)>,
    stroke: Stroke,
    //image: Option<Image>,
    // shapes: Vec<Shape>,
    // redo_shapes: Vec<Shape>,
    shapes: Vec<(Shapes, Vec<Pos2>, Stroke)>,
    redo_shapes: Vec<(Shapes, Vec<Pos2>, Stroke)>,
    // shape_info: Option<(Pos2, Pos2, Stroke)>,
    shape: Shapes,
    dim: (Pos2, Pos2),
    response_rect: Rect,
    to_screen: emath::RectTransform,
    //square_proportion: Vec2,
    original_values: Vec2,
    coeff_x: f32,
    coeff_y: f32
}
#[derive(PartialEq)]
pub enum Shapes {
    Rect,
    Circle,
    None,
}

impl Shapes {
    pub fn to_name(&self) -> &'static str {
        return match self {
            Shapes::Rect => "□",
            Shapes::Circle => "⭕",
            Shapes::None => "〰"
        };
    }
    pub fn get_shape(&self, pos: Vec<Pos2>, stroke: Stroke) -> Shape {
        return match self {
            Shapes::Rect => {
                if pos.len() >= 2 {
                    //println!("Rect");
                    let rect = Rect::from_two_pos(pos[0], pos[1]);
                    Shape::rect_stroke(rect, 0., stroke)
                } else {
                    //println!("Noop rect");
                    Shape::Noop
                }
            }
            Shapes::Circle => {
                if pos.len() >= 2 {
                    let mut radius = ((pos[1].x - pos[0].x).powf(2.0) + (pos[1].y - pos[0].y).powf(2.0)).sqrt();
                    //println!("Circle");
                    if radius == 0. {
                        radius = 10.;
                    }
                    Shape::circle_stroke(pos[0], radius, stroke)
                } else {
                    //println!("Noop Circle");
                    Shape::Noop
                }
            }
            _ => { Shape::Noop }
        };
    }
    /*
    pub fn is_some(&self) -> bool {
        return match self {
            Shapes::None => false,
            _ => true
        };
    }
     */
    pub fn is_none(&self) -> bool {
        return match self {
            Shapes::None => true,
            _ => false
        };
    }
    pub fn clone(&self) -> Self {
        return match self {
            Shapes::Rect => Shapes::Rect,
            Shapes::Circle => Shapes::Circle,
            Shapes::None => Shapes::None
        };
    }
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            //image: None,
            shapes: Default::default(),
            redo_shapes: Default::default(),
            shape: Shapes::None,
            //dim: (Default::default(), 0., 0.),
            dim: (Default::default(), Default::default()),
            response_rect: Rect::NOTHING,
            to_screen: emath::RectTransform::identity(Rect::NOTHING),
            //square_proportion: Vec2::ZERO,
            original_values: Vec2::ZERO,
            coeff_x: 0.,
            coeff_y: 0.,
        }
    }
}

impl Painting {
    // pub fn get_shapes(&self) -> Vec<Shape>  {
    //     self.shapes.clone()
    // }
    pub fn set_shape(&mut self, shape: Shapes) {
        self.shape = shape;
    }
    pub fn clear(&mut self) {
        self.shapes.clear();
    }
    pub fn undo(&mut self) {
        //println!("Shape len {}", self.shapes.len());
        if self.shapes.len() > 1 {
            //println!("Inside");
            let tmp = self.shapes.pop().unwrap();
            self.redo_shapes.push(self.shapes.pop().unwrap());
            self.shapes.push(tmp);
        }
    }
    pub fn redo(&mut self) {
        if !self.redo_shapes.is_empty() {
            let tmp = self.shapes.pop().unwrap();
            self.shapes.push(self.redo_shapes.pop().unwrap());
            self.shapes.push(tmp);
        }
    }
    pub fn stroke(&mut self, ui: &mut Ui) {
        stroke_ui(ui, &mut self.stroke, "");
    }
    pub fn ui_content(&mut self, ui: &mut Ui, texture: &&TextureHandle, mycut: &mut Option<MyCut>) -> egui::Response {
        let size = ui.available_size_before_wrap() * 0.93;
        let mut image_width = texture.size_vec2().x;
        let mut image_height = texture.size_vec2().y;
        if image_width > size.x { //is fat
            //Vec2 with the width of the rect and high that depends on the ratio of the texture
            image_width = size.x;
            image_height = size.x / texture.aspect_ratio();
        }
        if image_height > size.y { //is tall
            image_height = size.y;
            image_width = size.y * texture.aspect_ratio();
        }
        let (mut response, painter) =
            ui.allocate_painter(Vec2::new(image_width, image_height), Sense::drag());

        if self.original_values == Vec2::ZERO {
            self.original_values = texture.size_vec2();
        }

        let coeff_x = self.original_values.x/texture.size_vec2().x;
        let coeff_y = self.original_values.y/texture.size_vec2().y;
        self.coeff_x = coeff_x;
        self.coeff_y = coeff_y;
        //println!("x: {:?}, y: {:?}", coeff_x, coeff_x);
        // if self.square_proportion == Vec2::ZERO {
        //     self.square_proportion = response.rect.square_proportions();
        // }


        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Rect::from_min_size(response.rect.min, vec2(response.rect.width()*coeff_x, response.rect.height()*coeff_y)).square_proportions()),
            Rect::from_min_size(response.rect.min, vec2(response.rect.width()*coeff_x, response.rect.height()*coeff_y)),
        );


        //println!("response rect {:?}", response.rect);
        //println!("------------ \nto screen {:?}", to_screen);

        let from_screen = to_screen.inverse();

        self.to_screen = to_screen.clone();


        self.response_rect = response.rect.clone();
        //println!("response_min: {:?}, response_max: {:?}", response.rect.min, response.rect.max);
        painter.add(Shape::image(
            texture.id(),
            Rect::from_min_size(response.rect.min, vec2(image_width, image_height)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
            Color32::WHITE)
        );
        self.dim = (response.rect.min, response.rect.max);

        if mycut.is_none() {
            if self.shapes.is_empty() {
                self.shapes.push((self.shape.clone(), vec![], self.stroke.clone()));
            }
            if self.shape.is_none() {
                let (current_shape, current_line, current_color) = self.shapes.last_mut().unwrap();
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let canvas_pos = from_screen * pointer_pos;
                    //println!("canvas_pos: {:?}", canvas_pos);
                    if current_line.last() != Some(&canvas_pos) {
                        *current_color = self.stroke.clone();
                        *current_shape = self.shape.clone();
                        current_line.push(canvas_pos);
                        response.mark_changed();
                    }
                } else if !current_line.is_empty() {
                    self.redo_shapes.clear();
                    self.shapes.push((self.shape.clone(), vec![], self.stroke.clone()));
                    response.mark_changed();
                }
            } else {
                let (current_shape, current_line, current_color) = self.shapes.last_mut().unwrap();
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let canvas_pos = from_screen * pointer_pos;
                    if current_line.is_empty() {
                        current_line.push(canvas_pos);
                        //println!("Empty {}", current_line.len());
                    } else if current_line.last() != Some(&canvas_pos) {
                        //println!("Second point {}", current_line.len());
                        if current_line.len() > 1 { current_line.pop(); }
                        current_line.push(canvas_pos);
                        *current_color = self.stroke.clone();
                        *current_shape = self.shape.clone();
                        response.mark_changed();
                    }
                } else if !current_line.is_empty() {
                    self.redo_shapes.clear();
                    self.shapes.push((self.shape.clone(), vec![], self.stroke.clone()));
                    response.mark_changed();
                }
            }
        }

        let shapes = self
            .shapes
            .iter()
            .filter(|(_, line, _)| line.len() >= 2)
            .map(|(shape, line, stroke)| {
                match shape {
                    Shapes::None => {
                        //println!("shapes None");
                        let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                        return Shape::line(points, *stroke);
                    }
                    Shapes::Rect => {
                        //println!("shapes Rect");
                        let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                        return shape.get_shape(points, *stroke);
                    }
                    Shapes::Circle => {
                        //println!("shapes Circle");
                        let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                        return shape.get_shape(points, *stroke);
                    }
                }
            });

        //let shapes = shapes.chain(self.shapes.clone());
        //self.image = Some(create_image_buffer(self.lines.clone(), &mut mesh.clone()));
        painter.extend(shapes);
        if mycut.is_some() {
            mycut.as_mut().unwrap().select_cut_rectangle(ui, response.clone(), Vec2::new(image_width, image_height));
        }

        response
    }
    /*
    pub fn create_rgba(&mut self, ctx: &Context, _frame: &mut eframe::Frame) -> Option<(Vec<u8>, u32, u32)> {
        ctx.set_cursor_icon(CursorIcon::None);
        let mut rgba = None;
        let screens = Screen::all().unwrap();
        for screen in screens {
            let image = screen.capture().unwrap();
            println!("Image taken");
            rgba = Some((image.rgba().clone(), image.width(), image.height()));
        }
        return rgba;
    }
     */
    pub fn edit_rgba(&mut self, mut img: RgbaImage) -> Option<(Vec<u8>, u32, u32)> {
        // let pixels = img.enumerate_pixels_mut();
        // let pixel = img.get_pixel(0, 0);


        //let to_img = self.to_screen;
        //println!("response_min: {:?}, response_max: {:?}", response.rect.min, response.rect.max);
        let width = img.width();
        let height = img.height();
        let to_img = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Rect::from_min_size(Pos2::ZERO, self.original_values).square_proportions()),
            Rect::from_min_size(Pos2::ZERO, self.original_values),
        );
        //println!("where: {:?}, {:?}", self.dim.0, self.dim.1);
        self.shapes
            .iter()
            .filter(|(_, line, _)| line.len() >= 2)
            .for_each(|(shape, line, stroke)| {
                //println!("Color: {:?}, line: {}",  stroke.color, line.len());
                match shape {
                    Shapes::None => {
                        let points: Vec<Pos2> = line.iter().map(|p| to_img * *p).collect();
                        //println!("Points: {:?}", points.len());
                        for i in 0..points.len() - 1 {
                            let start = points[i];
                            let end = points[i + 1];
                            //println!("Getting pixels {}, start: {:?}, end: {:?}", i, start, end);
                            let pixels = calc_pixels_rect(start, end, stroke.width);

                                let alpha = stroke.color[3] as f32 / 255.0; // Alpha value in the range [0, 1]
                                let color = image::Rgba(stroke.color.to_array());
                                for p in pixels {//println!("p.x {}, p.y {}, dim {}, ppp {}", p.x, p.y, img.width(), ctx.pixels_per_point());
                                    if ((p.x) as u32) < width && ((p.y) as u32) < height && p.x >= 0. && p.y >= 0. {
                                        let pixel = img.get_pixel_mut((p.x) as u32, (p.y) as u32);
                                        // For example, you can set the pixel to red

                                        *pixel = image::Rgba([
                                            (pixel[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8,
                                            (pixel[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8,
                                            (pixel[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8,
                                            255
                                        ]);
                                    }
                                }
                        }
                    }
                    Shapes::Rect => {
                        let mut points = Vec::new();
                        points.push(to_img * line[0]);
                        points.push(to_img * Pos2::new(line[0].x, line[1].y));
                        points.push(to_img * line[1]);
                        points.push(to_img * Pos2::new(line[1].x, line[0].y));
                        points.push(to_img * line[0]);
                        //points = line.iter().map(|p| { to_img * p; println!("p.x {}, p.x - x {}", p.x, p.x - x); Pos2::new(p.x - x, p.y - y) }).collect();
                        for i in 0..points.len() - 1 {
                            let start = points[i];
                            let end = points[i + 1];
                            //println!("img.width {}", width);
                            let pixels = calc_pixels_rect(start, end, stroke.width);
                            let alpha = stroke.color[3] as f32 / 255.0; // Alpha value in the range [0, 1]
                            let color = image::Rgba(stroke.color.to_array());
                            for p in pixels {
                                //println!("p.x {}, p.y {}, dim {}, ppp {}", p.x, p.y, img.width(), ctx.pixels_per_point());
                                //if p.x as u32 <= img.width() && p.y as u32 <= img.height() {
                                //println!("p.x - x {}, p.x - x {}", p.x - x, (p.x - x) as u32);
                                if ((p.x) as u32) < width && ((p.y) as u32) < height && p.x >= 0. && p.y >= 0. {
                                    let pixel = img.get_pixel_mut((p.x) as u32, (p.y) as u32);

                                    *pixel = image::Rgba([
                                        (pixel[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8,
                                        (pixel[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8,
                                        (pixel[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8,
                                        255
                                    ]);
                                }
                               //}

                            }
                        }
                    }
                    Shapes::Circle => {
                        //println!("Circle");
                        let center = to_img * line[0];
                        let point = to_img * line[1];
                        let y_c = circle_equation(center/*, point*/);
                        let rad = distance(center - point);
                        let alpha = stroke.color[3] as f32 / 255.0; // Alpha value in the range [0, 1]
                        let color = image::Rgba(stroke.color.to_array());
                        let thickness = (stroke.width / 2.) as i32;
                        //println!("x_min: {} - {}, point: {}", center.x, rad, point.x);


                        // Increase the number of iterations for better precision
                        let num_iterations = 1000 * rad as i32;
                        let step_size = rad * 2.0 / num_iterations as f32;

                        for i in 0..=num_iterations {
                            let x_c = center.x - rad + (i as f32) * step_size;
                            let my_y = y_c(x_c as f32, rad);
                            let my_y_2 = center.y - (my_y - center.y);
                            //println!("my_y: {}, x: {}", my_y, x_c);

                            if !my_y.is_nan() {
                                for dx in (-thickness)..=thickness {
                                    for dy in (-thickness)..=thickness {
                                        if ((x_c as i32 + dx) as u32) < width && ((my_y as i32 + dy) as u32) < height && (x_c as i32 + dx) >= 0  && (my_y as i32 + dy) >= 0 {
                                            let pixel = img.get_pixel_mut((x_c + dx as f32) as u32, (my_y + dy as f32) as u32);

                                            *pixel = image::Rgba([
                                                (pixel[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8,
                                                (pixel[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8,
                                                (pixel[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8,
                                                255
                                            ]);
                                        }
                                    }
                                }
                            }

                            if !my_y_2.is_nan() {
                                for dx in (-thickness)..=thickness {
                                    for dy in (-thickness)..=thickness {
                                        if ((x_c as i32 + dx) as u32) < width && ((my_y_2 as i32 + dy) as u32) < height && ((x_c as i32 + dx)) >= 0 && (my_y as i32 + dy) >= 0 {
                                            let pixel = img.get_pixel_mut((x_c + dx as f32) as u32, (my_y_2 + dy as f32) as u32);

                                            *pixel = image::Rgba([
                                                (pixel[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8,
                                                (pixel[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8,
                                                (pixel[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8,
                                                255
                                            ]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        return Some((img.to_vec(), img.width(), img.height()));
    }

    pub fn adapt_to_cut(&mut self, cutrect: Rect) {

        for (_, line, _) in &mut self.shapes {
            let points: Vec<Pos2> = line.iter_mut().map(|p| {

                //println!("rimappo");
                let mut point = self.to_screen * *p;

                //println!("before {:?}", point);
                point.x -= cutrect.min.x;
                point.y -= cutrect.min.y;
                //println!("after {:?}", point);

                return self.to_screen.inverse() * point;
            }).collect();

            //println!("--------------------------------");

            *line = points;
        }
    }
}



pub fn calc_pixels(start: Pos2, end: Pos2, thickness: f32) -> Vec<Pos2> {
    let mut pixels = Vec::new();

    // // Calculate the direction vector of the line segment
    // let direction = end - start;
    //
    // // Calculate the length of the direction vector
    // let length = direction.length();
    //
    // // Normalize the direction vector
    // let normalized_direction = direction / length;
    //
    // // Calculate the perpendicular vector to the direction vector
    // let perpendicular = egui::Pos2::new(-normalized_direction.y, normalized_direction.x);
    //
    // // Calculate the offset from the line based on thickness
    // let offset = vec2(perpendicular.x * (thickness / 2.0), perpendicular.y * (thickness / 2.0));

    let delta_x = (end.x - start.x).abs();
    let delta_y = (end.y - start.y).abs();

    let step_x = if start.x < end.x { 1.0 } else { -1.0 };
    let step_y = if start.y < end.y { 1.0 } else { -1.0 };

    let mut x = start.x;
    let mut y = start.y;

    let mut error = delta_x - delta_y;
    //println!("start {:?}, end {:?}", start, end);
    pixels.push(Pos2::new(x, y));

    while (end.x - x).abs() > 0.7 || (end.y - y).abs() > 0.7 {
        //println!("point: {} {}", x, y);
        let double_error = error * 2.0;

        if double_error > -delta_y {
            error -= delta_y;
            x += step_x;
        }

        if double_error < delta_x {
            error += delta_x;
            y += step_y;
        }
        // let num_pixels = (length / thickness).ceil() as usize;
        // for i in 0..num_pixels {
        //     let t = i as f32 / (num_pixels - 1) as f32; // Spread the pixels along the line
        //     let pixel_pos = start + normalized_direction * (length * t) - offset;
        //     pixels.push(pixel_pos);
        // }
        let num_iterations = (thickness * 2.) + 1.;
        let step_size = thickness / num_iterations;

        //println!("{}", step_size);

        for dx in 0..num_iterations as i32 {
            for dy in 0..num_iterations as i32 {
                let offset_x = step_size * dx as f32;
                let offset_y = step_size * dy as f32;
                let mut new_x = x + offset_x as f32;
                let mut new_y = y + offset_y as f32;
                //println!("{} {}", new_x, new_y);
                pixels.push(Pos2::new(new_x, new_y));
                new_x = x - offset_x as f32;
                new_y = y - offset_y as f32;
                pixels.push(Pos2::new(new_x, new_y));
            }
        }

        pixels.push(Pos2::new(x, y));
    }

    pixels
}
pub fn calc_pixels_rect(start: Pos2, end: Pos2, thickness: f32) -> Vec<Pos2>  {
    let mut pixels = Vec::new();
    let delta_x = (end.x - start.x).abs();
    let delta_y = (end.y - start.y).abs();

    let step_x = if start.x < end.x { 1.0 } else { -1.0 };
    let step_y = if start.y < end.y { 1.0 } else { -1.0 };

    let mut x = start.x;
    let mut y = start.y;

    let mut error = delta_x - delta_y;
    //println!("start {:?}, end {:?}", start, end);
    pixels.push(Pos2::new(x, y));

    while (end.x - x).abs() > 0.7 || (end.y - y).abs() > 0.7 {
        //println!("point: {} {}", x, y);
        let double_error = error * 2.0;

        if double_error > -delta_y {
            error -= delta_y;
            x += step_x;
        }

        if double_error < delta_x {
            error += delta_x;
            y += step_y;
        }
        // let num_pixels = (length / thickness).ceil() as usize;
        // for i in 0..num_pixels {
        //     let t = i as f32 / (num_pixels - 1) as f32; // Spread the pixels along the line
        //     let pixel_pos = start + normalized_direction * (length * t) - offset;
        //     pixels.push(pixel_pos);
        // }
        let num_iterations = (thickness * 2.) + 1.;
        let step_size = thickness / num_iterations;

        //println!("{}", step_size);

        let thickness_ = thickness / 2.;
        for dx in -thickness_ as i32..=thickness_ as i32 {
            for dy in -thickness_ as i32..=thickness_ as i32 {
                let new_x = x + dx as f32;
                let new_y = y + dy as f32;
                pixels.push(Pos2::new(new_x, new_y));
            }
        }

        pixels.push(Pos2::new(x, y));
    }

    pixels
}
/*
fn compute_x_on_rect(p1: Pos2, p2: Pos2, x_r: f32) -> f32 {
    ((x_r - p1.x) * ((p2.y - p1.y) / (p2.x - p1.x))) + p1.y
}

fn compute_y_on_rect(p1: Pos2, p2: Pos2, y_r: f32) -> f32 {
    ((y_r - p1.y) * ((p2.x - p1.x) / (p2.y - p2.x))) + p1.x
}
 */


fn distance(p1: Vec2) -> f32 {
    ((p1.x * p1.x) + (p1.y * p1.y)).sqrt() as f32
}

fn circle_equation(center: Pos2/*, point_on_circle: Pos2*/) -> impl Fn(f32, f32) -> f32 {
    //let radius = distance(point_on_circle - center);

    move |x, rad| (center.y + ((rad * rad - (x - center.x) * (x - center.x)).sqrt()))
}
/*
fn perpendicular_line_equation(start: Pos2, end: Pos2) -> impl Fn(f32) -> f32 {
    let m = -(end.x - start.x)/(end.y - start.y);
    let b = start.y - m * start.x;
    move |x| m * x + b
}
 */