use egui::*;
use egui::epaint::RectShape;

#[derive(Clone)]
pub struct MyCut {
    cut_rect: Rect,
    scaled_rect: Rect,
    offsetxl: f32,
    //x left
    offsetxr: f32,
    //x right
    offsetyu: f32,
    //y up
    offsetyd: f32,
    // y down
    last_click: Pos2,
    side: SquareSide,
    limit_reached: bool,
}

#[derive(Clone, Copy)]
enum SquareSide {
    None,
    Up,
    Down,
    Right,
    Left,
    Center,
}


impl Default for MyCut {
    fn default() -> Self {
        Self {
            cut_rect: Rect::NOTHING,
            scaled_rect: Rect::NOTHING,
            offsetxl: 0.0,
            offsetxr: 0.0,
            offsetyu: 0.0,
            offsetyd: 0.0,
            last_click: Pos2::default(),
            side: SquareSide::None,
            limit_reached: false,
        }
    }
}

//todo
//- aggiungere bottoni per conferma e chiudere cut
//- ridimensionamento del rettangolo FATTO
//- cut immagine
//- aggiustare forme per tenerle proporzionate in base alla finestra e non farle cambiare come nella paint FATTO
//- aggiustare storage offset per salvare la proporzione e non il valore FATTO
//- fare in modo che il cut rect si possa muovere nello spazio FATTO
//- pulire il codice e le strutture dati non usate
//- mergiare aurora FATTO

/**SPIEGONI:
- I controlli negli if per poter calcolare gli input prevedono il controllo di cut_rect che è sempre aggiornato rispetto all'offset
 nella versione precedente si aggiungeva/sottraeva l'offset nel controllo, ma è concettualmente sbagliato in quanto l'offset è la misurazione di quanto mi sto spostando dal bordo
 **/

