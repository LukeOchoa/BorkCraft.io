use crate::{
    borkcraft_app::{modal_machine, BorkCraft, ErrorMessage},
    ureq_did_request_go_through_f, ResponseResult,
};
use eframe::egui::{self, Key, Response};
use egui_extras::{Size, TableBuilder};
use poll_promise::Promise;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Once},
    thread,
};
use ureq::Header;
static START: Once = Once::new();
const LIST1: &'static [&'static str] = &["1", "2", "3", "4", "5"];

pub struct NetherPortalModal {
    pub modal: String,
    pub modal_list: Arc<Mutex<Option<Vec<String>>>>,
}

#[derive(Debug, Default)]
pub struct NetherPortalInformation {
    whitelist: MemberIds,
    nether_portals: HashMap<String, NetherPortal>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NetherPortal {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "OverWorld")]
    overworld: Portal,
    #[serde(rename = "Nether")]
    nether: Portal,
    #[serde(rename = "Username")]
    username: String,
}
#[derive(Debug, Deserialize, Default)]
pub struct Portal {
    #[serde(rename = "Xcord")]
    xcord: i32,
    #[serde(rename = "Ycord")]
    ycord: i32,
    #[serde(rename = "Zcord")]
    zcord: i32,
    #[serde(rename = "Locale")]
    locale: String,
    #[serde(rename = "Owner")]
    owner: String,
    #[serde(rename = "Notes")]
    notes: String,
}
impl NetherPortalInformation {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}
pub enum PortalValue<'a> {
    Text(&'a String),
    Number(&'a i32),
}
//impl PortalValue {
//    pub fn resolve(&self) -> &
//}
impl Portal {
    pub fn get(&self, index: &str) -> Result<PortalValue, String> {
        match index {
            "xcord" => Ok(PortalValue::Number(&self.xcord)),
            "ycord" => Ok(PortalValue::Number(&self.zcord)),
            "zcord" => Ok(PortalValue::Number(&self.ycord)),
            "locale" => Ok(PortalValue::Text(&self.locale)),
            "owner" => Ok(PortalValue::Text(&self.owner)),
            "notes" => Ok(PortalValue::Text(&self.notes)),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct MemberIds {
    #[serde(rename = "Member_Ids")]
    member_ids: Vec<MemberId>,
}
#[derive(Debug, Deserialize, Default)]
pub struct MemberId {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
}
impl MemberId {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}
impl MemberIds {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}

fn match_out_nether_portal_keys_to_result_string(
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

fn json_string_to_member_ids(json_string: String) -> Result<MemberIds, ErrorMessage> {
    match serde_json::from_str::<MemberIds>(&json_string) {
        Ok(type_member_ids) => return Ok(type_member_ids),
        Err(error) => Err(ErrorMessage::pure_error_message(Some(error.to_string()))),
    }
}

fn request_to_nether_portal(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
) -> Result<ResponseResult, ErrorMessage> {
    let result = ureq_did_request_go_through_f(
        did_request_go_through,
        Box::new(|response: ureq::Response| {
            let json_string = response.into_string();
            match json_string {
                Ok(string) => match serde_json::from_str(&string) {
                    Ok(some_nether_portal) => {
                        return Ok(ResponseResult::NetherPortal(some_nether_portal));
                    }
                    Err(error) => return Err(error.to_string()),
                },
                Err(error) => return Err(error.to_string()),
            }
        }),
    );

    result
}

fn nether_portal_get_request(url: &str, member_id: String) -> Result<ureq::Response, ureq::Error> {
    let did_request_go_through = ureq::get(&format!("{}{}", url, &member_id)).call();

    did_request_go_through
}

pub fn nether_portal(
    borkcraft_self: &mut BorkCraft,
    ui: &mut egui::Ui,
    nether_portals_keys_url: &'static str,
    member_ids_url: &'static str,
) {
    START.call_once(|| {
        let nether_portals_am_clone = Arc::clone(&borkcraft_self.nether_portals);
        let error_message_am_clone = Arc::clone(&borkcraft_self.error_message);
        let modal_list_am_clone = Arc::clone(&borkcraft_self.modal_nether_portal.modal_list);
        thread::spawn(move || {
            // get nether portal keys from server
            // then match out the result and transfer ownership to App state
            let did_request_go_through = ureq::get(nether_portals_keys_url).call();
            let result = match_out_nether_portal_keys_to_result_string(did_request_go_through);
            match result {
                Ok(string) => {
                    let memberids_or_error_message = json_string_to_member_ids(string);
                    match memberids_or_error_message {
                        Ok(member_ids) => {
                            nether_portals_am_clone.lock().unwrap().whitelist = member_ids
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

            // create a useable list of member ids to somehow prevent deadlocks by using two locks on App state arc mutex...
            let mut member_ids = Vec::new();
            for member_id in &nether_portals_am_clone.lock().unwrap().whitelist.member_ids {
                member_ids.push(member_id.id.clone());

                //borkcraft_self
                //    .modal_nether_portal
                //    .modal_list
                //    .push(member_id.id.clone());
            }
            let mut something: Option<String> = None;
            if let None = &*modal_list_am_clone.lock().unwrap() {
                something = None;
            } else {
                something = Some("string".to_string());
            }
            match something {
                Some(_) => {}
                None => {
                    *modal_list_am_clone.lock().unwrap() = Some(member_ids.clone());
                }
            }
            // get the nether portal associated with each id in member_ids list
            // then assign its value to App State
            for id in member_ids {
                let did_request_go_through = nether_portal_get_request(member_ids_url, id.clone());
                let result = request_to_nether_portal(did_request_go_through);
                match result {
                    Ok(response_result) => {
                        if let ResponseResult::NetherPortal(nether_portal) = response_result {
                            nether_portals_am_clone
                                .lock()
                                .unwrap()
                                .nether_portals
                                .insert(id, nether_portal);
                        } else {
                            panic!("Magical Faries have occured at line 139 in nether_portals.rs");
                        }
                    }
                    Err(error) => {
                        *error_message_am_clone.lock().unwrap() = error;
                    }
                }
            }
        });
    });

    ui.label("you selected nether portals...!");
    // Things this function will need:
    //// 1) each nether portal object
    //// 2) each of those objects attached to the currently selected name
    //// 3) a modal wheel to select what user is chosen to display
    //// 4) some state to store the user that is chosen
    //// 5) a list of users to choose from stored in state

    match borkcraft_self.modal_nether_portal.modal_list.try_lock() {
        Ok(some_result) => {
            if let Some(result) = &*some_result {
                modal_machine(
                    &mut borkcraft_self.modal_nether_portal.modal,
                    ui,
                    &result,
                    8,
                );
            }
        }
        Err(_) => {
            ui.spinner();
        }
    }

    if borkcraft_self.modal_nether_portal.modal != "" {
        ui.push_id(6, |ui| {
            for (_key, value) in &borkcraft_self.nether_portals.lock().unwrap().nether_portals {
                let portal_value_names =
                    vec!["xcord", "ycord", "zcord", "locale", "owner", "notes"];
                for item in &portal_value_names {
                    let some_value: String;
                    match value.overworld.get(item) {
                        Ok(portal_value) => match portal_value {
                            PortalValue::Number(number) => {
                                some_value = number.to_string();
                            }
                            PortalValue::Text(text) => some_value = text.to_string(),
                        },
                        Err(error) => {
                            panic!("Magus Faries @ line 309: ... {}", error);
                        }
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(some_value);
                    });
                    //if *item == "notes" {
                    //} else {
                    //}
                }
            }

            // TableBuilder::new(ui)
            //     .striped(true)
            //     .cell_layout(egui::Layout::left_to_right(egui::Align::Max))
            //     .column(Size::initial(90.0).at_least(30.0))
            //     .column(Size::remainder().at_least(60.0))
            //     .header(20.0, |mut header| {
            //         header.col(|ui| {
            //             ui.heading("Row");
            //         });
            //         header.col(|ui| {
            //             ui.heading("Content");
            //         });
            //     })
            //     .body(|mut body| {
            //         let headers = ["Nether", "OverWorld"];
            //         let portal_value_names =
            //             vec!["xcord", "ycord", "zcord", "locale", "owner", "notes"];
            //         for (_key, value) in
            //             &borkcraft_self.nether_portals.lock().unwrap().nether_portals
            //         {
            //             for item in &portal_value_names {
            //                 let some_value: String;
            //                 match value.overworld.get(item) {
            //                     Ok(portal_value) => match portal_value {
            //                         PortalValue::Number(number) => {
            //                             some_value = number.to_string();
            //                         }
            //                         PortalValue::Text(text) => some_value = text.to_string(),
            //                     },
            //                     Err(error) => {
            //                         panic!("Magus Faries @ line 309: ... {}", error);
            //                     }
            //                 }
            //                 body.row(30.0, |mut row| {
            //                     row.col(|ui| {
            //                         ui.horizontal_wrapped(|ui| {
            //                             ui.label(some_value);
            //                         });
            //                         //if *item == "notes" {
            //                         //} else {
            //                         //}
            //                     });
            //                 });
            //             }
            //         }
            //     });
        });
    }
    // for (key, value) in &borkcraft_self.nether_portals.lock().unwrap().nether_portals {
    //     let formation = format!("id: {} \n{:?}\n", key, value);
    //     println!("{}", formation);
    // }
}
