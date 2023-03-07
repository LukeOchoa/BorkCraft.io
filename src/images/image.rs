use egui_extras::RetainedImage;
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    eframe_tools::modal_machines::{self, ModalMachineGear},
    thread_tools::ThreadPool,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageDetails {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "True_name")]
    pub true_name: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub local_image: Option<String>, // if local_image is "True" then the image was added by the user in the current "program session"(not client/server session)
                                     // maybe at some point if the user request, then this will be used to determin if this data in the struct should be saved to a server
                                     // i need a path if its a local image.
                                     // i could create a path variable or i could simply use local_image as an Option<String> aka Option<Path>
                                     // and when my code looks for a bool value this could be the replacement...
}
// Big Boi Data is: HashMap<String, ImageAndDetails>
pub struct ImageAndDetails {
    pub image: RetainedImage,
    pub image_details: ImageDetails,
}

// SELECT true_name, count(true_name) FROM netherportal_images WHERE username='World Spawn' GROUP BY true_name;

fn response_to_retained_image(
    response: ureq::Response,
    url: String,
) -> Result<RetainedImage, String> {
    // "http://localhost:8123/getnetherportalimages",
    let mut bytes: Vec<u8> = Vec::new();
    response.into_reader().read_to_end(&mut bytes).unwrap();
    let image = egui_extras::image::RetainedImage::from_image_bytes(url, &bytes).unwrap();

    Ok(image)
}

fn fetch_image_response(name: &String, url: String) -> Result<ureq::Response, String> {
    // "http://localhost:1234/getnetherportalimage?name={}",
    match ureq::get(&format!("{}?name={}", url, name)).call() {
        Ok(response) => return Ok(response),
        Err(error) => return Err(error.to_string()),
    }
}

//fn _get_images_from_server(
//    image_details_hm: HashMap<String, ImageDetails>,
//) -> Result<HashMap<String, ImageAndDetails>, String> {
//    let mut hashy: HashMap<String, ImageAndDetails> = HashMap::new();
//    for (_key, image_details) in image_details_hm {
//        let response = fetch_image_response(&image_details.name)?;
//        let image = response_to_retained_image(response)?;
//        hashy.insert(
//            image_details.name.clone(),
//            ImageAndDetails {
//                image,
//                image_details,
//            },
//        );
//    }
//    Ok(hashy)
//}

fn get_image_from_server(
    tx: Sender<Result<ImageAndDetails, String>>,
    image_details: ImageDetails,
    get_nether_portal_images_url: String,
) {
    let subfn = || -> Result<ImageAndDetails, String> {
        let response = fetch_image_response(
            &image_details.name,
            get_nether_portal_images_url.to_string(),
        )?;
        println!(
            "we found before the error\n\n {}|{}",
            image_details.name, image_details.true_name
        );
        let image = response_to_retained_image(response, get_nether_portal_images_url.to_string())?;
        let image_and_details = ImageAndDetails {
            image,
            image_details,
        };

        Ok(image_and_details)
    };
    tx.send(subfn()).unwrap();
}

fn response_to_image_details(response: ureq::Response) -> HashMap<String, ImageDetails> {
    let json_string = response.into_string().unwrap();
    // The key is essentially meaningless. Its just easy to deserialize into a hashmap as far as i know...
    let some_image_details: HashMap<String, ImageDetails> =
        serde_json::from_str(&json_string).unwrap();

    some_image_details
}

fn ask_server_for_image_list_for_netherportals(
    true_name: &String,
    url: String,
) -> Result<ureq::Response, String> {
    // "http://localhost:8123/getnetherportalimagenames?true_name={}",
    println!(
        "DISGASSSTIN: {}",
        format!("{}?true_name={}", url, true_name),
    );
    match ureq::get(&format!("{}?true_name={}", url, true_name)).call() {
        Ok(response) => Ok(response),
        Err(error) => Err(error.to_string()),
    }
}

pub type ImageCollection = HashMap<String, ImageAndDetails>;

pub fn get_nether_portal_images(
    true_name: &String,
    get_nether_portal_images_url: &'static str,
    get_npin_url: &'static str,
) -> Result<ImageCollection, String> {
    let response =
        ask_server_for_image_list_for_netherportals(true_name, get_npin_url.to_string())?;
    let list = response_to_image_details(response);
    let length = list.len();
    let pool = ThreadPool::new(length);

    let (sender, receiver): (
        Sender<Result<ImageAndDetails, String>>,
        Receiver<Result<ImageAndDetails, String>>,
    ) = mpsc::channel();
    for (_, image_details) in list {
        let tx = sender.clone();
        pool.execute(|| {
            println!("NAME OF THE IMAGE TO GET? -> |{}|", image_details.name);
            get_image_from_server(tx, image_details, get_nether_portal_images_url.to_string());
        });
    }

    let mut image_collection: ImageCollection = HashMap::new();
    let mut subfn = || -> Result<(), String> {
        let image_and_details = receiver.recv().or_else(|error| Err(error.to_string()))??;
        println!("revc!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!11");
        image_collection.insert(
            image_and_details.image_details.name.clone(),
            image_and_details,
        );
        Ok(())
    };
    for _ in 0..length {
        if let Err(error) = subfn() {
            panic!(
                "The length is: -> |{}|  ###  what you tried to get by name: name: -> ||    ###  98798797987987\n\nh{}",
                length,
                error.to_string()
            );
        }
    }
    println!("RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR");
    Ok(image_collection)
}

fn display_retained_image(retained_image: &RetainedImage, ui: &mut eframe::egui::Ui) {
    let mut size = retained_image.size_vec2();
    size *= (ui.available_width() / size.x).min(1.0);
    retained_image.show_size(ui, size);
}

pub fn make_partial_gear(list: &HashMap<String, ImageAndDetails>) -> Vec<String> {
    let mut gear: Vec<String> = Vec::new();
    for (key, _) in list {
        gear.push(key.clone());
    }
    gear
}

pub fn display_nether_portal_images(
    image_list: &HashMap<String, ImageAndDetails>,
    image_modal: &mut String,
    image_gear: &Vec<String>,
    ui: &mut eframe::egui::Ui,
) {
    // create modal for image choice
    modal_machines::modal_machine(
        image_modal,
        ui,
        ModalMachineGear::Immutable(&image_gear),
        Some("Images"),
        11,
    );

    // get image if it exists with the chosen modal from app state
    if let Some(image_and_details) = image_list.get(image_modal) {
        let retained_image = &image_and_details.image;
        display_retained_image(&retained_image, ui)
    }
}
