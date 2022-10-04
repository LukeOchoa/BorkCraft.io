use crate::{
    borkcraft_app::{modal_machine, BorkCraft, ErrorMessage},
    eframe_tools::modal_machines::{self, act_on_tooth},
    thread_tools::ThreadPool,
    ureq_did_request_go_through_f, ResponseResult,
};
use eframe::egui::{self, plot::Text, Key, Response, Ui};
use egui_extras::{Size, TableBuilder};
use image::error;
use poll_promise::Promise;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::format,
    hash::Hash,
    sync::{
        mpsc::{self, Receiver, Sender},
        {Arc, Mutex, Once},
    },
    thread,
};
use ureq::Header;
static START: Once = Once::new();
const LIST1: &'static [&'static str] = &["1", "2", "3", "4", "5"];

pub struct NetherPortalModal {
    pub modal: String,
    pub modal_list: Arc<Mutex<Option<Vec<String>>>>,
}

// #[derive(Debug, Default)]
// pub struct ModalListItem {
//     id: String,
//     true_name: String,
// }

#[derive(Debug, Default)]
pub struct NewNetherPortalModal {
    pub modal: String,
    pub modal_list: Vec<String>, //Vec<ModalListItem>
}
#[derive(Debug, Default)]
pub struct NewNetherPortalInformation {
    pub modal_information: NewNetherPortalModal,
    pub all_nether_portals: HashMap<String, NetherPortal>,
    pub displayable_nether_portal: Option<NetherPortal>,
}
impl NewNetherPortalInformation {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn try_nether_portal_information(
        nether_portal_information_am: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
        ui: &mut eframe::egui::Ui,
        mut action: impl FnMut(&mut NewNetherPortalInformation, &mut egui::Ui),
    ) {
        match nether_portal_information_am.try_lock() {
            Ok(mut guarded_option) => match &mut *guarded_option {
                Some(nether_portal_information) => {
                    action(nether_portal_information, ui);
                }
                None => {
                    ui.spinner();
                }
            },
            Err(_) => {
                ui.spinner();
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct NetherPortalInformation {
    whitelist: MemberIds,
    nether_portals: HashMap<String, NetherPortal>,
}

#[derive(Debug, Deserialize, Default, Clone)]
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
impl NetherPortal {
    pub fn im_lazy_cloned(&self, index: &str) -> Result<Portal, String> {
        match index {
            "OverWorld" => return Ok(self.overworld.clone()),
            "Nether" => return Ok(self.nether.clone()),
            _ => Err("You have personally summoned `Special Magus Faries` for violating the contract of our forefathers...!".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
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
    #[serde(rename = "True_Name")]
    true_name: String,
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
impl<'a> PortalValue<'a> {
    pub fn to_string(&self) -> String {
        match self {
            PortalValue::Number(num) => return num.to_string(),
            PortalValue::Text(text) => return text.to_string(),
        }
    }
}

impl Portal {
    pub fn get(&self, index: &str) -> Result<PortalValue, String> {
        match index {
            "xcord" => Ok(PortalValue::Number(&self.xcord)),
            "ycord" => Ok(PortalValue::Number(&self.zcord)),
            "zcord" => Ok(PortalValue::Number(&self.ycord)),
            "locale" => Ok(PortalValue::Text(&self.locale)),
            "owner" => Ok(PortalValue::Text(&self.owner)),
            "notes" => Ok(PortalValue::Text(&self.notes)),
            "true_name" => Ok(PortalValue::Text(&self.true_name)),
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
    println!("the requested id!!!!!!!!!!!!!!!!11111 |{}|", member_id);
    let did_request_go_through = ureq::get(&format!("{}{}", url, &member_id)).call();

    did_request_go_through
}
fn get_count_of_nether_portals() -> Result<i32, String> {
    #[derive(Deserialize)]
    struct Count {
        count: String,
    }
    let result = ureq::get("http://localhost:8123/netherportalcount").call();
    let response = match result {
        Ok(response) => response,
        Err(error) => return Err(error.to_string()),
    };
    let json_string = match response.into_string() {
        Ok(some_string) => some_string,
        Err(_) => return Err("failed to convert response to string @ line 187".to_string()),
    };

    let some_count: Count = match serde_json::from_str(&json_string) {
        Ok(some_count) => some_count,
        Err(error) => return Err(error.to_string()),
    };

    Ok(some_count.count.parse().unwrap())
}

fn get_nether_portal_by_keyset_pagination(offset: i32) -> ureq::Response {
    loop {
        let limit = 5;
        let url = &format!(
            "http://localhost:8123/vecnetherportals?offset={}&limit={}",
            offset, limit
        );
        let result = ureq::get(url).call();
        if let Ok(response) = result {
            return response;
        }
        println!("we got stuck in get_nether_portal_by_keyset_pagination...?!");
    }
}
fn get_some_nether_portals(tx: Sender<HashMap<String, NetherPortal>>, offset: i32) {
    // get the data from the webserver, convert it into a useable data structure
    // then throw it downstream with the (mpsc::Sender)
    println!("occurance");
    let response = get_nether_portal_by_keyset_pagination(offset);
    let json_string = response.into_string().unwrap();
    let some_netherportals: HashMap<String, NetherPortal> =
        serde_json::from_str(&json_string).unwrap();

    tx.send(some_netherportals).unwrap()
}

fn start_up_things() -> Result<HashMap<String, NetherPortal>, String> {
    // Get the maximum item count for (nether portals) to be used as a SQL offset
    // TODO actually use this function LOL
    //let _amount_of_tasks = match get_count_of_nether_portals() {
    //    Ok(some_number) => some_number,
    //    Err(error_string) => return Err(error_string),
    //};

    // Create a threadpool and distribute the load amongst each thread as needed
    let (sender, receiver): (
        Sender<HashMap<String, NetherPortal>>,
        Receiver<HashMap<String, NetherPortal>>,
    ) = mpsc::channel();
    let pool = ThreadPool::new(4);

    let mut offset = 0;
    while offset < 5 {
        let tx = sender.clone();
        pool.execute(move || get_some_nether_portals(tx, offset));
        offset = offset + 5
    }
    // Read all data from threads into a container and return the container as a (Result)
    let mut netherportals: HashMap<String, NetherPortal> = HashMap::new();
    for some_netherportals in receiver.recv() {
        netherportals.extend(some_netherportals);
    }

    Ok(netherportals)
}

fn build_nether_portals_modal(
    some_nether_portals: &HashMap<String, NetherPortal>,
) -> NewNetherPortalModal {
    let mut some_modal = NewNetherPortalModal {
        modal: String::default(),
        modal_list: Vec::new(),
    };
    for (_key, some_nether_portal) in some_nether_portals {
        some_modal
            .modal_list
            .push(some_nether_portal.nether.true_name.clone());
        some_modal
            .modal_list
            .push(some_nether_portal.overworld.true_name.clone());
    }

    some_modal
}
fn get_related_nether_portals<'a>(
    nether_portal_information: &'a NewNetherPortalInformation,
    true_name: &str,
) -> Option<&'a NetherPortal> {
    for (_key, value) in &nether_portal_information.all_nether_portals {
        if &value.nether.true_name == true_name || &value.overworld.true_name == true_name {
            return Some(&value);
        }

        //}
    }
    None
}

fn modal_machine_for_nether_portals(
    some_nether_portal_information: &mut NewNetherPortalInformation,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    // Setup variables
    let some_modal = &mut some_nether_portal_information.modal_information.modal;
    let some_gear = modal_machines::ModalMachineGear::Immutable(
        &some_nether_portal_information.modal_information.modal_list,
    );
    // Create a modal machine from the modal list in all_nether_portal_information(NewNetherPortalInformation)
    let tooth = modal_machines::modal_machine(some_modal, ui, some_gear, 88);
    act_on_tooth(tooth, |some_option| {
        let some_nether_portal: Option<&NetherPortal> =
            get_related_nether_portals(&some_nether_portal_information, some_option);
        if let Some(nether_portal) = some_nether_portal {
            // Format a the information to be displayed
            // Save it in (App State)
            //let formated_item = format!("|{:?}|", nether_portal);
            //println!("{}", formated_item);
            some_nether_portal_information.displayable_nether_portal = Some(nether_portal.clone());
        } else {
            if let Ok(mut an_error_message) = error_message.try_lock() {
                *an_error_message = ErrorMessage::pure_error_message(Some(
                    "Royal Magus faries @ line 390".to_string(),
                ))
            }
        }
    });
}
fn displayable_nether_portal<'a>(
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    nether_portal_information: &'a NewNetherPortalInformation,
) {
    let portal_value_names = vec![
        "xcord",
        "ycord",
        "zcord",
        "locale",
        "owner",
        "notes",
        "true_name",
    ];

    fn make_rich(ss: String, size: Option<f32>) -> egui::widget_text::RichText {
        let some_size = match size {
            Some(some_size) => some_size,
            None => 25.0,
        };
        egui::RichText::new(ss).font(egui::FontId::proportional(some_size))
    }
    match &nether_portal_information.displayable_nether_portal {
        Some(nether_portal) => {
            let headers = ["Nether", "OverWorld"];

            for header in headers {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(20.0);
                    ui.end_row();
                    ui.label(make_rich(header.to_string(), Some(32.0)));
                    ui.end_row();
                    for item in &portal_value_names {
                        match nether_portal.im_lazy_cloned(header) {
                            Ok(portal) => match portal.get(&item) {
                                Ok(pv) => {
                                    ui.label(make_rich(item.to_string(), None));
                                    ui.label(make_rich("=>".to_string(), None));
                                    ui.label(make_rich(pv.to_string(), None));
                                    ui.end_row();
                                }
                                Err(error_string) => {
                                    *error_message.lock().unwrap() =
                                        ErrorMessage::pure_error_message(Some(error_string));
                                }
                            },
                            Err(error_string) => {
                                *error_message.lock().unwrap() =
                                    ErrorMessage::pure_error_message(Some(error_string));
                            }
                        }
                    }
                });
            }
        }
        None => {}
    };
}
pub fn new_nether_portal(
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    all_nether_portal_information: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
    ui: &mut egui::Ui,
) {
    START.call_once(|| {
        // create variables usable inside thread::spawn
        let error_message_am_clone = Arc::clone(error_message);
        let all_nether_portal_information_am_clone = Arc::clone(all_nether_portal_information);

        thread::spawn(move || {
            // Retrieve nether portal information from webserver and handle any errors
            let nether_portals = match start_up_things() {
                Ok(some_netherportals) => some_netherportals,
                Err(error_string) => {
                    *error_message_am_clone.lock().unwrap() =
                        ErrorMessage::pure_error_message(Some(error_string));
                    panic!("lol i ended it manually");
                }
            };
            // Create a usable "gear"/Vec<String> of information for modal_machine so the user can choose the appropriate portal
            // Also save all information gained from start_up_things() to main app `State` through an (A.M. Clone)
            let some_modal = build_nether_portals_modal(&nether_portals);
            *all_nether_portal_information_am_clone.lock().unwrap() =
                Some(NewNetherPortalInformation {
                    modal_information: some_modal,
                    all_nether_portals: nether_portals,
                    displayable_nether_portal: None,
                });
        });
    });

    // use try_nether_portal_information to gain access to its value
    NewNetherPortalInformation::try_nether_portal_information(
        all_nether_portal_information,
        ui,
        |some_nether_portal_information, ui| {
            // create a modal machine with its information
            modal_machine_for_nether_portals(some_nether_portal_information, ui, error_message);
            // display the information inside the datastructure to the ui
            //if let Some(some_string) = &some_nether_portal_information.displayable_nether_portal {
            //    ui.label(egui::RichText::new(some_string).font(egui::FontId::proportional(40.0)));
            //    //ui.label(some_string).
            //}
            displayable_nether_portal(ui, error_message, some_nether_portal_information);
        },
    );
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
            println!(
                "{:?}",
                nether_portals_am_clone.lock().unwrap().nether_portals //borkcraft_self.nether_portals.lock().unwrap().nether_portals
            );

            let mut member_ids = Vec::new();
            let somethingref = &nether_portals_am_clone.lock().unwrap().nether_portals;
            for (_key, value) in somethingref {
                member_ids.push(value.overworld.locale.clone())
            }
            *modal_list_am_clone.lock().unwrap() = Some(member_ids);
        });
    });

    ui.label("you selected nether portals...!");
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
            egui::Resize::default()
                .default_height(100.0)
                .show(ui, |ui| {
                    for (_key, value) in
                        &borkcraft_self.nether_portals.lock().unwrap().nether_portals
                    {
                        let portal_value_names = vec![
                            "xcord",
                            "ycord",
                            "zcord",
                            "locale",
                            "owner",
                            "notes",
                            "true_name",
                        ];
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
                                ui.label(item.to_string());
                                ui.label("=>");
                                ui.label(some_value);
                                ui.end_row();
                            });
                        }
                    }
                })
        });
    }
}