impl MyCut {
    pub fn select_cut_rectangle(&mut self, ui: &mut Ui, response: Response, image_dim: Vec2) {
        let cutdim = image_dim.clone();
        //println!("{:?}", cutdim);

        self.cut_rect.min.x = response.rect.min.x + (response.rect.width() * self.offsetxl);
        self.cut_rect.min.y = response.rect.min.y + (response.rect.height() * self.offsetyu);
        self.cut_rect.max.x = response.rect.max.x - (response.rect.width() * self.offsetxr);
        self.cut_rect.max.y = response.rect.max.y - (response.rect.height() * self.offsetyd);

        //println!("cut rect cutlib {:?}, size {:?}", self.cut_rect, self.cut_rect.size());

        if self.cut_rect.is_positive() {
            ui.painter().add(Shape::Rect(RectShape {
                rounding: Rounding::none(),
                fill: Color32::from_rgba_unmultiplied(255, 255, 255, 2),
                stroke: Stroke::new(3.0, Color32::WHITE),
                rect: self.cut_rect,
            }));

            let xr = response.rect.min.x + (response.rect.width() * self.offsetxl);
            let yr = response.rect.min.y + (response.rect.height() * self.offsetyu);

            let xthird = self.cut_rect.size().x / 3.0;
            let ythird = self.cut_rect.size().y / 3.0;

            //punti a 1/3w e 2/3w
            let point1thirdw = Pos2::new(xr + xthird, yr);
            let point2thirdw = Pos2::new(xr + 2. * xthird, yr);

            //punti a 1/3h e 2/3h
            let point1thirdh = Pos2::new(xr, yr + ythird);
            let point2thirdh = Pos2::new(xr, yr + 2. * ythird);

            //punti a 1/3wh e 2/3wh
            let point1thirdwh = Pos2::new(xr + xthird, yr + self.cut_rect.size().y);
            let point2thirdwh = Pos2::new(xr + 2. * xthird, yr + self.cut_rect.size().y);

            //punti a 1/3hw e 2/3 hw
            let point1thirdhw = Pos2::new(xr + self.cut_rect.size().x, yr + ythird);
            let point2thirdhw = Pos2::new(xr + self.cut_rect.size().x, yr + 2. * ythird);

            //lines
            let linea1 = Shape::dashed_line(&[point1thirdw, point1thirdwh], Stroke::new(1.0, Color32::WHITE), 2.5, 5.);
            let linea2 = Shape::dashed_line(&[point2thirdw, point2thirdwh], Stroke::new(1.0, Color32::WHITE), 2.5, 5.);
            let linea3 = Shape::dashed_line(&[point1thirdh, point1thirdhw], Stroke::new(1.0, Color32::WHITE), 2.5, 5.);
            let linea4 = Shape::dashed_line(&[point2thirdh, point2thirdhw], Stroke::new(1.0, Color32::WHITE), 2.5, 5.);

            //linea di mezzo
            //let halfline = Shape::dashed_line(&[Pos2::new(10.0, 31.0 + image_dim.y / 2.0), Pos2::new(10.0 + image_dim.x, 31.0 + image_dim.y / 2.0)], Stroke::new(1.0, Color32::RED), 2.5, 5.);

            ui.painter().add(linea1);
            ui.painter().add(linea2);
            ui.painter().add(linea3);
            ui.painter().add(linea4);
            //ui.painter().add(halfline);
        }
        //cambio mouse quando passo sopra il rettangolo esterno
        let ctx = ui.ctx();

        //definizione limiti area di hovering
        let bound = 10.;
        let inner_rect = self.cut_rect.shrink(bound);

        if let Some(p_hover) = ctx.pointer_hover_pos() {

            //sopra e sotto
            if (p_hover.y <= self.cut_rect.min.y + bound && p_hover.y >= self.cut_rect.min.y - bound) || (p_hover.y <= self.cut_rect.max.y + bound && p_hover.y >= self.cut_rect.max.y - bound) {
                ctx.set_cursor_icon(CursorIcon::ResizeVertical);
            }
            //sinistra
            else if (p_hover.x <= self.cut_rect.min.x + bound && p_hover.x >= self.cut_rect.min.x - bound) || (p_hover.x <= self.cut_rect.max.x + bound && p_hover.x >= self.cut_rect.max.x - bound) {
                ctx.set_cursor_icon(CursorIcon::ResizeEast);
            } else if inner_rect.contains(p_hover) {
                ctx.set_cursor_icon(CursorIcon::AllScroll);
            }
        }

        if let Some(p_interact) = response.interact_pointer_pos() {
            if (p_interact.x <= (image_dim.x + response.rect.min.x) && p_interact.x >= response.rect.min.x) && (p_interact.y <= (image_dim.y + response.rect.min.y) && p_interact.y >= response.rect.min.y) {
                match self.side {
                    SquareSide::None => {
                        if (p_interact.y - (response.rect.min.y + (response.rect.height() * self.offsetyu))).abs() <= bound {
                            //println!("difference {:?} response {:?}", (p_interact.y - response.rect.min.y + self.offsetyu), response.rect.min);
                            ctx.set_cursor_icon(CursorIcon::ResizeVertical);
                            self.side = SquareSide::Up;
                        } else if (p_interact.y - (response.rect.max.y - (response.rect.height() * self.offsetyd))).abs() <= bound {
                            ctx.set_cursor_icon(CursorIcon::ResizeVertical);
                            self.side = SquareSide::Down;
                        } else if (p_interact.x - (response.rect.max.x - (response.rect.width() * self.offsetxr))).abs() <= bound {
                            ctx.set_cursor_icon(CursorIcon::ResizeWest);
                            self.side = SquareSide::Right;
                        } else if (p_interact.x - (response.rect.min.x + (response.rect.width() * self.offsetxl))).abs() <= bound {
                            ctx.set_cursor_icon(CursorIcon::ResizeEast);
                            self.side = SquareSide::Left;
                        } else if inner_rect.contains(p_interact) {
                            ctx.set_cursor_icon(CursorIcon::AllScroll);
                            self.side = SquareSide::Center;
                        }
                    }
                    SquareSide::Up => {
                        if p_interact != self.last_click {
                            let tmp = compute_offset(response.rect.min, p_interact).y / response.rect.height();
                            if (self.cut_rect.max.y - self.cut_rect.min.y) >= 50. || (self.limit_reached == true && tmp < self.offsetyu) {
                                self.offsetyu = tmp;
                                self.limit_reached = false;
                            } else {
                                self.limit_reached = true;
                            }
                            self.last_click = p_interact;
                        }
                    }
                    SquareSide::Down => {
                        if p_interact != self.last_click {
                            let tmp = compute_offset(response.rect.max, p_interact).y / response.rect.height();

                            if (self.cut_rect.max.y - self.cut_rect.min.y) >= 50. || (self.limit_reached == true && tmp < self.offsetyd) {
                                self.offsetyd = tmp;
                                self.limit_reached = false;
                            } else {
                                self.limit_reached = true;
                            }
                            self.last_click = p_interact;
                        }
                    }
                    SquareSide::Right => {
                        if p_interact != self.last_click {
                            //println!("click destra");
                            let tmp = compute_offset(response.rect.max, p_interact).x / response.rect.width();
                            if (self.cut_rect.max.x - self.cut_rect.min.x) >= 50. || (self.limit_reached == true && tmp < self.offsetxr) {
                                self.offsetxr = tmp;
                                self.limit_reached = false;
                            } else {
                                self.limit_reached = true;
                            }
                            self.last_click = p_interact;
                        }
                    }
                    SquareSide::Left => {
                        if p_interact != self.last_click {
                            let tmp = compute_offset(response.rect.min, p_interact).x / response.rect.width();
                            if (self.cut_rect.max.x - self.cut_rect.min.x) >= 50. || (self.limit_reached == true && tmp < self.offsetxl) {
                                self.offsetxl = tmp;
                                self.limit_reached = false;
                            } else {
                                self.limit_reached = true;
                            }
                            self.last_click = p_interact;
                        }
                    }
                    SquareSide::Center => {
                        if p_interact != self.last_click {
                            if p_interact.x < self.last_click.x {
                                let tmp = compute_offset(self.last_click, p_interact).x / response.rect.width();

                                if (response.rect.max.x - (response.rect.width() * (self.offsetxr + tmp))) <= (image_dim.x + response.rect.min.x) && (response.rect.min.x + (response.rect.width() * (self.offsetxl - tmp))) >= response.rect.min.x {
                                    self.offsetxl -= tmp;
                                    self.offsetxr += tmp;
                                }
                            }

                            if p_interact.x > self.last_click.x {
                                let tmp = compute_offset(self.last_click, p_interact).x / response.rect.width();

                                if (response.rect.max.x - (response.rect.width() * (self.offsetxr - tmp))) <= (image_dim.x + response.rect.min.x) && (response.rect.min.x + (response.rect.width() * (self.offsetxl + tmp))) >= response.rect.min.x {
                                    self.offsetxl += tmp;
                                    self.offsetxr -= tmp;
                                }
                            }

                            if p_interact.y < self.last_click.y {
                                let tmp = compute_offset(self.last_click, p_interact).y / response.rect.height();

                                if (response.rect.max.y - (response.rect.height() * (self.offsetyd + tmp))) <= (image_dim.y + response.rect.min.y) && (response.rect.min.y + (response.rect.height() * (self.offsetyu - tmp))) >= response.rect.min.y {
                                    self.offsetyu -= tmp;
                                    self.offsetyd += tmp;
                                }
                            }

                            if p_interact.y > self.last_click.y {
                                let tmp = compute_offset(self.last_click, p_interact).y / response.rect.height();

                                if (response.rect.max.y - (response.rect.height() * (self.offsetyd - tmp))) <= (image_dim.y + response.rect.min.y) && (response.rect.min.y + (response.rect.height() * (self.offsetyu + tmp))) >= response.rect.min.y {
                                    self.offsetyu += tmp;
                                    self.offsetyd -= tmp;
                                }
                            }

                            self.last_click = p_interact;
                        }
                    }
                }
            }
            self.last_click = p_interact;
        } else {
            self.side = SquareSide::None;
        }
        //println!("response rect {:?} size {:?}", response.rect, response.rect.size());
    }

