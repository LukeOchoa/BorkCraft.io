// my crate imports
use crate::{errors::client_errors::*, login::login_page::*};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;

// standard library
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// Other
use serde_derive::{Deserialize, Serialize};

#[derive(Default)]
pub struct ImageCache {
    pub cache: HashMap<i32, Image>,
}
#[derive(Default)]
pub struct Image {
    pub name: String,
    pub image: Option<RetainedImage>,
    pub path: Option<String>, // path is a location(a file) to a user submitted image.
}

impl ImageCache {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}

fn handle_errors(an_error: &mut ErrorMessage, ctx: &egui::Context, ui: &mut egui::Ui) {
    an_error.is_window_open = an_error.display_error(ctx);
    an_error.impure_open_error_window_on_click(ui);
}
pub struct BorkCraft {
    pub image_cache: Arc<Mutex<ImageCache>>,
    pub login_form: LoginForm,
    pub error_message: ErrorMessage,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: ErrorMessage::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SessionTime {
    pub key: String,
    pub time: TimeTime,
    pub message: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TimeTime {
    pub hour: String,
    pub minute: String,
    pub second: String,
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            handle_errors(&mut self.error_message, ctx, ui);
            login(self, ui);
        });
    }
}
