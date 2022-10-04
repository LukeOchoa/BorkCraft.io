mod borkcraft;
mod errors;
mod login;
mod pages;

pub use borkcraft::*;
use borkcraft_app::SessionInformation;
use errors::client_errors::ErrorMessage;
pub use pages::nether_portals::*;

use serde::Serialize;
pub fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
    serde_json::to_vec(cereal).unwrap()
}
pub enum ResponseResult {
    Text(String),
    SessionInformation(SessionInformation),
    NetherPortal(NetherPortal),
}
pub fn ureq_did_request_go_through_f(
    did_request_go_through: Result<ureq::Response, ureq::Error>,
    job: Box<dyn Fn(ureq::Response) -> Result<ResponseResult, String>>,
) -> Result<ResponseResult, ErrorMessage> {
    let failed_to_convert_response =
        "Failed to convert response to a variant of type Enum ResponseResult...: error ==";
    let bad_server_response_error = "network was sent but denied by the server... Maybe wrong key was given? [retrieving nether portal keys]: error ==";
    let _no_connection_error =
        "network request was not able to connect... [retrieving nether portal keys2]: error ==";

    match did_request_go_through {
        Ok(response) => match response.status() {
            200..=299 => match job(response) {
                Ok(result) => return Ok(result),
                Err(error) => {
                    return Err(ErrorMessage::pure_error_message(Some(format!(
                        "{}{}",
                        failed_to_convert_response, error
                    ))))
                }
            },
            _ => {
                let error_string =
                    format!("{}{}", bad_server_response_error, response.status_text());
                return Err(ErrorMessage::pure_error_message(Some(error_string)));
            }
        },
        Err(error) => {
            println!("Proc: {}", error.kind());
            let error_string = format!("{}: {}", error.kind(), error.to_string());
            return Err(ErrorMessage::pure_error_message(Some(error_string)));
        }
    }
}
pub fn match_out_nether_portal_keys_to_string2(
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
mod eframe_tools {

    pub mod modal_machines {
        use crate::NewNetherPortalInformation;
        use std::sync::{Arc, Mutex};

        type Tooth = Option<String>;

        pub enum ModalMachineGear<'a> {
            Constant(&'static Vec<String>),
            Immutable(&'a Vec<String>),
            Mutable(&'a mut Vec<String>),
        }
        pub fn modal_machine(
            selected_modal: &mut String,
            ui: &mut eframe::egui::Ui,
            //const_page_options: &'static Vec<String>,
            gear: ModalMachineGear,
            ui_id: i32,
        ) -> Tooth {
            let mut some_tooth: Tooth = None;
            ui.push_id(ui_id, |ui| {
                eframe::egui::ComboBox::from_label("Choose a Modal...!")
                    .selected_text(selected_modal.clone())
                    .show_ui(ui, |ui| {
                        let mut wheel = |some_gear: &Vec<String>| {
                            for tooth in some_gear {
                                if ui
                                    .selectable_value(selected_modal, tooth.to_string(), tooth)
                                    .clicked()
                                {
                                    some_tooth = Some(tooth.to_string())
                                }
                            }
                        };
                        match gear {
                            ModalMachineGear::Constant(some_constant_gear) => {
                                wheel(some_constant_gear);
                            }
                            ModalMachineGear::Immutable(some_immutable_gear) => {
                                wheel(some_immutable_gear);
                            }
                            ModalMachineGear::Mutable(some_mutable_gear) => {
                                wheel(some_mutable_gear);
                            }
                        }
                    });
            });

            some_tooth
        }

        pub fn act_on_tooth(some_tooth: Tooth, mut action: impl FnMut(&str)) {
            if let Some(tooth) = some_tooth {
                action(&tooth);
            }
        }

        pub fn try_modal_machine(
            nether_portal_information_am: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
            mut action: impl FnMut(Tooth),
            ui: &mut eframe::egui::Ui,
        ) {
            //! A simple function to abstract this ugly logic for creating a modal_machine
            //! from the information inside (struct NewNetherPortalInformation) that is
            //! trapped inside an (Arc Mutex)
            match nether_portal_information_am.try_lock() {
                Ok(mut guarded_option) => match &mut *guarded_option {
                    Some(nether_portal_information) => {
                        let tooth = modal_machine(
                            &mut nether_portal_information.modal_information.modal,
                            ui,
                            ModalMachineGear::Immutable(
                                &nether_portal_information.modal_information.modal_list,
                            ),
                            88,
                        );
                        action(tooth)
                        //act_on_tooth(tooth, |some_option| action(some_option));
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
        pub fn try_nether_portal_information(
            nether_portal_information_am: &Arc<Mutex<Option<NewNetherPortalInformation>>>,
            mut action: impl FnMut(&mut NewNetherPortalInformation),
            ui: &mut eframe::egui::Ui,
        ) {
            match nether_portal_information_am.try_lock() {
                Ok(mut guarded_option) => match &mut *guarded_option {
                    Some(nether_portal_information) => {
                        //let tooth = modal_machine(
                        //    &mut nether_portal_information.modal_information.modal,
                        //    ui,
                        //    ModalMachineGear::Immutable(
                        //        &nether_portal_information.modal_information.modal_list,
                        //    ),
                        //    88,
                        //);
                        //action(tooth)
                        //act_on_tooth(tooth, |some_option| action(some_option));
                        action(nether_portal_information);
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
}

pub mod thread_tools {
    use std::{
        sync::{mpsc, Arc, Mutex},
        thread,
    };
    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: Option<mpsc::Sender<Job>>,
    }

    type Job = Box<dyn FnOnce() + Send + 'static>;

    impl ThreadPool {
        /// Create a new ThreadPool.
        ///
        /// The size is the number of threads in the pool.
        ///
        /// # Panics
        ///
        /// The `new` function will panic if the size is zero.
        pub fn new(size: usize) -> ThreadPool {
            assert!(size > 0);

            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool {
                workers,
                sender: Some(sender),
            }
        }

        pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);

            self.sender.as_ref().unwrap().send(job).unwrap();
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            drop(self.sender.take());

            for worker in &mut self.workers {
                println!("Shutting down worker {}", worker.id);

                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
    struct Worker {
        id: usize,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
        fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");

                        job();
                        println!("job {id} finished executing?");
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            });

            Worker {
                id,
                thread: Some(thread),
            }
        }
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn
// }
