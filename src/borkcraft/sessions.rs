use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread, time,
};

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

#[derive(Serialize)]
struct Key {
    key: String,
}

pub fn current_session_time(
    session_information: Arc<Mutex<SessionInformation>>,
    ctx: egui::Context,
) {
    thread::spawn(move || loop {
        let result = ureq::post("http://localhost:8123/sessiontimeleft").send_json(Key {
            key: session_information.lock().unwrap().session_time.key.clone(),
        });
        match result {
            Ok(response) => {
                if response.status() == 202 {
                    let a_time: TimeTime = response.into_json().unwrap();
                    let one = "1".to_string();
                    // if the session is expired; i.e. if all time is less than one second
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
