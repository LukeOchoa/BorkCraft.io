// my crate imports
use crate::{errors::client_errors::*, login::login_page::*};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;

// standard library
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread, time,
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

#[derive(Default)]
pub struct SessionInformation {
    key: String,
    session_time: SessionTime,
    is_logged_in: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SessionTime {
    pub key: String,
    pub time: TimeTime,
    pub message: String,
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
    pub session_time: Arc<Mutex<SessionInformation>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        let session_timex = Arc::new(Mutex::new(SessionInformation {
            key: String::default(),
            session_time: SessionTime::default(),
            is_logged_in: bool::default(),
        }));
        current_session_time(Arc::clone(&session_timex));
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: ErrorMessage::default(),
            session_time: session_timex,
        }
    }
}

fn current_session_time(session_information: Arc<Mutex<SessionInformation>>) {
    thread::spawn(move || loop {
        let result = ureq::post("http://localhost:8123/sessiontimeleft")
            .send_json(&session_information.lock().unwrap().key);
        match result {
            Ok(response) => {
                if response.status() == 202 {
                    let a_time: TimeTime = response.into_json().unwrap();
                    let one = "1".to_string();
                    if a_time.hour < one && a_time.minute < one && a_time.second < one {
                        session_information.lock().unwrap().is_logged_in = false
                    }
                    session_information.lock().unwrap().session_time.time = a_time;
                }
            }
            Err(_) => {}
        }
        thread::sleep(time::Duration::from_secs(3))
    });
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            handle_errors(&mut self.error_message, ctx, ui);
            login(self, ui);
        });
    }
}
