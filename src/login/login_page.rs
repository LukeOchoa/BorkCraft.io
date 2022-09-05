use crate::borkcraft_app::{BorkCraft, SessionTime};
use eframe::egui;
use serde_derive::Serialize;
use std::ops::{Index, IndexMut};

use ureq::{Error, Response};

const LOGIN_FORM: &'static [&'static str] = &["username", "password"];

#[derive(Default, Serialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub session_key: String,
}
impl Index<&'_ str> for LoginForm {
    type Output = String;
    fn index(&self, s: &str) -> &String {
        match s {
            "password" => &self.password,
            "username" => &self.username,
            _ => panic!("unknown field: {}", s),
        }
    }
}
impl IndexMut<&'_ str> for LoginForm {
    fn index_mut(&mut self, s: &str) -> &mut String {
        match s {
            "password" => &mut self.password,
            "username" => &mut self.username,
            _ => panic!("unknown field: {}", s),
        }
    }
}
impl LoginForm {
    pub fn get(&self, index: &str) -> Result<&String, String> {
        match index {
            "password" => Ok(&self.password),
            "username" => Ok(&self.username),
            "session_key" => Ok(&self.session_key),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        }
    }
    pub fn clone(&self) -> LoginForm {
        LoginForm {
            username: self.username.clone(),
            password: self.password.clone(),
            session_key: self.session_key.clone(),
        }
    }
}

pub fn login(the_self: &mut BorkCraft, ui: &mut egui::Ui) {
    egui::Grid::new(1).show(ui, |ui| {
        for item in LOGIN_FORM.iter() {
            ui.label(item.clone());
            ui.add(egui::TextEdit::singleline(&mut the_self.login_form[item]));
            ui.end_row();
        }

        if ui.button("Login...!").clicked() {
            let did_request_go_through = submit_login_information(the_self.login_form.clone());
            match did_request_go_through {
                Ok(response) => match response.status() {
                    202 => handle_response_success(response),
                    _ => {
                        the_self.error_message.impure_set_error_message(
                            handle_response_failure(response.status_text()),
                            true,
                        );
                    }
                },
                Err(error) => {
                    the_self.error_message.impure_set_error_message(
                        handle_response_failure(&error.to_string()),
                        true,
                    );
                }
            }
        }
    });
}

// 3) spin up a thread that continually tracks how long until the session expires
// this function takes the user's login info and makes a post request. Then it returns the response
fn submit_login_information(login_form: LoginForm) -> Result<Response, Error> {
    let result = ureq::post("http://localhost:8123/nativelogin2").send_json(login_form);

    result
}

fn handle_response_success(response: Response) {
    let session_time: SessionTime = response.into_json().unwrap();
    println!("Time until Session expires: |{:?}|", session_time)
}

// parse the error and return a user friendly error
fn handle_response_failure(status_code: &str) -> Option<String> {
    // create a comprehensive list of error messages for the user's understanding
    match status_code {
        "403" => Some(format!(
            "Your request was deemed invalid, username/password failed to be correct: {}",
            status_code.to_string()
        )),
        _ => Some(format!(
            "Your network request did not go through: {}",
            status_code.to_string()
        )),
    }
}
