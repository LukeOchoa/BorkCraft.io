use crate::{
    borkcraft_app::{BorkCraft, ErrorMessage},
    ureq_did_request_go_through_f, ResponseResult,
};
use std::{
    collections::HashMap,
    sync::{Arc, Once},
    thread,
};

use eframe::egui;
use serde_json::Value;
static START: Once = Once::new();

// abstractions for did_request_go_through

// 1) match the result from the http request

//// 1.1) match-ARM Ok(response) => On ok, match the response.status()
//// 1.1.1) match-ARM on specific [list of http codes](s) => execute custom logic
//// on catchall(_ =>) => set the error with ErrorMessage
//// 1.2) match-ARM Err(error) => set the error message
///

//error_job: &dyn Fn(String, String) -> Result<String, ErrorMessage>,
//fn ureq_did_request_go_through_f(
//    did_request_go_through: Result<ureq::Response, ureq::Error>,
//    job: Box<dyn Fn(ureq::Response) -> Result<ResponseResult, String>>,
//) -> Result<ResponseResult, ErrorMessage> {
//    let failed_to_convert_response =
//        "Failed to convert response to a variant of type Enum ResponseResult...: error ==";
//    let bad_server_response_error = "network was sent but denied by the server... Maybe wrong key was given? [retrieving nether portal keys]: error ==";
//    let no_connection_error =
//        "network request was not able to connect... [retrieving nether portal keys2]: error ==";
//
//    match did_request_go_through {
//        Ok(response) => match response.status() {
//            200..=299 => match job(response) {
//                Ok(result) => return Ok(result),
//                Err(error) => {
//                    return Err(ErrorMessage::pure_error_message(Some(format!(
//                        "{}{}",
//                        failed_to_convert_response, error
//                    ))))
//                }
//            },
//            _ => {
//                let error_string =
//                    format!("{}{}", bad_server_response_error, response.status_text());
//                return Err(ErrorMessage::pure_error_message(Some(error_string)));
//            }
//        },
//        Err(error) => {
//            let error_string = format!("{}{}", no_connection_error, error.to_string());
//            return Err(ErrorMessage::pure_error_message(Some(error_string)));
//        }
//    }
//}
fn match_out_nether_portal_keys_to_string2(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
) -> Result<String, ErrorMessage> {
    let result = ureq_did_request_go_through_f(
        did_request_go_through,
        Box::new(
            |response: ureq::Response| -> Result<ResponseResult, String> {
                match response.into_string() {
                    Ok(string) => return Ok(ResponseResult::Text(string)),
                    Err(error) => return Err(error.to_string()),
                }
            },
        ),
    );
    match result {
        Ok(response_result) => {
            //let mut some_string = String::new();
            if let ResponseResult::Text(string) = response_result {
                //some_string = string;
                return Ok(string);
            } else {
                panic!("Magical faires occured at line 82 in nether_portals.rs");
            }
            //return some_string;
        }
        Err(error) => return Err(error),
    }
}
// fn match_out_nether_portal_keys_to_string(
//     did_request_go_through: Result<ureq::Response, ureq::Error>,
// ) -> Result<String, ErrorMessage> {
//     match did_request_go_through {
//         Ok(response) => match response.status() {
//             200 => {
//                 let string_result = response.into_string();
//                 match string_result {
//                     Ok(string) => return Ok(string),
//                     Err(error) => {
//                         let error_string = Some(format!(
//                             "failed to turn nether portal keys into a vec8: error == {}",
//                             error.to_string()
//                         ));
//                         return Err(ErrorMessage::pure_error_message(error_string));
//                     }
//                 }
//             }
//             _ => {
//                 let error_string = Some(format!("network was sent but denied by the server... Maybe wrong key was given? [retrieving nether portal keys]: error == {}", response.status_text()));
//                 return Err(ErrorMessage::pure_error_message(error_string));
//             }
//         },
//         Err(error) => {
//             let error_string = Some(format!("network request was not able to connect... [retrieving nether portal keys]: error == {}", error.to_string()));
//             return Err(ErrorMessage::pure_error_message(error_string));
//         }
//     }
// }

fn json_string_to_map(map_as_string: String) -> Result<HashMap<i32, String>, ErrorMessage> {
    let value_enum = serde_json::from_str::<serde_json::Value>(&map_as_string);
    match value_enum {
        Ok(value) => match value {
            Value::Object(map) => {
                let mut hashy: HashMap<i32, String> = HashMap::default();
                for (key, value) in map {
                    hashy.insert(
                        key.parse::<i32>().unwrap(),
                        value.as_str().unwrap().to_string(),
                    );
                }
                return Ok(hashy);
            }
            _ => {
                let error_string = Some(format!(
                    "Somehow the value in Enum::Value was not a hashmap but... Magical Faries...! i hve no idea what this means so lets just hope this error doesnt happen lol..."));
                return Err(ErrorMessage::pure_error_message(error_string));
            }
        },
        Err(error) => {
            let error_string = Some(format!(
                "Failed to convert map_as_string to a json value: error == {}",
                error.to_string()
            ));
            return Err(ErrorMessage::pure_error_message(error_string));
        }
    }
}

pub fn nether_portal(borkcraft_self: &mut BorkCraft, ui: &mut egui::Ui, const_url: &'static str) {
    START.call_once(|| {
        let nether_portals_am_clone = Arc::clone(&borkcraft_self.nether_portals);
        let error_message_am_clone = Arc::clone(&borkcraft_self.error_message);
        thread::spawn(move || {
            let did_request_go_through = ureq::get(const_url).call();
            let result = match_out_nether_portal_keys_to_string2(did_request_go_through);
            match result {
                Ok(string) => {
                    let hashmap_or_error_message = json_string_to_map(string);
                    match hashmap_or_error_message {
                        Ok(hashmap) => {
                            nether_portals_am_clone.lock().unwrap().push(hashmap);
                        }
                        Err(error) => {
                            *error_message_am_clone.lock().unwrap() = error;
                        }
                    }
                }
                Err(error) => {
                    *error_message_am_clone.lock().unwrap() = error;
                }
            }
        });
    });
    ui.label(format!(
        "You selected: {}...!",
        borkcraft_self.selected_modal_page
    ));
    ui.label(format!(
        "some vec of hashmaps: {:?}",
        borkcraft_self.nether_portals.lock().unwrap()
    ));
}
