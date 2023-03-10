use crate::{
    borkcraft_app::ErrorMessage, //{modal_machine, BorkCraft, ErrorMessage},
    borkcraft_app::WindowMessage,
    eframe_tools::egui_tools::{get_file_name, path_of_image_to_vec_u8},
    eframe_tools::modal_machines::{self, act_on_tooth},
    images::image::{
        display_nether_portal_images, make_partial_gear, ImageAndDetails, ImageCollection,
        ImageDetails,
    },
    local_files::add_files,
    sessions::SessionInformation,
    thread_tools::ThreadPool,
    to_vec8,
    url_tools::*,
    windows::client_windows::{DeletionQueue, GenericWindow, Victim},
};

use eframe::egui::{self};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, {Mutex, Once},
    },
    thread,
};
static START: Once = Once::new();

pub struct NetherPortalModal {
    pub modal: String,
    pub modal_list: Arc<Mutex<Option<Vec<String>>>>,
}

#[derive(Debug, Default)]
pub struct NewNetherPortalModal {
    pub modal: String,
    pub previous_modal: String,
    pub modal_list: Vec<String>, //Vec<ModalListItem>
    pub image_modal: String,
    pub deletion_modal: String,
}

// all_nether_portal_images needs a way to hold information that says
// : "This HashMap has not been built out with its necessary information yet"
// And
// : "This HashMap is in the process of being built out, Please do not try to build it again"
// Maybe use an Enum that is either: Hasher, Nothing, BeingBuilt
pub enum StateOfImages {
    HashMap(HashMap<String, ImageAndDetails>),
    BeingBuilt,
    Nothing,
    BuildFailed,
}
impl StateOfImages {
    pub fn hashmap_mut(&mut self) -> Option<&mut HashMap<String, ImageAndDetails>> {
        if let StateOfImages::HashMap(hasher) = self {
            Some(hasher)
        } else {
            None
        }
    }
    pub fn hashmap_ref(&self) -> Option<&HashMap<String, ImageAndDetails>> {
        if let StateOfImages::HashMap(hasher) = self {
            Some(hasher)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct NewNetherPortalInformation {
    pub modal_information: NewNetherPortalModal,
    pub all_nether_portals: HashMap<String, NetherPortal>,
    pub all_nether_portal_images: Arc<Mutex<HashMap<String, StateOfImages>>>, //HashMap<String, ImageAndDetails>>>>,
    pub displayable_nether_portal: Option<(String, StringNetherPortal)>,
    pub copy_of_nether_portals: HashMap<String, NetherPortal>,
    pub nether_portal_deletion_wheel: Option<DeletionQueue>,
    pub image_deletion_window: GenericWindow,
    pub building_a_nether_portal: Arc<Mutex<(StringNetherPortal, bool)>>,
    pub actual_nether_portal_deletion: HashMap<String, bool>,
    pub actual_nether_portal_generic_window: GenericWindow,
    pub modify: bool,
}

impl NewNetherPortalInformation {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn try_nether_portal_information(
        nether_portal_information_am: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
        ui: &mut eframe::egui::Ui,
        ctx: egui::Context,
        mut action: impl FnMut(&mut NewNetherPortalInformation, &mut egui::Ui, egui::Context),
    ) {
        match nether_portal_information_am.try_lock() {
            Ok(mut guarded_option) => match &mut *guarded_option {
                Some(nether_portal_information) => {
                    action(nether_portal_information, ui, ctx);
                }
                None => {
                    println!("u1");
                    ui.spinner();
                }
            },
            Err(_) => {
                println!("u2");
                ui.spinner();
            }
        }
    }
}
fn get_ref_by_field<'a>(
    hasher: &'a HashMap<String, NetherPortal>,
    field_name: &str,
    field_value: &str,
) -> Result<(String, &'a NetherPortal), String> {
    for (key, netherportal) in hasher.iter() {
        if netherportal.nether.get(field_name)?.to_string() == field_value
            || &netherportal.overworld.get(field_name)?.to_string() == field_value
        {
            return Ok((key.to_string(), netherportal));
        }
    }
    Err("Field does not exist".to_string())
}

fn get_mut_ref_by_field<'a>(
    //&mut self,
    hasher: &'a mut HashMap<String, NetherPortal>,
    field_name: &str,
    field_value: &str,
) -> Result<(String, &'a mut NetherPortal), String> {
    for (key, netherportal) in &mut hasher.iter_mut() {
        if netherportal.nether.get(field_name)?.to_string() == field_value
            || &netherportal.overworld.get(field_name)?.to_string() == field_value
        {
            return Ok((key.to_string(), netherportal));
        }
    }
    Err("Field does not exist".to_string())
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct StringNetherPortal {
    pub id: String,
    pub overworld: StringPortal,
    pub nether: StringPortal,
    pub username: String,
}
impl StringNetherPortal {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn get_mut_ref(&mut self, index: &str) -> Result<&mut StringPortal, String> {
        match index {
            "OverWorld" => return Ok(&mut self.overworld),
            "Nether" => return Ok(&mut self.nether),
            _ => Err("You have personally summoned `Special Magus Faries` for violating the contract of our forefathers...!".to_string()),
        }
    }

    pub fn get(&self, index: &str) -> Result<&StringPortal, String> {
        match index {
            "OverWorld" => return Ok(&self.overworld),
            "Nether" => return Ok(&self.nether),
            _ => Err("You have personally summoned `Special Magus Faries` for violating the contract of our forefathers...!".to_string()),
        }
    }

    pub fn convert(nether_portal: NetherPortal) -> Result<StringNetherPortal, String> {
        let portal_value_names = vec![
            "xcord",
            "ycord",
            "zcord",
            "locale",
            "owner",
            "notes",
            "true_name",
        ];
        let mut nether: StringPortal = StringPortal::default();
        for item in &portal_value_names {
            *nether.get_mut(item)? = nether_portal.nether.get(item)?.to_string();
        }
        let mut overworld: StringPortal = StringPortal::default();
        for item in &portal_value_names {
            *overworld.get_mut(item)? = nether_portal.overworld.get(item)?.to_string();
        }
        let string_nether_portal = StringNetherPortal {
            id: nether_portal.id.to_string(),
            nether,
            overworld,
            username: nether_portal.username,
        };

        Ok(string_nether_portal)
    }
}
impl NetherPortal {
    pub fn im_lazy_cloned(&self, index: &str) -> Result<Portal, String> {
        match index {
            "OverWorld" => return Ok(self.overworld.clone()),
            "Nether" => return Ok(self.nether.clone()),
            _ => Err("You have personally summoned `Special Magus Faries` for violating the contract of our forefathers...!".to_string()),
        }
    }
    pub fn get_mut_ref(&mut self, index: &str) -> Result<&mut Portal, String> {
        match index {
            "OverWorld" => return Ok(&mut self.overworld),
            "Nether" => return Ok(&mut self.nether),
            _ => Err("You have personally summoned `Special Magus Faries` for violating the contract of our forefathers...!".to_string()),
        }
    }

    pub fn convert(string_nether_portal: &StringNetherPortal) -> Result<NetherPortal, String> {
        let portal_value_names = vec![
            "xcord",
            "ycord",
            "zcord",
            "locale",
            "owner",
            "notes",
            "true_name",
        ];

        let (mut nether, mut overworld) = (Portal::default(), Portal::default());
        for index in &portal_value_names {
            Portal::save(
                nether.get_mut(index)?,
                string_nether_portal.nether.get_clone(index)?,
            )?;
            Portal::save(
                overworld.get_mut(index)?,
                string_nether_portal.overworld.get_clone(index)?,
            )?;
        }
        let id = match string_nether_portal.id.parse::<i32>() {
            Ok(number) => number,
            Err(error) => return Err(error.to_string()),
        };
        Ok(NetherPortal {
            id,
            nether,
            overworld,
            username: string_nether_portal.username.to_string(),
            //modified: true,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
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

#[derive(Debug, Default, Clone, Serialize)]
pub struct StringPortal {
    xcord: String,
    ycord: String,
    zcord: String,
    locale: String,
    owner: String,
    notes: String,
    true_name: String,
}

#[derive(Debug)]
pub enum PortalValue<'a> {
    Text(&'a String),
    MutText(&'a mut String),
    Number(&'a i32),
    MutNumber(&'a mut i32),
}
impl<'a> PortalValue<'a> {
    pub fn to_string(&self) -> String {
        match self {
            PortalValue::Number(num) => return num.to_string(),
            PortalValue::Text(text) => return text.to_string(),
            PortalValue::MutNumber(mut_num) => return mut_num.to_string(),
            PortalValue::MutText(mut_text) => return mut_text.to_string(),
        }
    }
}

impl StringPortal {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn get_clone(&self, index: &str) -> Result<String, String> {
        let result = match index {
            "xcord" => Ok(&self.xcord),
            "ycord" => Ok(&self.zcord),
            "zcord" => Ok(&self.ycord),
            "locale" => Ok(&self.locale),
            "owner" => Ok(&self.owner),
            "notes" => Ok(&self.notes),
            "true_name" => Ok(&self.true_name),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        };
        match result {
            Ok(value) => return Ok(value.to_string()),
            Err(error_string) => return Err(error_string),
        }
    }
    pub fn get_ref(&self, index: &str) -> Result<&String, String> {
        match index {
            "xcord" => Ok(&self.xcord),
            "ycord" => Ok(&self.zcord),
            "zcord" => Ok(&self.ycord),
            "locale" => Ok(&self.locale),
            "owner" => Ok(&self.owner),
            "notes" => Ok(&self.notes),
            "true_name" => Ok(&self.true_name),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        }
    }
    pub fn get_mut(&mut self, index: &str) -> Result<&mut String, String> {
        match index {
            "xcord" => Ok(&mut self.xcord),
            "ycord" => Ok(&mut self.zcord),
            "zcord" => Ok(&mut self.ycord),
            "locale" => Ok(&mut self.locale),
            "owner" => Ok(&mut self.owner),
            "notes" => Ok(&mut self.notes),
            "true_name" => Ok(&mut self.true_name),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
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
    pub fn get_mut(&mut self, index: &str) -> Result<PortalValue, String> {
        match index {
            "xcord" => Ok(PortalValue::MutNumber(&mut self.xcord)),
            "ycord" => Ok(PortalValue::MutNumber(&mut self.zcord)),
            "zcord" => Ok(PortalValue::MutNumber(&mut self.ycord)),
            "locale" => Ok(PortalValue::MutText(&mut self.locale)),
            "owner" => Ok(PortalValue::MutText(&mut self.owner)),
            "notes" => Ok(PortalValue::MutText(&mut self.notes)),
            "true_name" => Ok(PortalValue::MutText(&mut self.true_name)),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        }
    }

    pub fn save(pv: PortalValue, state: String) -> Result<(), String> {
        match pv {
            PortalValue::MutText(text) => *text = state,
            PortalValue::MutNumber(number) => {
                let result = state.parse::<i32>();
                match result {
                    Ok(some_number) => *number = some_number,
                    Err(error) => return Err(error.to_string()),
                }
            }
            _ => return Err(format!("Cannot mutate: |{:?}| type PortalValue", pv)),
        }

        Ok(())
    }
}

fn get_nether_portal_by_keyset_pagination(offset: i32, url: String) -> ureq::Response {
    //offset={}&limit={}",
    loop {
        let limit = 5;
        let url = &format!(
            //"http://localhost:8123/vecnetherportals?,
            "{}?offset={}&limit={}",
            url, offset, limit
        );
        println!("{}", url);
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
    // retrieve_nether_portals_url
    let response =
        get_nether_portal_by_keyset_pagination(offset, Urls::default(Routes::GetNetherPortalBunch));
    let json_string = response.into_string().unwrap();
    let some_netherportals: HashMap<String, NetherPortal> =
        serde_json::from_str(&json_string).unwrap();
    println!("data from the webserver... |{:?}|", some_netherportals);
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

    let mut offset = -1;
    while offset < 4 {
        let tx = sender.clone();
        pool.execute(move || get_some_nether_portals(tx, offset));
        offset = offset + 5;
        println!("HOW MANY TIMES DID YOU CALL");
    }
    // Read all data from threads into a container and return the container as a (Result)
    // The String in HashMap<String, NetherPortal> is the index primary key from the database
    let mut netherportals: HashMap<String, NetherPortal> = HashMap::new();

    println!("PAST THE LOOP ##################################################################################");
    #[allow(for_loops_over_fallibles)] // how to fix...?
    let mut cnt = 0;
    loop {
        if cnt == 1 {
            break;
        }

        let obj = receiver.recv();
        if let Ok(x) = obj {
            netherportals.extend(x);
        }
        //match receiver.recv() {
        //    Ok(x) => {
        //        netherportals.extend(x);
        //        println!("more magic is happening");
        //    }
        //    Err(_) => return Ok(netherportals),
        //}
        println!("is the loop stuck?");
        cnt = cnt + 1;
    }
    println!("PAST THE LOOP ----------------------------------------------------------------------------------");
    //for x in receiver.recv() {
    //    println!("x == |{:?}|", x);
    //    netherportals.extend(x);
    //}

    Ok(netherportals)
}

fn build_nether_portals_modal(
    some_nether_portals: &HashMap<String, NetherPortal>,
) -> NewNetherPortalModal {
    let mut some_modal = NewNetherPortalModal {
        modal: String::default(),
        previous_modal: String::default(),
        modal_list: Vec::new(),
        image_modal: String::default(),
        deletion_modal: String::default(),
    };
    // Make a list of names based on (true_name) to be used as a selector for the user modal
    for (_key, some_nether_portal) in some_nether_portals {
        println!(
            "|{}  ---- {} |",
            some_nether_portal.nether.true_name, some_nether_portal.overworld.true_name
        );
        some_modal
            .modal_list
            .push(some_nether_portal.nether.true_name.clone());
        some_modal
            .modal_list
            .push(some_nether_portal.overworld.true_name.clone());
    }

    println!("did something get built!");
    some_modal
}

fn _get_related_nether_portal<'a>(
    nether_portal_information: &'a NewNetherPortalInformation,
    true_name: &str,
) -> Option<(String, NetherPortal)> {
    //! Returns the counterpart (Portal) struct value.
    //!
    //! If the user chooses to display the 'nether' variant of the portal location,
    //! You must also show its 'overworld' counterpart and visa versa...!
    for (key, netherportal) in &nether_portal_information.all_nether_portals {
        if &netherportal.nether.true_name == true_name
            || &netherportal.overworld.true_name == true_name
        {
            return Some((key.to_string(), netherportal.clone()));
        }
    }
    None
}

fn store_temporary_user_edits(
    some_nether_portal_information: &mut NewNetherPortalInformation,
) -> Option<String> {
    if some_nether_portal_information.modify {
        if let Some(value) = &some_nether_portal_information.displayable_nether_portal {
            println!("am i seen: |{:?}|", value.1);
        }
        let some_true_name = &some_nether_portal_information
            .modal_information
            .previous_modal;
        for (_key, netherportal) in &mut some_nether_portal_information.all_nether_portals {
            if &netherportal.nether.true_name == some_true_name
                || &netherportal.overworld.true_name == some_true_name
            {
                if let Some(some_tuple) =
                    &mut some_nether_portal_information.displayable_nether_portal
                {
                    match NetherPortal::convert(&some_tuple.1.clone()) {
                        Ok(converted_netherportal) => {
                            *netherportal = converted_netherportal;
                            println!("SUCCESSFULL CONVERSION");
                        }
                        Err(error_string) => return Some(error_string),
                    }
                }
            }
        }
    }

    None
}

fn make_displayable_netherportal(
    all_nether_portals: &HashMap<String, NetherPortal>,
    true_name: &str,
) -> Result<(String, StringNetherPortal), String> {
    //! Create a StringNetherPortal from a hashmap of NetherPortals that can be displayed

    // Get the netherportal from the master list of information
    let netherportal_tuple = get_ref_by_field(all_nether_portals, "true_name", true_name)?;

    // Convert it a StringNetherPortal
    let string_netherportal = StringNetherPortal::convert(netherportal_tuple.1.clone())?;

    Ok((netherportal_tuple.0, string_netherportal))
}

fn modal_machine_for_nether_portals(
    some_nether_portal_information: &mut NewNetherPortalInformation,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    some_nether_portal_information
        .modal_information
        .previous_modal = some_nether_portal_information
        .modal_information
        .modal
        .clone();

    // Setup variables
    let some_modal = &mut some_nether_portal_information.modal_information.modal;
    let some_gear = modal_machines::ModalMachineGear::Immutable(
        &some_nether_portal_information.modal_information.modal_list,
    );
    // Create a modal machine from the modal list in all_nether_portal_information(NewNetherPortalInformation)

    let tooth =
        modal_machines::modal_machine(some_modal, ui, some_gear, Some("Select A NetherPortal"), 88);
    act_on_tooth(tooth, |some_option| {
        // if the user is in the middle of modifing a value and they choose a different modal...
        // do this
        if let Some(error_string) = store_temporary_user_edits(some_nether_portal_information) {
            *error_message.lock().unwrap() = ErrorMessage::pure_error_message(Some(error_string));
        }

        // TODO remember to take the comments from the old implementation below this code block (that is commented out)
        // and add them to this new function and here were necessary
        // also use this function in another spot to fix the fact that the "MODIFY".button removes your changes if you unmodify and then
        // select a different netherportal modal.
        // I need to temp save those changes in the (struct.displayable_nether_portal) `App State` variable
        match make_displayable_netherportal(
            &some_nether_portal_information.all_nether_portals,
            some_option,
        ) {
            Ok(string_netherportal_tuple) => {
                some_nether_portal_information.displayable_nether_portal =
                    Some(string_netherportal_tuple)
            }
            Err(error_string) => {
                *error_message.lock().unwrap() =
                    ErrorMessage::pure_error_message(Some(error_string))
            }
        }

        // TODO somehow create a thread that watches for external changes from other potenial clients that write info to the
        // database. Then update the client in a user friendly way... more magic...
    });
}

fn make_rich(ss: String, size: Option<f32>) -> egui::widget_text::RichText {
    let some_size = match size {
        Some(some_size) => some_size,
        None => 25.0,
    };
    egui::RichText::new(ss).font(egui::FontId::proportional(some_size))
}
fn displayable_nether_portal<'a>(
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    nether_portal_information: &mut NewNetherPortalInformation,
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

    match &mut nether_portal_information.displayable_nether_portal {
        Some(nether_portal) => {
            let headers = ["Nether", "OverWorld"];
            for header in headers {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(20.0);
                    ui.end_row();
                    ui.label(make_rich(header.to_string(), Some(32.0)));
                    ui.end_row();
                    for item in &portal_value_names {
                        match nether_portal.1.get_mut_ref(header) {
                            Ok(portal) => match portal.get_mut(&item) {
                                Ok(pv) => {
                                    ui.label(make_rich(item.to_string(), None));
                                    ui.label(make_rich("=>".to_string(), None));
                                    if !nether_portal_information.modify {
                                        ui.label(make_rich(pv.to_string(), None));
                                    } else {
                                        ui.add(egui::TextEdit::singleline(pv));
                                    }
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

fn save_modified_data(
    all_nether_portals: &mut HashMap<String, NetherPortal>,
    displayable_nether_portal: &mut Option<(String, StringNetherPortal)>,
    current_netherportal_modal: &String,
) -> Option<String> {
    for (_key, netherportal) in all_nether_portals {
        if &netherportal.nether.true_name == current_netherportal_modal
            || &netherportal.overworld.true_name == current_netherportal_modal
        {
            if let Some(some_tuple) = displayable_nether_portal {
                match NetherPortal::convert(&some_tuple.1.clone()) {
                    Ok(converted_netherportal) => {
                        *netherportal = converted_netherportal;
                        println!("SUCCESSFULL CONVERSION");
                    }
                    Err(error_string) => {
                        return Some(error_string);
                    }
                }
            }
        }
    }
    return None;
}
fn do_you_have_access_rights(
    session_information: &Arc<Mutex<SessionInformation>>,
    current_netherportal: &String,
) -> bool {
    if session_information
        .lock()
        .unwrap()
        .access_rights
        .contains(current_netherportal)
    {
        true
    } else {
        false
    }
}

fn modify_button(
    modify: &mut bool,
    all_nether_portals: &mut HashMap<String, NetherPortal>,
    displayable_nether_portal: &mut Option<(String, StringNetherPortal)>,
    current_netherportal_modal: &String,
    session_information: &Arc<Mutex<SessionInformation>>,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    do_you_have_access_rights(session_information, current_netherportal_modal);
    if *current_netherportal_modal != String::default()
        && do_you_have_access_rights(session_information, current_netherportal_modal)
    {
        if ui
            .button(format!("Modify NetherPortal: {}", modify))
            .clicked()
        {
            //

            println!("give me my fried chimken ZEUS!");

            if !*modify {
                // TODO: create a different version of this function so that it pulls from a StringNetherPortal and not the currently saved HashMap<String, NetherPortal>
                match make_displayable_netherportal(all_nether_portals, &current_netherportal_modal)
                {
                    Ok(netherportal_tuple) => *displayable_nether_portal = Some(netherportal_tuple),
                    Err(error_string) => {
                        *error_message.lock().unwrap() =
                            ErrorMessage::pure_error_message(Some(error_string))
                    }
                }
            } else {
                println!("NETHER PORTAL NAME?: {}", current_netherportal_modal);
                if let Some(error_string) = save_modified_data(
                    all_nether_portals,
                    displayable_nether_portal,
                    current_netherportal_modal,
                ) {
                    *error_message.lock().unwrap() =
                        ErrorMessage::pure_error_message(Some(error_string))
                }
            }

            *modify = !*modify;
        }
    }
}

fn reset_netherportal_button(
    nether_portal_information: &mut NewNetherPortalInformation,
    true_name: &str,
    ui: &mut egui::Ui,
    ctx: egui::Context,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    //! Resets the nether portal current selected and being displayed
    fn subfn(
        // private function to use sugarsyntax for returning Err values
        nether_portal_information: &mut NewNetherPortalInformation,
        field_name: &str,
        field_value: &str,
    ) -> Result<(), String> {
        // Get the type (netherportal) that is being used by the app
        let netherportal = get_mut_ref_by_field(
            &mut nether_portal_information.all_nether_portals,
            field_name,
            field_value,
        )?;
        // Get the original copy of the type netherportal that is saved and untouched
        let original_netherportal = get_mut_ref_by_field(
            &mut nether_portal_information.copy_of_nether_portals,
            field_name,
            field_value,
        )?;

        // Re-initialize type (netherportal) being used by the app with the o.g. copy
        // And re-initialize the displayed netherportal because they are different for reasons lol...!
        *netherportal.1 = original_netherportal.1.clone();
        nether_portal_information.displayable_nether_portal = Some((
            netherportal.0,
            StringNetherPortal::convert(netherportal.1.clone())?,
        ));

        Ok(())
    }
    // Reset Button...
    if true_name != "" {
        if ui.button("Reset NetherPortal...?").clicked() {
            if let Err(error_string) = subfn(nether_portal_information, "true_name", true_name) {
                *error_message.lock().unwrap() =
                    ErrorMessage::pure_error_message(Some(error_string))
            }
            ctx.request_repaint();
        }
    }
}

fn get_netherportal_to_save<'a>(
    all_nether_portal_information: &'a HashMap<String, NetherPortal>,
    true_name: &str,
) -> Result<(String, &'a NetherPortal), String> {
    match get_ref_by_field(&all_nether_portal_information, "true_name", true_name) {
        Ok(netherportal) => Ok(netherportal),
        Err(error) => Err(error.to_string()),
    }
}

fn save_netherportal_to_database(netherportal: &NetherPortal, url: String) -> Result<u16, String> {
    // "http://localhost:8123/savenetherportals"
    match ureq::post(&url).send_json(netherportal) {
        Ok(response) => Ok(response.status()),

        Err(error) => Err(error.to_string()),
    }
}

fn reinitialize_copy_of_netherportal(
    updated_netherportal: NetherPortal,
    copy_of_netherportals: &mut HashMap<String, NetherPortal>,
) -> Result<(), String> {
    let netherportal_copy = get_mut_ref_by_field(
        copy_of_netherportals,
        "true_name",
        &updated_netherportal.nether.true_name,
    )?;
    *netherportal_copy.1 = updated_netherportal;

    Ok(())
}

fn save_netherportal(
    nether_portal_information: &mut NewNetherPortalInformation,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
) {
    let mut subfn = || -> Result<(), String> {
        // Get the netherportal to save
        let netherportal = get_netherportal_to_save(
            &nether_portal_information.all_nether_portals,
            &nether_portal_information.modal_information.modal,
        )?
        .1;

        // Send the data to the server for saving and check the status
        // save_nether_portal_url.to_string()
        let status = save_netherportal_to_database(
            netherportal,
            Urls::default(Routes::UpdateNetherPortalText),
        )?;
        if status != 202 {
            return Err(format!("Status is NOT 202: status -> |{}|", status));
        }

        // Update the og copy/backup of HashMap<String, NetherPortals>
        reinitialize_copy_of_netherportal(
            netherportal.clone(),
            &mut nether_portal_information.copy_of_nether_portals,
        )?;
        Ok(())
    };

    // Save Button
    if ui.button("test save").clicked() {
        match subfn() {
            Ok(_) => {
                let message = Some("Sucessfully saved NetherPortal to database!".to_string());
                *window_message.lock().unwrap() = WindowMessage::window_message(message);
            }
            Err(error_string) => {
                *error_message.lock().unwrap() =
                    ErrorMessage::pure_error_message(Some(error_string));
            }
        }
    }
}

fn insert_state_of_images(
    try_access: &Arc<Mutex<HashMap<String, StateOfImages>>>,
    key: String,
    state_of_images: StateOfImages,
) {
    //! Try gain lock from Type Arc<Mutex<HashMap<String, StateOfImages>>>
    loop {
        if let Ok(mut access) = try_access.try_lock() {
            access.insert(key, state_of_images);
            break;
        }
    }
}

fn load_images(
    anpi_am_clone: Arc<Mutex<HashMap<String, StateOfImages>>>,
    error_message: Arc<Mutex<ErrorMessage>>,
    some_modal: String,
    some_state_of_images: Option<ImageCollection>,
) {
    thread::spawn(move || {
        insert_state_of_images(
            &anpi_am_clone,
            some_modal.clone(),
            StateOfImages::BeingBuilt,
        );

        //let state_of_images = match crate::images::image::get_nether_portal_images(&some_modal) {
        match crate::images::image::get_nether_portal_images(&some_modal) {
            Ok(image_collection) => {
                //StateOfImages::HashMap(image_collection)
                if let Some(mut state_of_images) = some_state_of_images {
                    state_of_images.extend(image_collection);

                    insert_state_of_images(
                        &anpi_am_clone,
                        some_modal,
                        StateOfImages::HashMap(state_of_images),
                    );
                } else {
                    insert_state_of_images(
                        &anpi_am_clone,
                        some_modal,
                        StateOfImages::HashMap(image_collection),
                    );
                }
            }
            Err(error_string) => {
                *error_message.lock().unwrap() =
                    ErrorMessage::pure_error_message(Some(error_string));
                insert_state_of_images(&anpi_am_clone, some_modal, StateOfImages::BuildFailed);
            }
        };
    });
}

fn check_and_report_if_image_server_is_down(error_message: &mut Arc<Mutex<ErrorMessage>>) -> bool {
    if !ping("http://localhost:1234/ping") {
        let some_message = Some(format!(
            "The \"Image Server\" could not be reached. Maybe your internet is down...?"
        ));
        *error_message.lock().unwrap() = ErrorMessage::pure_error_message(some_message);
        println!("did this proc more than once");
        return false;
    }
    true
}

fn reload_images(
    all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
    modal: &String,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    if ui.button("Reload Images").clicked() {
        if check_and_report_if_image_server_is_down(error_message) {
            return;
        }
        // do this within a thread??
        if let Ok(mut access) = all_nether_portal_images.try_lock() {
            if let Some(image_list) = access.remove_entry(modal) {
                match image_list.1 {
                    StateOfImages::HashMap(hasher) => {
                        let anpi_am_clone = Arc::clone(&all_nether_portal_images);
                        let error_message_am_clone = Arc::clone(&error_message);
                        let some_modal = modal.clone();

                        load_images(
                            anpi_am_clone,
                            error_message_am_clone,
                            some_modal,
                            Some(hasher),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

fn nether_portal_image_handler(
    all_nether_portal_images: Arc<Mutex<HashMap<String, StateOfImages>>>,
    image_modal: &mut String,
    modal: &mut String,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    if let Ok(access) = all_nether_portal_images.try_lock() {
        if let Some(image_list) = access.get(modal) {
            match image_list {
                StateOfImages::Nothing => {
                    let anpi_am_clone = Arc::clone(&all_nether_portal_images);
                    let error_message_am_clone = Arc::clone(&error_message);
                    let some_modal = modal.clone();
                    if ping("http://localhost:1234/ping") {
                        // if image server is active (if this isnt checked, the whole program could crash lol)
                        println!("did this ever trigger!!!!!!!!!!!!!!");
                        load_images(anpi_am_clone, error_message_am_clone, some_modal, None);
                    } else {
                        // if the server is down at the moment of request then a image_collection still needs to be build and assigned
                        // otherwise the user wont be able to view the images they insert into the program
                        thread::spawn(move || {
                            let image_collection: ImageCollection = HashMap::new();
                            let state_of_images = StateOfImages::HashMap(image_collection);
                            insert_state_of_images(&anpi_am_clone, some_modal, state_of_images);
                        });
                    }
                }
                StateOfImages::HashMap(hasher) => {
                    let image_gear = &make_partial_gear(&hasher);
                    display_nether_portal_images(hasher, image_modal, image_gear, ui);
                }
                _ => {}
            }
        } else {
            if *modal != String::default() {
                let anpi_am_clone = Arc::clone(&all_nether_portal_images);
                let some_modal = modal.clone();
                thread::spawn(move || {
                    insert_state_of_images(&anpi_am_clone, some_modal, StateOfImages::Nothing);
                });
            }
        }
    }
}
#[derive(Serialize)]
struct ImageAndDetailsSerializable {
    #[serde(rename = "Image")]
    image: Vec<u8>,
    image_type: String,
    #[serde(rename = "Image_Details")]
    image_details: ImageDetails,
}

fn send_image_to_image_server(
    url: String,
    filename: &str,
    image: &Vec<u8>,
) -> Result<ureq::Response, String> {
    let url = &format!("{}?name={}", url, filename);

    ureq::post(url).send_bytes(image).or_else(|error| {
        return Err(format!(
            "we got an error from trying save the image...: -> |{}|",
            error.to_string()
        ));
    })
}

fn send_cereal_to_server(
    url: String,
    //image_details: &ImageDetails,
    cereal: &impl serde::Serialize,
) -> Result<ureq::Response, String> {
    let image_details = &to_vec8(cereal);

    ureq::post(&url).send_bytes(image_details).or_else(|error| {
        return Err(format!(
            "We had an error sending the image details to the database for records...: -> |{}|",
            error.to_string()
        ));
    })
}

fn status_check(response: &ureq::Response) -> Result<(), String> {
    //! Handles the response status of each http request according to some random design by me lol
    //! Returns a custom message (wrapped in an "Err Enum") for each incorrect status code
    //! or a an empty unit for "Ok()"
    let pattern = |status| format!("status_code: -> |{}|; ", status);
    let status = response.status();
    match status {
        202 => {
            println!("We successfully status checked");
            Ok(())
        }
        403 => Err(pattern(status)),
        _ => Err(format!(
            "The server has denied/disaproved our request...? status_code: -> |{}|",
            status
        )),
    }
}
fn name_from_response(response: ureq::Response, image_and_details: &mut ImageAndDetails) {
    //! retrives the name from the response and assigns it to image_and_details.image_details.name
    #[derive(Deserialize)]
    struct Name {
        name: String,
    }
    let name: Name = serde_json::from_reader(response.into_reader()).unwrap();

    image_and_details.image_details.name = name.name
}

fn save_image_to_serverx(
    image_and_details: &mut ImageAndDetails,
    save_image_url: String,
    save_image_details_url: String,
) -> Result<(), String> {
    let path = std::path::Path::new(
        image_and_details
            .image_details
            .local_image
            .as_ref()
            .unwrap(),
    );
    let filename = get_file_name(path);
    let image = &path_of_image_to_vec_u8(path).unwrap();

    let response = send_image_to_image_server(save_image_url, filename, image)?;
    status_check(&response)?;

    name_from_response(response, image_and_details);

    let response = send_cereal_to_server(save_image_details_url, &image_and_details.image_details)?;
    status_check(&response)?;

    Ok(())
}

fn save_images(
    all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
    image_key: &String,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
) {
    if ui.button("Save Added Images").clicked() {
        //if !check_and_report_if_image_server_is_down(error_message) {
        //    println!("server down");
        //    return;
        //}

        let anpi_am_clone = Arc::clone(&all_nether_portal_images);
        let error_message_am_clone = Arc::clone(&error_message);
        let window_message_am_clone = Arc::clone(&window_message);

        let some_image_key = image_key.clone();

        thread::spawn(move || {
            if let Ok(mut access) = anpi_am_clone.lock() {
                if let Some(image_list) = access.get_mut(&some_image_key) {
                    match image_list {
                        StateOfImages::HashMap(hasher) => {
                            for (key, image_and_details) in hasher.iter_mut() {
                                if let Some(_) = image_and_details.image_details.local_image {
                                    match save_image_to_serverx(
                                        image_and_details,
                                        Urls::default_i(Routes::SaveImage),
                                        //save_image_url.clone(),
                                        //save_image_details_url.clone(),
                                        Urls::default(Routes::SaveImageText),
                                    ) {
                                        Ok(_) => {
                                            WindowMessage::try_access(
                                                &window_message_am_clone,
                                                |mut access| {
                                                    let message = Some(format!("The Image was successfully saved: image_name: -> |{}|", image_and_details.image_details.name));
                                                    *access =
                                                        WindowMessage::window_message(message);
                                                },
                                            );
                                            // Reset image now that it is technically no longer a "local image" becuase it was saved to db
                                            // Dont want people spamming the "Save Added Images" button...
                                            image_and_details.image_details.local_image = None;
                                        }
                                        Err(err) => {
                                            ErrorMessage::try_access(
                                                &error_message_am_clone,
                                                |mut access| {
                                                    *access = ErrorMessage::pure_error_message(
                                                        Some(err.clone()),
                                                    );
                                                },
                                            );
                                        }
                                    }
                                    println!(
                                        "image is not from here: image_name -> |{}| ## key -> |{}| ## bool_value -> |{:?}|",
                                        image_and_details.image_details.true_name, key, image_and_details.image_details.local_image
                                    );
                                }
                            }
                        }
                        _ => {
                            println!("no hasher?");
                        }
                    }
                }
            }
        });
    }
}

fn actually_delete_images_from_server(
    image_and_details: &ImageAndDetails,
    delete_image_url: String,
    delete_image_from_client_url: String,
) -> Result<(), String> {
    // "http://localhost:1234/deleteimage?
    let url = &format!(
        "{}?name={}",
        delete_image_url, image_and_details.image_details.name
    );
    // TODO: if any of these requests fail, the client and servers cant bounce back from this properly
    // make sure to fix it in the future
    let response = ureq::delete(url)
        .call()
        .or_else(|err| Err(err.to_string()))?;
    status_check(&response)?;

    // "http://localhost:8123/deleteimagefromclient"
    let response = send_cereal_to_server(
        delete_image_from_client_url,
        &image_and_details.image_details,
    )?;
    status_check(&response)?;

    Ok(())
}

fn delete_images_from_server(
    some_deletion_queue: &mut Option<DeletionQueue>,
    nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
    modal: &String,
) -> Result<(), Vec<String>> {
    let mut result: Vec<String> = Vec::new();

    let mut subfn = || -> Result<(), String> {
        fn queue_from_deletion_queue(
            some_deletion_queue: &mut Option<DeletionQueue>,
        ) -> Result<&Vec<Victim>, String> {
            let deletion_queue = if let Some(deletion_queue) = some_deletion_queue {
                deletion_queue
            } else {
                return Err("No deletion_queue in Option<DeletionQueue>".to_string());
            };
            let queue = deletion_queue
                .queue
                .as_ref()
                .ok_or("no queue in deletion_queue".to_string())?;

            Ok(queue)
        }

        let mut hasher = nether_portal_images.lock().unwrap();
        let state_of_images = hasher
            .get_mut(modal)
            .ok_or(format!("failed to get value with key -> |{}|", modal))?;
        let hasher = state_of_images
            .hashmap_mut()
            .ok_or(format!("hashmap_mut failed"))?;

        let queue = queue_from_deletion_queue(some_deletion_queue)?;

        for victim in queue.iter() {
            if victim.staged {
                hasher.get(&victim.name).and_then(|image_and_details| {
                    // delete_image_url.to_string()
                    // delete_image_from_client_url.to_string()
                    if let Err(err) = actually_delete_images_from_server(
                        image_and_details,
                        Urls::default_i(Routes::DeleteImage),
                        Urls::default(Routes::DeleteClientImage),
                    ) {
                        result.push(err);
                    }
                    Some(image_and_details)
                });
            }
        }

        Ok(())
    };

    if let Err(err) = subfn() {
        result.push(err);
        return Err(result);
    }
    if !result.is_empty() {
        return Err(result);
    }
    Ok(())
}

fn delete_images(
    all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
    nether_portal_deletion_queue: &mut Option<DeletionQueue>,
    _deletion_modal: &mut String,
    current_nether_portal_modal: &String,
    ui: &mut egui::Ui,
    ctx: egui::Context,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
) {
    DeletionQueue::handle_deletion_queue(
        all_nether_portal_images,
        nether_portal_deletion_queue,
        current_nether_portal_modal,
        ui,
        ctx,
    );

    if ui.button("Delete Images From Server").clicked() {
        match delete_images_from_server(
            nether_portal_deletion_queue,
            all_nether_portal_images,
            current_nether_portal_modal,
        ) {
            Ok(_) => {
                *window_message.lock().unwrap() = WindowMessage::window_message(Some(
                    "All images sent deleted successfully".to_string(),
                ));
            }
            Err(errors) => {
                let mut message = String::new();
                errors.iter().for_each(|err_string| {
                    message = format!("{}error: -> |{}|\n", message, err_string)
                });
                *error_message.lock().unwrap() = ErrorMessage::pure_error_message(Some(message));
            }
        }
    }

    // 1) iter over deletion list
    // 2) if list.item.staged == true
    // 3) create a queue and delete each picture
    // SERVER SIDE
    // make a route to handle a request with image details
    // take details and say to db DELETE BY THE HAND OF GOD
    // happy delete noises
    // descend into further madness...

    // also after each successfull delete... somehow remove them from the client as well
    // also make a delete button
    // also tell the user of the success or failure...

    // kill
}
pub trait Otherwise {
    // result: &Result<(), String>
    fn otherwise(&self, string: &str) -> Result<(), String>;
}
impl Otherwise for Result<(), String> {
    fn otherwise(&self, string: &str) -> Result<(), String> {
        if let Err(err) = self {
            return Err(format!("{}{}", err, string));
        }
        Ok(())
    }
}

fn add_nether_portal_to_server(
    string_nether_portal: &impl serde::Serialize,
    url: String,
) -> Result<(), String> {
    let response = send_cereal_to_server(
        //"http://localhost:8123/addnetherportal".to_string(),
        url,
        string_nether_portal,
    )?;
    status_check(&response).otherwise("/addnetherportals; Was not successfully added...")?;

    Ok(())
}

fn add_nether_portal(
    //all_nether_portal_information: &mut Arc<Mutex<Option<NewNetherPortalInformation>>>,
    nether_portals: &HashMap<String, NetherPortal>,
    generic_window: &mut GenericWindow,
    building_a_nether_portal: &mut Arc<Mutex<(StringNetherPortal, bool)>>, //Option<StringNetherPortal>,
    _current_nether_portal_modal: &String,
    username: &String,
    ctx: egui::Context,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
) {
    let _ctx1 = ctx.clone();
    let am_clone_bnp = Arc::clone(building_a_nether_portal);
    ui.push_id(101010, |ui| {
        generic_window.is_window_open = generic_window.display_closure(
            &ctx,
            "Add Images",
            Box::new(move |ui, _ctx1| {
                let headers = ["Nether", "OverWorld"];
                let portal_field_names = vec![
                    "xcord",
                    "ycord",
                    "zcord",
                    "locale",
                    "owner",
                    "notes",
                    "true_name",
                ];

                ui.horizontal_wrapped(|ui| {
                    let mut both = am_clone_bnp.lock().unwrap();
                    let string_nether_portal = &mut both.0;
                    headers.iter().for_each(|header| {
                        ui.heading(make_rich(header.to_string(), Some(22.0)));
                        ui.end_row();
                        portal_field_names.iter().for_each(|field| {
                            let string_portal = string_nether_portal.get_mut_ref(header).unwrap();
                            ui.label(format!("{}: ->", field));
                            ui.add(egui::TextEdit::singleline(
                                string_portal.get_mut(field).unwrap(),
                            ));
                            ui.end_row();
                        });
                    });
                    if ui.button("Save Nether Portal").clicked() {
                        both.1 = true;
                    }
                });
            }),
        );

        generic_window.open_window_on_click(ui, "Add Nether Portals");
    });
    let mut both = building_a_nether_portal.lock().unwrap();

    if both.1 {
        both.0.username = username.to_string();
        both.0.id = 0.to_string();
        let netherportal = match NetherPortal::convert(&both.0) {
            Ok(np) => np,
            Err(err) => {
                *error_message.lock().unwrap() =
                    ErrorMessage::pure_error_message(Some(err + " special"));
                both.1 = false;
                return;
            }
        };
        // add_nether_portal_url.to_string()
        match add_nether_portal_to_server(&netherportal, Urls::default(Routes::AddNetherPortalText))
        {
            Ok(_) => {
                *window_message.lock().unwrap() = WindowMessage::window_message(Some(
                    "Successfully saved netherportal".to_string(),
                ));
            }
            Err(err) => {
                // i was supposed to do something here i think, but i totally forgot what i needed this information for
                // maybe it was in a differnt area and has since be resolved
                // but if something with saving nps goes wrong.. at least i know where to look lol
                for (key, _) in nether_portals {
                    println!("the KEy IS: -> |{}|", key);
                    //anpi.all_nether_portals.insert(k, v)
                    // xjump xla
                }
                *error_message.lock().unwrap() = ErrorMessage::pure_error_message(Some(err))
            }
        };
        both.1 = false
    }
}

fn delete_nether_portal(
    _nether_portals: &mut HashMap<String, NetherPortal>,
    nether_portal_list: &Vec<String>,
    nether_portal_hashlist: &mut HashMap<String, bool>,
    generic_window: &mut GenericWindow,
    ctx: egui::Context,
    ui: &mut egui::Ui,
    _error_message: &mut Arc<Mutex<ErrorMessage>>,
    _window_message: &mut Arc<Mutex<WindowMessage>>,
) {
    // some jump
    generic_window.is_window_open =
        generic_window.display_closure_x(&ctx, "Delete Nether Portals", |ui| {
            ui.horizontal_wrapped(|ui| {
                nether_portal_hashlist
                    .iter_mut()
                    .for_each(|(key, element)| {
                        ui.label(format!("{}: ->", key));
                        if ui.button(element.to_string()).clicked() {
                            *element = !*element;
                        }
                        ui.end_row();
                    })
            });
        });
    // update on window open
    if generic_window.is_window_open {
        nether_portal_list.iter().for_each(|element| {
            if !nether_portal_hashlist.contains_key(element) {
                nether_portal_hashlist.insert(element.to_string(), false);
            }
        });
    }
    generic_window.open_window_on_click(ui, "Delete Nether Portals");
}

fn ping(server_url: &str) -> bool {
    let result = ureq::get(server_url).call();
    match result {
        Ok(_response) => {
            return true;
        }
        Err(_error) => return false,
    };
}

// get_nether_portal_images_url,
// get_nether_portal_image_names_url,

// Big Boi Function
pub fn new_nether_portal(
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
    all_nether_portal_information: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
    session_information: &Arc<Mutex<SessionInformation>>,
    user_picked_filepath: &mut Option<String>,
    username: &String,
    ui: &mut egui::Ui,
    ctx_clone: egui::Context,
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
            let copy = nether_portals.clone();
            *all_nether_portal_information_am_clone.lock().unwrap() =
                Some(NewNetherPortalInformation {
                    modal_information: some_modal,
                    all_nether_portals: nether_portals,
                    all_nether_portal_images: Arc::new(Mutex::new(HashMap::new())),
                    displayable_nether_portal: None,
                    copy_of_nether_portals: copy,
                    nether_portal_deletion_wheel: None,
                    image_deletion_window: GenericWindow::default(),
                    building_a_nether_portal: Arc::new(Mutex::new((
                        StringNetherPortal::default(),
                        false,
                    ))),
                    actual_nether_portal_deletion: HashMap::default(),
                    actual_nether_portal_generic_window: GenericWindow::default(),
                    modify: bool::default(),
                });
        });
    });

    // use try_nether_portal_information to gain access to its value
    NewNetherPortalInformation::try_nether_portal_information(
        all_nether_portal_information,
        ui,
        ctx_clone.clone(),
        |some_nether_portal_information, ui, ctx| {
            //let access_am_clone = Arc::clone(&all_nether_portal_information.lock().unwrap().as_ref().unwrap().all_nether_portal_images);
            ui.horizontal(|ui| {
                // create a modal machine with its information
                modal_machine_for_nether_portals(some_nether_portal_information, ui, error_message);
                add_nether_portal(
                    &mut some_nether_portal_information.all_nether_portals,
                    &mut some_nether_portal_information.image_deletion_window,
                    &mut some_nether_portal_information.building_a_nether_portal,
                    &some_nether_portal_information.modal_information.modal,
                    username,
                    ctx.clone(),
                    ui,
                    error_message,
                    window_message,
                );
                // delete_nether_portal();
                // allow user to modify the current chosen netherportal being displayed
                modify_button(
                    &mut some_nether_portal_information.modify,
                    &mut some_nether_portal_information.all_nether_portals,
                    &mut some_nether_portal_information.displayable_nether_portal,
                    &some_nether_portal_information.modal_information.modal,
                    session_information,
                    ui,
                    error_message,
                );
                reset_netherportal_button(
                    some_nether_portal_information,
                    &some_nether_portal_information
                        .modal_information
                        .modal
                        .to_string(),
                    ui,
                    ctx_clone.clone(),
                    error_message,
                );
                save_netherportal(
                    some_nether_portal_information,
                    ui,
                    error_message,
                    window_message,
                );
                save_images(
                    &mut some_nether_portal_information.all_nether_portal_images,
                    &some_nether_portal_information.modal_information.modal,
                    ui,
                    error_message,
                    window_message,
                );
                delete_images(
                    &mut some_nether_portal_information.all_nether_portal_images,
                    &mut some_nether_portal_information.nether_portal_deletion_wheel,
                    &mut some_nether_portal_information
                        .modal_information
                        .deletion_modal,
                    &some_nether_portal_information.modal_information.modal,
                    ui,
                    ctx.clone(),
                    error_message,
                    window_message,
                );
                delete_nether_portal(
                    &mut some_nether_portal_information.all_nether_portals,
                    &some_nether_portal_information.modal_information.modal_list,
                    &mut some_nether_portal_information.actual_nether_portal_deletion,
                    &mut some_nether_portal_information.actual_nether_portal_generic_window,
                    ctx.clone(),
                    ui,
                    error_message,
                    window_message,
                );
                add_files(
                    user_picked_filepath,
                    &mut some_nether_portal_information.all_nether_portal_images,
                    &some_nether_portal_information.modal_information.modal,
                    username,
                    ui,
                    window_message,
                    error_message,
                );
                reload_images(
                    &mut some_nether_portal_information.all_nether_portal_images,
                    &some_nether_portal_information.modal_information.modal,
                    ui,
                    error_message,
                );
                ui.end_row();
            });
            ui.horizontal(|ui| {
                // display the information inside the datastructure to the ui
                displayable_nether_portal(ui, error_message, some_nether_portal_information);

                // if you can access client images, show some images...
                nether_portal_image_handler(
                    Arc::clone(&some_nether_portal_information.all_nether_portal_images),
                    &mut some_nether_portal_information.modal_information.image_modal,
                    &mut some_nether_portal_information.modal_information.modal,
                    ui,
                    error_message,
                );
            });
        },
    );
}
