use eframe::egui;
use egui_extras::RetainedImage;

use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    borkcraft_app::ErrorMessage,
    images::image::{ImageAndDetails, ImageDetails},
    windows::client_windows::WindowMessage,
    StateOfImages,
};

//fn path_to_retained_image(path: String) -> Result<RetainedImage, String> {}

//fn response_to_retained_image(response: ureq::Response) -> Result<RetainedImage, String> {
//    let mut bytes: Vec<u8> = Vec::new();
//    response.into_reader().read_to_end(&mut bytes).unwrap();
//    let image = egui_extras::image::RetainedImage::from_image_bytes(
//        "http://localhost:8123/getnetherportalimages",
//        &bytes,
//    )
//    .unwrap();
//
//    Ok(image)
//}

pub fn add_files(
    //the_self: &mut BorkCraft,
    user_picked_filepath: &mut Option<String>,
    all_netherportal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
    true_name: &String,
    username: &String,
    ui: &mut egui::Ui,
    window_message: &mut Arc<Mutex<WindowMessage>>,
    error_message: &mut Arc<Mutex<ErrorMessage>>,
) {
    if ui.button("Open file...").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            //the_self.user_picked_filepath = Some(path.display().to_string());
            *user_picked_filepath = Some(path.display().to_string());
        }

        if let Some(picked_path) = &user_picked_filepath {
            *window_message.lock().unwrap() = WindowMessage::window_message(Some(format!(
                "You selected file: {}",
                picked_path.clone()
            )));
        }

        //let _x = &*all_netherportal_images
        //    .lock()
        //    .unwrap()
        //    .get(true_name)
        //    .unwrap();
        fn get_file_type(path: &Path) -> &str {
            let file_extension = path.extension().and_then(std::ffi::OsStr::to_str).unwrap();

            file_extension
        }
        fn get_file_name(path: &Path) -> &str {
            let filename = path.file_name().and_then(std::ffi::OsStr::to_str).unwrap();

            filename
        }
        fn turn_path_into_image(path: &Path) -> Result<RetainedImage, String> {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(_) => {
                    return Err(format!(
                        "The file: |{}| could not be opened...",
                        get_file_name(path)
                    ))
                }
            };

            let mut reader = std::io::BufReader::new(file);
            let mut buffer = Vec::new();
            match reader.read_to_end(&mut buffer) {
                Ok(_) => {}
                Err(_) => {
                    return Err(format!(
                        "Failed to read file: |{}| all the way to the end of file...",
                        get_file_name(path)
                    ))
                }
            }
            let image = egui_extras::image::RetainedImage::from_image_bytes("your mom", &buffer);

            image
        }

        if let Some(almost_path) = user_picked_filepath {
            let path = std::path::Path::new(almost_path);
            let file_extension = get_file_type(path);
            let filename = get_file_name(path);

            let acceptable_file_types = vec!["png", "jpeg", "jpg"];

            if acceptable_file_types.contains(&file_extension) {
                println!(
                    "The path name is... -> |{}| new -> |{}|",
                    almost_path, filename
                );

                match &mut *all_netherportal_images
                    .lock()
                    .unwrap()
                    .get_mut(true_name)
                    .unwrap()
                {
                    StateOfImages::HashMap(hasher) => {
                        // all filenames must be unique
                        let image = match turn_path_into_image(path) {
                            Ok(image) => image,
                            Err(error_string) => {
                                *error_message.lock().unwrap() =
                                    ErrorMessage::pure_error_message(Some(error_string));
                                return;
                            }
                        };
                        let image_details = ImageDetails {
                            id: -1,
                            name: get_file_name(path).to_string(,
                            true_name: true_name.clone(), // filename.to_string()
                            username: username.to_string(),
                            local_image: Some(almost_path.to_string()),
                        };
                        let image_and_details = ImageAndDetails {
                            image,
                            image_details,
                        };
                        if !hasher.contains_key(almost_path) {
                            hasher.insert(almost_path.to_string(), image_and_details);
                            let message = Some(format!("The file: |{}| at path: |{}| was successfully added to the program...!", filename, almost_path));
                            *window_message.lock().unwrap() =
                                WindowMessage::window_message(message);
                            for (key, _) in hasher {
                                println!("hasher check: key -> |{}|", key);
                            }
                        } else {
                            let message = Some(format!("The file: |{}| at path: |{}| is already present in this program...? Perhaps try renaming the file, there is a naming clash...", filename, almost_path));
                            *error_message.lock().unwrap() =
                                ErrorMessage::pure_error_message(message);
                        }

                        //match hasher.try_insert(almost_path.to_string(), image_and_details) {
                        //    Ok(_) => {
                        //        let message = Some(format!("The file: |{}| at path: |{}| was successfully added to the program...!", filename, almost_path));
                        //        *window_message.lock().unwrap() =
                        //            WindowMessage::window_message(message);
                        //    }
                        //    Err(_) => {
                        //        let message = Some(format!("The file: |{}| at path: |{}| is already present in this program...? Perhaps try renaming the file, there is a naming clash...", filename, almost_path));
                        //        *error_message.lock().unwrap() =
                        //            ErrorMessage::pure_error_message(message);
                        //    }
                        //}
                    }
                    _ => {}
                }
                //let file = std::fs::File::open(path).unwrap();
                //let mut reader = std::io::BufReader::new(file);
                //let mut buffer = Vec::new();

                //reader.read_to_end(&mut buffer).unwrap();
                //let image = egui_extras::image::RetainedImage::from_image_bytes("your mom", &buffer);
            }
            //println!("The file-type is... -> |{}|", file_extension);
            //match &*all_netherportal_images
            //    .lock()
            //    .unwrap()
            //    .get(true_name)
            //    .unwrap()
            //{
            //    StateOfImages::HashMap(hasher) => {
            //        for (key, _) in hasher {
            //            println!("hasher -> Key: {} || Value: BLANK", key);
            //        }
            //    }
            //    _ => {}
            //}
        }
    }

    //for (key, _value) in &*all_netherportal_images.lock().unwrap() {
    //    println!("key: {} || value: BLANK", key);
    //}
}

// files have to be submitted

// files have to be re-added/added to the client's register?

// you have to check if the user has < 10 files in order to add files

// when a file is added and submitted, it needs to be accounted for on the database and given a unique name
