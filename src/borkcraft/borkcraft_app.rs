// my crate imports
use crate::{errors::client_errors::*, login::login_page::*};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;

// standard library
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Once},
    thread, time,
};

// Other
use serde_derive::{Deserialize, Serialize};

static START: Once = Once::new();

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

#[derive(Default)]
pub struct SessionInformation {
    pub key: String,
    pub session_time: SessionTime,
    pub is_logged_in: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SessionTime {
    pub key: String,
    pub time: TimeTime,
}
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TimeTime {
    pub hour: String,
    pub minute: String,
    pub second: String,
}

impl SessionInformation {
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

#[derive(Serialize)]
struct Key {
    key: String,
}
fn current_session_time(session_information: Arc<Mutex<SessionInformation>>, ctx: egui::Context) {
    thread::spawn(move || loop {
        let result = ureq::post("http://localhost:8123/sessiontimeleft").send_json(Key {
            key: session_information.lock().unwrap().session_time.key.clone(),
        });
        match result {
            Ok(response) => {
                if response.status() == 202 {
                    let a_time: TimeTime = response.into_json().unwrap();
                    let one = "1".to_string();
                    if a_time.hour < one && a_time.minute < one && a_time.second < one {
                        session_information.lock().unwrap().is_logged_in = false
                    }
                    session_information.lock().unwrap().session_time.time = a_time;
                    ctx.request_repaint();
                }
            }
            Err(_) => {}
        }
        thread::sleep(time::Duration::from_secs(3));
    });
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
            login(self, ui);
            display_session_time_left(
                ui,
                &self.session_information.lock().unwrap().session_time.time,
            );
        });
    }
}
