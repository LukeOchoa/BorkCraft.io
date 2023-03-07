use crate::borkcraft_app::WindowMessage;
use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread, time,
};

#[derive(Default)]
pub struct SessionInformation {
    pub key: String,
    pub time: TimeTime,
    pub is_logged_in: bool,
    pub access_rights: Vec<String>,
    pub window_message: WindowMessage,
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
    session_time_left_url: &'static str,
    ctx: egui::Context,
) {
    thread::spawn(move || loop {
        // "http://localhost:8123/sessiontimeleft"
        let result = ureq::post(session_time_left_url).send_json(Key {
            key: session_information.lock().unwrap().key.clone(),
        });

        let time = if let Ok(response) = result {
            if response.status() == 202 {
                let a_time: TimeTime = response.into_json().unwrap();
                let one = "1".to_string();
                // if the session is expired; i.e. if all time is less than one second
                if a_time.hour < one && a_time.minute < one && a_time.second < one {
                    session_information.lock().unwrap().is_logged_in = false
                }

                a_time
            } else {
                TimeTime::default()
            }
        } else {
            TimeTime::default()
        };

        write_session(&mut session_information.lock().unwrap(), time);
        ctx.request_repaint();
        thread::sleep(time::Duration::from_secs(3));
    });
}

fn write_session(session_information: &mut SessionInformation, time: TimeTime) {
    // Create a message to be displayed
    let message = Some(format!("session time remaining: \n{:?}", time));
    // Give session_information the TimeTime object to be potentially used other places
    session_information.time = time;
    // Assign "message" to session information
    session_information.window_message.message = message //WindowMessage::window_message(message);
}
