// my crate imports
pub use crate::{
    errors::client_errors::*, login::login_page::*, pages::nether_portals::*, sessions::*,
    windows::client_windows::WindowMessage,
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
static START: Once = Once::new();

const _PUBLIC_DNS: &'static str = "ec2-54-176-200-180.us-west-1.compute.amazonaws.com";

const LOGOUT_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/logout"; //"localhost:8334/logout"; // REPLACED! "http://localhost:8123/nativelogout";
const LOGIN_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/login"; //"http://localhost:8334/login"; //  REPLACED! "http://localhost:8123/nativelogin2";

const ADD_NETHER_PORTAL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/addnetherportaltext"; // "http://localhost:8334/addnetherportaltext"; // REPLACED! "http://localhost:8123/addnetherportal";

const SAVE_IMAGE_DETAILS_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/addnetherportalimagedetails";
//"http://" + PUBLIC_DNS + ":8334/addnetherportalimagedetails"; // REPLACED! "http://localhost:8334/savenetherportalimagedetails";    // "http://localhost:8123/saveimagefromclient";

const SAVE_IMAGE_URL: &'static str = "http://localhost:1234/saveimage"; // DONT REPLACE YET
const SAVE_NETHER_PORTAL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/savenetherportaltextchanges";
// "http://localhost:8334/savenetherportaltextchanges"; // REPLACED! "http://localhost:8123/savenetherportals";

const DELETE_IMAGE: &'static str = "http://localhost:1234/deleteimage"; // DONT REPLACE YET
const DELETE_IMAGE_FROM_CLIENT: &'static str = "http://localhost:1234/deleteimagefromclient"; // DONT REPLACE YET "http://localhost:1234/deleteimage?name={}"

const RETRIEVE_NETHER_PORTALS: &'static str = "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/getnetherportalstextinformation";
// "http://localhost:8334/getnetherportalstextinformation?"; // REPLACED! "http://localhost:8123/vecnetherportals?";

const GET_ACCESS_RIGHTS_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/getaccessrights"; // "http://localhost:8334/getaccessrights"; // REPLACED! "http://localhost:8123/getaccessrights";
const GET_NETHER_PORTAL_IMAGES_URL: &'static str = "http://localhost:1234/getnetherportalimage"; // DONT REPLACE YET

const SESSION_TIME_LEFT_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/sessiontimeleft"; // "http://localhost:8334/sessiontimeleft"; //  REPLACED! "http://localhost:8123/sessiontimeleft";

const GET_NETHER_PORTAL_IMAGE_NAMES_URL: &'static str =
    "http://ec2-54-176-200-180.us-west-1.compute.amazonaws.com:8334/getnetherportalimagenames";
// "http://localhost:8123/getnetherportalimagenames"; //  REPLACED! "http://localhost:8123/getnetherportalimagenames?true_name={}",

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
    pub user_picked_filepath: Option<String>,
    pub image_cache: Arc<Mutex<ImageCache>>,
    pub login_form: LoginForm,
    pub error_message: Arc<Mutex<ErrorMessage>>,
    pub window_message: Arc<Mutex<WindowMessage>>,
    pub session_information: Arc<Mutex<SessionInformation>>,
    pub selected_modal_page: String,
    pub modal_nether_portal: NetherPortalModal,
    pub all_nether_portal_information: Arc<Mutex<Option<NewNetherPortalInformation>>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
            error_message: Arc::new(Mutex::new(ErrorMessage::default())),
            window_message: Arc::new(Mutex::new(WindowMessage::default())),
            session_information: Arc::new(Mutex::new(SessionInformation::default())),
            selected_modal_page: "".to_string(),
            modal_nether_portal: NetherPortalModal {
                modal: "".to_string(),
                modal_list: Arc::new(Mutex::new(None)),
            },
            //nether_portals: Arc::new(Mutex::new(NetherPortalInformation::default())),
            all_nether_portal_information: Arc::new(Mutex::new(None)),
            user_picked_filepath: None,
        }
    }
}

fn handle_errors(an_error: &mut ErrorMessage, ctx: &egui::Context, ui: &mut egui::Ui) {
    an_error.is_window_open = an_error.display_error(ctx);
    an_error.impure_open_error_window_on_click(ui);
}
fn handle_window_message(a_message: &mut WindowMessage, ctx: &egui::Context, ui: &mut egui::Ui) {
    a_message.is_window_open = a_message.display_message(ctx);
    a_message.open_window_on_click(ui, "Client Messages");
}

fn display_session_time_left(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    window_message: &mut WindowMessage,
) {
    window_message.is_window_open = window_message.display_message(ctx);
    window_message.open_window_on_click(ui, "Session Time Left");
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
            current_session_time(
                Arc::clone(&self.session_information),
                SESSION_TIME_LEFT_URL,
                ctx.clone(),
            );
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                handle_errors(&mut self.error_message.lock().unwrap(), ctx, ui);
                handle_window_message(&mut self.window_message.lock().unwrap(), ctx, ui);
                display_session_time_left(
                    ui,
                    ctx,
                    &mut self.session_information.lock().unwrap().window_message,
                );
                ui.end_row();
            });
            modal_machine(
                &mut self.selected_modal_page,
                ui,
                &vec!["Login".to_string(), "Nether Portals".to_string()],
                7,
            );

            match self.selected_modal_page.as_str() {
                "Login" => login(
                    self,
                    ui,
                    LOGIN_FORM,
                    LOGIN_URL,
                    LOGOUT_URL,
                    GET_ACCESS_RIGHTS_URL,
                ),
                "Nether Portals" => {
                    //let time = self.session_information.lock().unwrap().time.second.clone();
                    if self.session_information.lock().unwrap().is_logged_in {
                        new_nether_portal(
                            &mut self.error_message,
                            &mut self.window_message,
                            &mut self.all_nether_portal_information,
                            &self.session_information,
                            &mut self.user_picked_filepath,
                            &self.login_form.username,
                            GET_NETHER_PORTAL_IMAGE_NAMES_URL,
                            GET_NETHER_PORTAL_IMAGES_URL,
                            DELETE_IMAGE,
                            DELETE_IMAGE_FROM_CLIENT,
                            SAVE_NETHER_PORTAL,
                            RETRIEVE_NETHER_PORTALS,
                            ADD_NETHER_PORTAL,
                            SAVE_IMAGE_URL,
                            SAVE_IMAGE_DETAILS_URL,
                            ui,
                            ctx.clone(),
                        )
                    } else {
                        ui.label("Please Log-in to see this information...");
                    }
                } //nether_portal(self, ui, NETHER_PORTAL_KEYS_URL, MEMBER_IDS_URL),
                _ => {
                    ui.label("Where would you like to go Borker...?");
                }
            }
        });
    }
}
