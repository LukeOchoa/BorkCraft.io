use crate::pages::nether_portals;
// my crate imports
pub use crate::{
    errors::client_errors::*, login::login_page::*, pages::nether_portals::*, sessions::*,
};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;
use serde_json::Value;

// standard library
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Once},
};

const LOGIN_FORM: &'static [&'static str] = &["username", "password"];
const PAGE_OPTIONS: &'static [&'static str] = &["Login", "Nether Portals"];
static START: Once = Once::new();

const LOGOUT_URL: &'static str = "http://localhost:8123/nativelogout";
const LOGIN_URL: &'static str = "http://localhost:8123/nativelogin2";
const NETHER_PORTAL_KEYS_URL: &'static str = "http://localhost:8123/somelist";

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

pub struct BorkCraft {
    pub image_cache: Arc<Mutex<ImageCache>>,
    pub login_form: LoginForm,
    pub error_message: Arc<Mutex<ErrorMessage>>,
    pub session_information: Arc<Mutex<SessionInformation>>,
    pub selected_modal_page: String,
    pub nether_portals: Arc<Mutex<Vec<HashMap<i32, String>>>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        let session_informationx = Arc::new(Mutex::new(SessionInformation::default()));
        let nether_portalsx = Arc::new(Mutex::new(Vec::new()));
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: Arc::new(Mutex::new(ErrorMessage::default())),
            session_information: session_informationx,
            selected_modal_page: "".to_string(),
            nether_portals: nether_portalsx,
        }
    }
}

fn handle_errors(an_error: &mut ErrorMessage, ctx: &egui::Context, ui: &mut egui::Ui) {
    an_error.is_window_open = an_error.display_error(ctx);
    an_error.impure_open_error_window_on_click(ui);
}

fn display_session_time_left(ui: &mut egui::Ui, time_left: &TimeTime) {
    egui::Grid::new(4).show(ui, |ui| {
        ui.label(format!("session time remaining: \n{:?}", time_left));
    });
}

fn modal_machine(
    selected_modal: &mut String,
    ui: &mut egui::Ui,
    const_page_options: &'static [&'static str],
) {
    egui::ComboBox::from_label("Choose a Modal...!")
        .selected_text(selected_modal.clone())
        .show_ui(ui, |ui| {
            for option in const_page_options {
                ui.selectable_value(selected_modal, option.to_string(), *option);
            }
        });
}

//pub fn retrieve_user() {
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
//        Value::Null => println!("It was null!"),
//        Value::Bool(_boolean) => println!("It was a boolean!"),
//        Value::Number(_number) => println!("It was a number!"),
//        Value::String(_string) => println!("It was a String!"),
//        Value::Array(_vec) => println!("It was a Vector!"),
//        Value::Object(map) => println!("It was a map!\n {}", map["AllNetherPortals"][0]["Nether"]),
//    }
//}

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

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        START.call_once(|| {
            //let keysagain = retrieve_keys_to_nether_portals().unwrap();
            //println!("keys hopefully: \n{:?}", keysagain.get(&1).unwrap());
            current_session_time(Arc::clone(&self.session_information), ctx.clone());
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            handle_errors(&mut self.error_message.lock().unwrap(), ctx, ui);
            display_session_time_left(ui, &self.session_information.lock().unwrap().time);
            modal_machine(&mut self.selected_modal_page, ui, PAGE_OPTIONS);

            match self.selected_modal_page.as_str() {
                "Login" => login(self, ui, LOGIN_FORM, LOGIN_URL, LOGOUT_URL),
                "Nether Portals" => nether_portal(self, ui, NETHER_PORTAL_KEYS_URL),
                _ => {
                    ui.label("Where would you like to go Borker...?");
                }
            }
        });
    }
}
