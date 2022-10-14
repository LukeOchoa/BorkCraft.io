use eframe::egui::Ui;
use egui_extras::RetainedImage;
use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    borkcraft_app::{modal_machine, ErrorMessage},
    eframe_tools::modal_machines::{self, act_on_tooth, ModalMachineGear},
    thread_tools::ThreadPool,
};
pub struct NetherPortalImages {
    all_netherportal_images: Arc<Mutex<HashMap<String, HashMap<String, ImageAndDetails>>>>,
}

#[derive(Deserialize)]
pub struct ImageDetails {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "True_name")]
    true_name: String,
    #[serde(rename = "Username")]
    username: String,
}
// Big Boi Data is: HashMap<String, ImageAndDetails>
pub struct ImageAndDetails {
    image: RetainedImage,
    image_details: ImageDetails,
}

fn response_to_retained_image(response: ureq::Response) -> Result<RetainedImage, String> {
    let mut bytes: Vec<u8> = Vec::new();
    response.into_reader().read_to_end(&mut bytes).unwrap();
    let image = egui_extras::image::RetainedImage::from_image_bytes(
        "http://localhost:8123/getnetherportalimages",
        &bytes,
    )
    .unwrap();

    Ok(image)
}

fn fetch_image_response(name: &String) -> Result<ureq::Response, String> {
    match ureq::get(&format!(
        "http://localhost:1234/getnetherportalimage?name={}",
        name
    ))
    .call()
    {
        Ok(response) => return Ok(response),
        Err(error) => return Err(error.to_string()),
    }
}

fn get_images_from_server(
    image_details_hm: HashMap<String, ImageDetails>,
) -> Result<HashMap<String, ImageAndDetails>, String> {
    let mut hashy: HashMap<String, ImageAndDetails> = HashMap::new();
    for (_key, image_details) in image_details_hm {
        let response = fetch_image_response(&image_details.name)?;
        let image = response_to_retained_image(response)?;
        hashy.insert(
            image_details.name.clone(),
            ImageAndDetails {
                image,
                image_details,
            },
        );
    }
    Ok(hashy)
}

fn get_image_from_server(tx: Sender<Result<ImageAndDetails, String>>, image_details: ImageDetails) {
    let subfn = || -> Result<ImageAndDetails, String> {
        let response = fetch_image_response(&image_details.name)?;
        let image = response_to_retained_image(response)?;
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
) -> Result<ureq::Response, String> {
    match ureq::get(&format!(
        "http://localhost:8123/getnetherportalimagenames?true_name={}",
        true_name
    ))
    .call()
    {
        Ok(response) => Ok(response),
        Err(error) => Err(error.to_string()),
    }
}

pub type ImageCollection = HashMap<String, ImageAndDetails>;

pub fn get_nether_portal_images(true_name: &String) -> Result<ImageCollection, String> {
    let response = ask_server_for_image_list_for_netherportals(true_name)?;
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
            get_image_from_server(tx, image_details);
        });
    }

    let mut image_collection: ImageCollection = HashMap::new();
    let mut subfn = || -> Result<(), String> {
        let image_and_details = receiver.recv().or_else(|error| Err(error.to_string()))??;
        image_collection.insert(
            image_and_details.image_details.name.clone(),
            image_and_details,
        );
        Ok(())
    };
    for _ in 0..length {
        subfn()?;
    }

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
    error_message: &mut ErrorMessage,
) {
    let tooth = modal_machines::modal_machine(
        image_modal,
        ui,
        ModalMachineGear::Immutable(&image_gear),
        Some("Images"),
        11,
    );
    act_on_tooth(tooth, |some_option| {
        let retained_image = &image_list.get(some_option).unwrap().image;
        display_retained_image(&retained_image, ui)
    });
}

// get a list of images that is: HashMap<String, ImageAndDetails>
// create a modal with the names of the image
// on choice display an image
// make a function that takes an Image to display
// make a set of buttons that will let u scroll through the images
