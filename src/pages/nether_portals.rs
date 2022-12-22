use crate::{
    borkcraft_app::ErrorMessage, //{modal_machine, BorkCraft, ErrorMessage},
    borkcraft_app::WindowMessage,
    eframe_tools::modal_machines::{self, act_on_tooth},
    images::image::{display_nether_portal_images, make_partial_gear, ImageAndDetails},
    thread_tools::ThreadPool,
};
use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        {Arc, Mutex, Once},
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

#[derive(Default)]
pub struct NewNetherPortalInformation {
    pub modal_information: NewNetherPortalModal,
    pub all_nether_portals: HashMap<String, NetherPortal>,
    pub all_nether_portal_images: Arc<Mutex<HashMap<String, StateOfImages>>>, //HashMap<String, ImageAndDetails>>>>,
    pub displayable_nether_portal: Option<(String, StringNetherPortal)>,
    pub copy_of_nether_portals: HashMap<String, NetherPortal>,
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

#[derive(Clone, Debug)]
pub struct StringNetherPortal {
    pub id: String,
    pub overworld: StringPortal,
    pub nether: StringPortal,
    pub username: String,
}
impl StringNetherPortal {
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

    pub fn convert(string_nether_portal: StringNetherPortal) -> Result<NetherPortal, String> {
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

#[derive(Debug, Default, Clone)]
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
    // The String in HashMap<String, NetherPortal> is the index primary key from the database
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
        previous_modal: String::default(),
        modal_list: Vec::new(),
        image_modal: String::default(),
    };
    // Make a list of names based on (true_name) to be used as a selector for the user modal
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
                    match NetherPortal::convert(some_tuple.1.clone()) {
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

    fn make_rich(ss: String, size: Option<f32>) -> egui::widget_text::RichText {
        let some_size = match size {
            Some(some_size) => some_size,
            None => 25.0,
        };
        egui::RichText::new(ss).font(egui::FontId::proportional(some_size))
    }
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
                match NetherPortal::convert(some_tuple.1.clone()) {
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

fn modify_button(
    modify: &mut bool,
    all_nether_portals: &mut HashMap<String, NetherPortal>,
    displayable_nether_portal: &mut Option<(String, StringNetherPortal)>,
    current_netherportal_modal: &String,
    ui: &mut egui::Ui,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    if *current_netherportal_modal != String::default() {
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

fn save_netherportal_to_database(netherportal: &NetherPortal) -> Result<u16, String> {
    match ureq::post("http://localhost:8123/savenetherportals").send_json(netherportal) {
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
        let status = save_netherportal_to_database(netherportal)?;
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

                    thread::spawn(move || {
                        insert_state_of_images(
                            &anpi_am_clone,
                            some_modal.clone(),
                            StateOfImages::BeingBuilt,
                        );

                        let state_of_images =
                            match crate::images::image::get_nether_portal_images(&some_modal) {
                                Ok(image_collection) => StateOfImages::HashMap(image_collection),
                                Err(error_string) => {
                                    *error_message_am_clone.lock().unwrap() =
                                        ErrorMessage::pure_error_message(Some(error_string));
                                    StateOfImages::BuildFailed
                                }
                            };

                        insert_state_of_images(&anpi_am_clone, some_modal, state_of_images);
                    });
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

pub fn new_nether_portal(
    error_message: &mut Arc<Mutex<ErrorMessage>>,
    window_message: &mut Arc<Mutex<WindowMessage>>,
    all_nether_portal_information: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
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
                    modify: bool::default(),
                });
        });
    });

    // use try_nether_portal_information to gain access to its value
    NewNetherPortalInformation::try_nether_portal_information(
        all_nether_portal_information,
        ui,
        |some_nether_portal_information, ui| {
            //let access_am_clone = Arc::clone(&all_nether_portal_information.lock().unwrap().as_ref().unwrap().all_nether_portal_images);
            ui.horizontal(|ui| {
                // create a modal machine with its information
                modal_machine_for_nether_portals(some_nether_portal_information, ui, error_message);
                // allow user to modify the current chosen netherportal being displayed
                modify_button(
                    &mut some_nether_portal_information.modify,
                    &mut some_nether_portal_information.all_nether_portals,
                    &mut some_nether_portal_information.displayable_nether_portal,
                    &some_nether_portal_information.modal_information.modal,
                    ui,
                    error_message,
                );
                // display the information inside the datastructure to the ui
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
                ui.end_row();
            });
            ui.horizontal(|ui| {
                displayable_nether_portal(ui, error_message, some_nether_portal_information);

                // if you can access client images
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
