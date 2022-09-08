use crate::{
    borkcraft_app::{BorkCraft, SessionInformation, SessionTime},
    errors::client_errors::ErrorMessage,
};
use eframe::egui;
use serde::Serialize;
use serde_derive::Serialize;
use std::ops::{Index, IndexMut};
use ureq::{Error, Response};

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

pub fn did_logout_succeed(did_request_go_through: Result<Response, Error>) -> Option<ErrorMessage> {
    let mut error_message = ErrorMessage::default();

    match did_request_go_through {
        Ok(response) => match response.status() {
            202 => return None,
            _ => panic!("Something went very wrong...?"),
        },
        Err(error) => {
            let error_string = handle_response_failure(&error.to_string());
            error_message.impure_set_error_message(error_string, true);
            return Some(error_message);
        }
    }
}

pub fn login(
    the_self: &mut BorkCraft,
    ui: &mut egui::Ui,
    const_login_form: &'static [&'static str],
    const_login_url: &'static str,
    const_logout_url: &'static str,
) {
    egui::Grid::new(1).show(ui, |ui| {
        // If Client Is Logged In
        if !the_self.session_information.lock().unwrap().is_logged_in {
            for form_element in const_login_form.iter() {
                ui.label(form_element.clone());
                ui.add(egui::TextEdit::singleline(
                    &mut the_self.login_form[form_element],
                ));
                ui.end_row();
            }
            if ui.button("Login...!").clicked() {
                // submit all login info to server
                let did_request_go_through = submit_bytes_to_url(
                    to_vec8(&the_self.login_form),
                    &const_login_url.to_string(),
                );
                // Check the response #Mutations
                match did_request_go_through {
                    Ok(response) => match response.status() {
                        202 => {
                            // Write the response data to state
                            *the_self.session_information.lock().unwrap() =
                                login_response_to_session_information(response);
                            the_self.login_form.session_key =
                                the_self.session_information.lock().unwrap().key.clone();
                        }
                        _ => {
                            let error_string = handle_response_failure(response.status_text());
                            the_self
                                .error_message
                                .impure_set_error_message(error_string, true);
                        }
                    },
                    Err(error) => {
                        let error_string = err_to_string(error);
                        let error_string = handle_response_failure(&error_string);
                        the_self
                            .error_message
                            .impure_set_error_message(error_string, true);
                    }
                }
            }
        } else {
            ui.label("Your already logged in!");
            if ui.button("logout...?").clicked() {
                // Send Log-Out request
                let did_request_go_through = submit_bytes_to_url(
                    to_vec8(&the_self.login_form),
                    &const_logout_url.to_string(),
                );
                // Check if http request was sucessfull #Mutations
                let result = did_logout_succeed(did_request_go_through);
                if let Some(set_error_message) = result {
                    the_self.error_message = set_error_message;
                } else {
                    // On YES, reset (session and login form) state
                    *the_self.session_information.lock().unwrap() = SessionInformation::default();
                    the_self.login_form = LoginForm::default();
                }
            }
        }
    });
}

fn err_to_string(error: Error) -> String {
    if let Error::Status(u16boi, _) = error {
        return u16boi.to_string();
    } else {
        //panic!("Magical Faries have occured...!")
        return "Could not connect to server...?!".to_string();
    }
}

fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(cereal).unwrap()
}

fn submit_bytes_to_url(body: Vec<u8>, url: &String) -> Result<Response, Error> {
    let result = ureq::post(url).send_bytes(&body);

    result
}

fn login_response_to_session_information(response: Response) -> SessionInformation {
    let session_time: SessionTime = response.into_json().unwrap();
    SessionInformation {
        time: session_time.time,
        key: session_time.key,
        is_logged_in: true,
    }
}

// parse the error and return a user friendly error
fn handle_response_failure(status_code: &str) -> Option<String> {
    // create a comprehensive list of error messages for the user's understanding
    match status_code {
        "403" => Some(format!(
            "Your request was deemed invalid: {}",
            status_code.to_string()
        )),
        _ => Some(format!(
            "Your network request did not go through: {}",
            status_code.to_string()
        )),
    }
}

//fn dynamic_handle_response_failure(status_code: &str, ) {}