    //todo funzione che taglia l'immagine in funzione di cut_rect
    //nella funzione prendo una clone della self.prova (myapp) e ritorno Option<Rgba>
    pub fn get_cut_rect(&mut self, originalsize: Vec2) -> Rect {
        let initialpoint = Pos2::new(originalsize.x * self.offsetxl, originalsize.y * self.offsetyu);

        let mut scaledsize = Vec2::ZERO;

        scaledsize.y = originalsize.y - originalsize.y * (self.offsetyd + self.offsetyu);
        scaledsize.x = originalsize.x - originalsize.x * (self.offsetxl + self.offsetxr);


        self.scaled_rect = Rect::from_min_size( initialpoint, scaledsize);
        self.scaled_rect
    }

    pub fn get_rect(&mut self) -> Rect {
        let mut rect = self.cut_rect;

        rect.min.x -= 10.0;
        rect.min.y -= 31.0;

        return rect
    }
}

fn compute_offset(standard: Pos2, current: Pos2) -> Pos2 { //standard: dove si trovano i bordi, current: dove e' stato cliccato
    Pos2::new((standard.x - current.x).abs(), (standard.y - current.y).abs())
}


//big = current | small = previous
/*
fn check_ratio_image(small: Vec2, big: Vec2) -> Vec2 {
    if small.x > 0. && small.y > 0. {
        return Vec2::new(big.x / small.x, big.y / small.y);
    }
    Vec2::new(1.0, 1.0)
}
 */