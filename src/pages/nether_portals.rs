use crate::{
    borkcraft_app::{BorkCraft, ErrorMessage},
    to_vec8,
};
use std::{
    collections::HashMap,
    fmt::format,
    sync::{Arc, Mutex, Once},
    thread,
};

use eframe::egui;
use serde_json::Value;
static START: Once = Once::new();

//pub fn retrieve_user() -> HashMap<String, Vec<HashMap<String, String>>> {
//    let hashy: HashMap<String, Vec<HashMap<String, String>>> =
//        ureq::get("http://localhost:8123/netherportals")
//            .call()
//            .unwrap()
//            .into_json()
//            .unwrap();
//
//    hashy
//}

//fn retrieve_user() {
//    let ob = ureq::get("http://localhost:8123/netherportals")
//        .call()
//        .unwrap()
//        .into_string()
//        .unwrap();
//
//    let newob = serde_json::from_str::<serde_json::Value>(&ob).unwrap();
//    //println!("\n\n\n\n newob: {}", newob["AllNetherPortals"]);
//
//    match newob {
//        //Value::Null => println!("It was null!"),
//        //Value::Bool(_boolean) => println!("It was a boolean!"),
//        //Value::Number(_number) => println!("It was a number!"),
//        //Value::String(_string) => println!("It was a String!"),
//        //Value::Array(_vec) => println!("It was a Vector!"),
//        Value::Object(map) => println!("It was a map!\n {}", map["AllNetherPortals"][0]["Nether"]),
//        _ => panic!("You have found Magical Faries...!"),
//    }
//}

//fn retrieve_keys_to_nether_portals() -> Option<HashMap<i32, String>> {
//    let result = ureq::get("http://localhost:8123/netherportals")
//        .call()
//        .unwrap()
//        .into_string()
//        .unwrap();
//    let hopefully_a_map = serde_json::from_str::<serde_json::Value>(&result).unwrap();
//    match hopefully_a_map {
//        Value::Object(map) => {
//            let mut hashy: HashMap<i32, String> = HashMap::default();
//            for (key, value) in map {
//                hashy.insert(key.parse::<i32>().unwrap(), value.to_string());
//            }
//            return Some(hashy);
//        }
//        _ => return None,
//    }
//}

//fn did_request_go_through_f() {
//
// abstractions for did_request_go_through

// 1) match the result from the http request

//// 1.1) match-ARM Ok(response) => On ok, match the response.status()
//// 1.1.1) match-ARM on specific [list of http codes](s) => execute custom logic
//// on catchall(_ =>) => set the error with ErrorMessage

//// 1.2) match-ARM Err(error) => set the error message

//}

// const_nether_portal_keys_url: &'static str,

// fn match_out_nether_portal_keys_to_map(
//     did_request_go_through: Result<ureq::Response, ureq::Error>,
// ) -> Option<HashMap<i32, String>> {
//     match did_request_go_through {
//         Value::Object(map) => {
//             let mut hashy: HashMap<i32, String> = HashMap::default();
//             for (key, value) in map {
//                 hashy.insert(
//                     key.parse::<i32>().unwrap(),
//                     value.as_str().unwrap().to_string(),
//                 );
//             }
//             return Some(hashy);
//         }
//         _ => return None,
//     }
// }

// fn did_request_go_through_f(
//     did_request_go_through: Result<ureq::Response, ureq::Error>,
// ) -> Option<Vec<u8>> {
// }

fn match_out_nether_portal_keys_to_string(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
) -> Result<String, ErrorMessage> {
    match did_request_go_through {
        Ok(response) => match response.status() {
            200 => {
                let string_result = response.into_string();
                match string_result {
                    Ok(string) => return Ok(string),
                    Err(error) => {
                        let mut error_message = ErrorMessage::default();
                        let error_string = Some(format!(
                            "failed to turn nether portal keys into a vec8: error == {}",
                            error.to_string()
                        ));
                        error_message.impure_set_error_message(error_string, true);
                        return Err(error_message);
                    }
                }
            }
            _ => {
                let mut error_message = ErrorMessage::default();
                let error_string = Some(format!("network was sent but denied by the server... Maybe wrong key was given? [retrieving nether portal keys]: error == {}", response.status_text()));
                error_message.impure_set_error_message(error_string, true);
                return Err(error_message);
            }
        },
        Err(error) => {
            let mut error_message = ErrorMessage::default();
            let error_string = Some(format!("network request was not able to connect... [retrieving nether portal keys]: error == {}", error.to_string()));
            error_message.impure_set_error_message(error_string, true);
            return Err(error_message);
        }
    }
}

// fn magic(
//     did_request_go_through: Result<ureq::Response, ureq::Error>,
// ) -> Option<HashMap<i32, String>> {
//     match did_request_go_through {
//         Value::Object(map) => {
//             let mut hashy: HashMap<i32, String> = HashMap::default();
//             for (key, value) in map {
//                 hashy.insert(
//                     key.parse::<i32>().unwrap(),
//                     value.as_str().unwrap().to_string(),
//                 );
//             }
//             return Some(hashy);
//         }
//         _ => return None,
//     }
// }

// fn http_request_for_nether_portal_keys(
//     const_url: &'static str,
// ) -> Result<ureq::Response, ureq::Error> {
//     ureq::get(const_nether_portal_keys_url).call()
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
                let mut error_message = ErrorMessage::default();
                let error_string = Some(format!(
                    "Somehow the value in Enum::Value was not a hashmap but... Magical Faries...! i hve no idea what this means so lets just hope this error doesnt happen lol..."));
                error_message.impure_set_error_message(error_string, true);
                return Err(error_message);
            }
        },
        Err(error) => {
            let mut error_message = ErrorMessage::default();
            let error_string = Some(format!(
                "Failed to convert map_as_string to a json value: error == {}",
                error.to_string()
            ));
            error_message.impure_set_error_message(error_string, true);
            return Err(error_message);
        }
    }
}

fn retrieve_keys_to_nether_portals() -> Option<HashMap<i32, String>> {
    let result = ureq::get("http://localhost:8123/somelist")
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    let hopefully_a_map = serde_json::from_str::<serde_json::Value>(&result).unwrap();
    match hopefully_a_map {
        Value::Object(map) => {
            let mut hashy: HashMap<i32, String> = HashMap::default();
            for (key, value) in map {
                hashy.insert(
                    key.parse::<i32>().unwrap(),
                    value.as_str().unwrap().to_string(),
                );
            }
            return Some(hashy);
        }
        _ => return None,
    }
}

pub fn nether_portal(borkcraft_self: &mut BorkCraft, ui: &mut egui::Ui, const_url: &'static str) {
    START.call_once(|| {
        //: Arc<Mutex<Vec<HashMap<i32, String>>>> =
        let nether_portals_am_clone = Arc::clone(&borkcraft_self.nether_portals);
        let error_message_am_clone = Arc::clone(&borkcraft_self.error_message);
        thread::spawn(move || {
            let did_request_go_through = ureq::get(const_url).call();
            let result = match_out_nether_portal_keys_to_string(did_request_go_through);
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
