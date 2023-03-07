use crate::{
    borkcraft_app::{BorkCraft, SessionInformation, SessionTime},
    errors::client_errors::ErrorMessage,
    //thread_tools::ThreadPool,
    to_vec8,
    ureq_did_request_go_through_f,
    windows::client_windows::WindowMessage,
    ResponseResult,
};
use eframe::{egui, epaint::ahash::HashMap};
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

fn get_access_rights(username: String, url: String) -> Result<ureq::Response, ureq::Error> {
    //"http://localhost:8123/getaccessrights?username={}",
    let url = &format!("{}?username={}", url, username);
    let result = ureq::get(url).call();

    result
}

pub fn login(
    the_self: &mut BorkCraft,
    ui: &mut egui::Ui,
    const_login_form: &'static [&'static str],
    const_login_url: &'static str,
    const_logout_url: &'static str,
    const_access_rights_url: &'static str,
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
                // Check the Response
                let result = ureq_did_request_go_through_f(
                    did_request_go_through,
                    Box::new(|response: ureq::Response| {
                        return Ok(ResponseResult::SessionInformation(
                            login_response_to_session_information(response),
                        ));
                    }),
                );

                match result {
                    Ok(response_result) => {
                        if let ResponseResult::SessionInformation(session_information) =
                            response_result
                        {
                            the_self.login_form.session_key = session_information.key.clone();
                            *the_self.session_information.lock().unwrap() = session_information;
                        } else {
                            panic!("Magical Faires occured at line 91 in login_page.rs");
                        }
                    }
                    Err(error) => *the_self.error_message.lock().unwrap() = error,
                }

                // get access rights list from server
                let result = get_access_rights(
                    the_self.login_form.username.clone(),
                    const_access_rights_url.to_string(),
                );
                match result {
                    Ok(response) => {
                        // the list is a hashmap technically with the type of "hashmap -> array of strings"
                        // so convert it from the json response
                        let mut hasher: HashMap<String, Vec<String>> =
                            serde_json::from_str(&response.into_string().unwrap()).unwrap();

                        // then take that array of strings and remove it from the hashmap to a random object
                        let ival = hasher.remove("access_rights").unwrap();

                        // transfer ownership to the SessionInformation master data structure
                        the_self.session_information.lock().unwrap().access_rights = ival;
                    }
                    Err(error) => {
                        *the_self.error_message.lock().unwrap() =
                            ErrorMessage::pure_error_message(Some(error.to_string()))
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
                // Check if http request was sucessfull
                let result = did_logout_succeed(did_request_go_through);
                if let Some(set_error_message) = result {
                    *the_self.error_message.lock().unwrap() = set_error_message;
                } else {
                    // On YES, reset (session and login form) state
                    *the_self.session_information.lock().unwrap() = SessionInformation::default();
                    the_self.login_form = LoginForm::default();
                }
            }
        }
    });
}

pub fn did_logout_succeed(did_request_go_through: Result<Response, Error>) -> Option<ErrorMessage> {
    match did_request_go_through {
        Ok(response) => match response.status() {
            202 => return None,
            _ => panic!("Something went very wrong...?"), // prob change this to err mssg
        },
        Err(error) => {
            let error_string = handle_response_failure(&error.to_string());
            return Some(ErrorMessage::pure_error_message(error_string));
        }
    }
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
        access_rights: Vec::default(),
        window_message: WindowMessage::default(),
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
