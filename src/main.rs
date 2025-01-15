#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Duration;
use eframe::egui;
use egui::{Visuals, Color32};
use global_hotkey::{ GlobalHotKeyEvent };
use tokio::runtime::Runtime;

mod myapp;
use myapp::MyApp;
use crate::myapp::Layouts;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    //set up tokio runtime
    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    //set up options for application
    let options = eframe::NativeOptions {
        transparent: true,
        initial_window_size: Some(egui::vec2(400., 200.)),
        ..Default::default()
    };

    eframe::run_native(
        "Screenshot",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());
        ctx.request_repaint();

        //global hotkey event receiver
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            //println!("{:?}", event);
            self.hotkey_ly.match_event(event, _frame, &mut self.layout, &mut self.disabled_time,
                                       &self.dim, self.texture.is_some(), ctx,
                                       &mut self.saving, &mut self.config, &mut self.save_by_hk);
        }

        //timer event receiver
        if let Ok(duration) = self.rx.try_recv() {
            println!("Time elapsed in expensive_function() is: {:?}", duration);
            self.saving = false;
        }

        //matcher for layout navigation
        match self.layout {
            Layouts::Home => {
                self.home_layout(ctx,_frame);
            },
            Layouts::Screenshot => {
                self.screen_layout(ctx, _frame);
            },
            Layouts::Hotkey => {
                self.hotkey_layout(ctx, _frame);
            },
            Layouts::Path => {
                self.path_layout(ctx, _frame);
            },
            Layouts::About => {
                self.about_layout(ctx, _frame);
            },
        }
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        //Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()
        Color32::from_rgba_unmultiplied(0, 0, 0, 128).to_normalized_gamma_f32()
    }
}
