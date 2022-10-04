// my crate imports
pub use crate::{
    errors::client_errors::*, login::login_page::*, pages::nether_portals::*, sessions::*,
};

// emilk imports
use eframe::egui;
use egui_extras::RetainedImage;
//use serde_derive::Deserialize;

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
const MEMBER_IDS_URL: &'static str = "http://localhost:8123/sendmember?id=";

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
    pub modal_nether_portal: NetherPortalModal,
    pub nether_portals: Arc<Mutex<NetherPortalInformation>>,
    pub all_nether_portal_information: Arc<Mutex<Option<NewNetherPortalInformation>>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: Arc::new(Mutex::new(ErrorMessage::default())),
            session_information: Arc::new(Mutex::new(SessionInformation::default())),
            selected_modal_page: "".to_string(),
            modal_nether_portal: NetherPortalModal {
                modal: "".to_string(),
                modal_list: Arc::new(Mutex::new(None)),
            },
            nether_portals: Arc::new(Mutex::new(NetherPortalInformation::default())),
            all_nether_portal_information: Arc::new(Mutex::new(None)),
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

pub fn modal_machine(
    selected_modal: &mut String,
    ui: &mut egui::Ui,
    const_page_options: &Vec<String>,
    ui_id: i32,
) {
    ui.push_id(ui_id, |ui| {
        egui::ComboBox::from_label("Choose a Modal...!")
            .selected_text(selected_modal.clone())
            .show_ui(ui, |ui| {
                for option in const_page_options {
                    ui.selectable_value(selected_modal, option.to_string(), option);
                }
            });
    });
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        START.call_once(|| {
            current_session_time(Arc::clone(&self.session_information), ctx.clone());
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            handle_errors(&mut self.error_message.lock().unwrap(), ctx, ui);
            display_session_time_left(ui, &self.session_information.lock().unwrap().time);
            modal_machine(
                &mut self.selected_modal_page,
                ui,
                &vec!["Login".to_string(), "Nether Portals".to_string()],
                7,
            );

            match self.selected_modal_page.as_str() {
                "Login" => login(self, ui, LOGIN_FORM, LOGIN_URL, LOGOUT_URL),
                "Nether Portals" => new_nether_portal(
                    &mut self.error_message,
                    &mut self.all_nether_portal_information,
                    ui,
                ), //nether_portal(self, ui, NETHER_PORTAL_KEYS_URL, MEMBER_IDS_URL),
                _ => {
                    ui.label("Where would you like to go Borker...?");
                }
            }
        });
    }
}

// fn _some_function(ui: &mut egui::Ui, _ctx: &egui::Context) {
//     //egui::ScrollArea::vertical().show(ui, |ui| {
//     //.drag_bounds(egui::Rect {
//     //    min: egui::pos2(30.0, 30.0),
//     //    max: egui::pos2(300.0, 300.0),
//     //})
//     egui::Resize::default()
//         .default_height(100.0)
//         .show(ui, |ui| {
//             ui.horizontal_wrapped(|ui| {
//                 for i in 0..100 {
//                     ui.label(i.to_string());
//                 }
//             });
//         });
//     //});
// }
