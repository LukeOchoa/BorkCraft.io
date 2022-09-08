// my crate imports
pub use crate::{errors::client_errors::*, login::login_page::*, sessions::*};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;

// standard library
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Once},
};

const LOGIN_FORM: &'static [&'static str] = &["username", "password"];
static START: Once = Once::new();

const LOGOUT: &'static str = "http://localhost:8123/nativelogout";
const LOGIN: &'static str = "http://localhost:8123/nativelogin2";

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
    pub session_information: Arc<Mutex<SessionInformation>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        let session_informationx = Arc::new(Mutex::new(SessionInformation::default()));
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: ErrorMessage::default(),
            session_information: session_informationx,
        }
    }
}

fn display_session_time_left(ui: &mut egui::Ui, time_left: &TimeTime) {
    egui::Grid::new(4).show(ui, |ui| {
        ui.label(format!("session time remaining: \n{:?}", time_left));
    });
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        START.call_once(|| {
            current_session_time(Arc::clone(&self.session_information), ctx.clone());
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            handle_errors(&mut self.error_message, ctx, ui);
            login(self, ui, LOGIN_FORM, LOGIN, LOGOUT);
            display_session_time_left(ui, &self.session_information.lock().unwrap().time);
        });
    }
}
