use crate::StateOfImages;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(Default)]
pub struct GenericWindow {
    //pub f: Option<Box<dyn FnMut(&mut eframe::egui::Ui, eframe::egui::Context) + Send>>,
    pub is_window_open: bool,
    pub try_to_open_window: bool,
}

impl GenericWindow {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }

    pub fn generic_window(//f: Option<Box<dyn FnMut(&mut eframe::egui::Ui, eframe::egui::Context) + Send>>,
    ) -> GenericWindow {
        GenericWindow {
            //f,
            is_window_open: true,
            try_to_open_window: false,
        }
    }

    pub fn open_window_on_click(&mut self, ui: &mut eframe::egui::Ui, name: &str) {
        eframe::egui::Grid::new(21).show(ui, |ui| {
            if ui.button(name).clicked() {
                self.is_window_open = !self.is_window_open
            }
            if self.try_to_open_window {
                self.is_window_open = false;
                self.try_to_open_window = false;
            }
            ui.end_row();
        });
    }

    pub fn display_closure(
        &mut self,
        ctx: &eframe::egui::Context,
        name: &str,
        //f: Box<dyn FnMut(&mut eframe::egui::Ui, eframe::egui::Context) + Send>,
        mut f: Box<dyn FnMut(&mut eframe::egui::Ui, eframe::egui::Context)>,
    ) -> bool {
        let mut is_window_shut: bool = self.is_window_open;
        eframe::egui::Window::new(name)
            .open(&mut is_window_shut)
            .show(ctx, |ui| f(ui, ctx.clone()));
        is_window_shut
    }
}

#[derive(Default)]
pub struct WindowMessage {
    pub message: Option<String>,
    pub is_window_open: bool,
    pub try_to_open_window: bool,
}
impl WindowMessage {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn window_message(message: Option<String>) -> WindowMessage {
        WindowMessage {
            message,
            is_window_open: true,
            try_to_open_window: false,
        }
    }
    pub fn open_window_on_click(&mut self, ui: &mut eframe::egui::Ui, name: &str) {
        eframe::egui::Grid::new(5).show(ui, |ui| {
            if ui.button(name).clicked() {
                self.try_to_open_window = true;
                println!("Some click: |{:?}|", self.message);
            }
            if self.try_to_open_window {
                if let None = self.message {
                    ui.label("There are no messages...!");
                } else {
                    self.is_window_open = true;
                    self.try_to_open_window = false;
                }
            }
            ui.end_row();
        });
    }
    pub fn display_message(&self, ctx: &eframe::egui::Context) -> bool {
        let mut is_window_shut: bool = self.is_window_open;
        if let Some(message) = self.message.clone() {
            eframe::egui::Window::new("Window...!")
                .open(&mut is_window_shut)
                .show(ctx, |ui| {
                    ui.label(message);
                });
        }
        is_window_shut
    }
    pub fn try_access(
        window_message_am_clone: &Arc<Mutex<WindowMessage>>,
        mut f: impl FnMut(MutexGuard<WindowMessage>),
    ) {
        loop {
            if let Ok(access) = window_message_am_clone.try_lock() {
                f(access);
                break;
            }
        }
    }
}

#[derive(Default)]
pub struct Victim {
    pub name: String,
    pub staged: bool, // If the Victim is staged, it is to be deleted (staged for deletion)
}
#[derive(Default)]
pub struct DeletionQueue {
    pub queue: Option<Vec<Victim>>,
    pub is_window_open: bool,
    pub try_to_open_window: bool,
}

impl DeletionQueue {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
    pub fn build_deletion_queue(
        all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
        modal: &String,
    ) -> Vec<Victim> {
        let mut queue = Vec::new();
        let mut subfn = || -> Option<()> {
            let hasher = all_nether_portal_images.lock().unwrap();
            let state_of_images = hasher.get(modal)?;
            let hasher = state_of_images.hashmap_ref()?;

            for (_key, image_and_details) in hasher.iter() {
                queue.push(Victim {
                    name: image_and_details.image_details.name.to_string(),
                    staged: false,
                });
            }
            Some(())
        };
        subfn();
        queue
    }

    pub fn handle_deletion_queue(
        all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
        some_deletion_queue: &mut Option<DeletionQueue>,
        current_nether_portal_modal: &String,
        ui: &mut eframe::egui::Ui,
        ctx: eframe::egui::Context,
    ) -> Option<()> {
        let deletion_queue = some_deletion_queue.get_or_insert(DeletionQueue::default());
        deletion_queue.is_window_open = deletion_queue.display_deletion_window(&ctx);
        DeletionQueue::open_window_on_click(
            ui,
            "Deletion Queue",
            deletion_queue,
            all_nether_portal_images,
            current_nether_portal_modal,
        );

        Some(())
    }

    fn update_deletion_queue(
        &mut self,
        all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
        modal: &String,
    ) -> Option<()> {
        let mut subfn = || -> Option<()> {
            let hasher = all_nether_portal_images.lock().unwrap();
            let state_of_images = hasher.get(modal)?;
            let hasher = state_of_images.hashmap_ref()?;

            let queue = self.queue.as_mut()?;
            for (_key, image_and_details) in hasher.iter() {
                let mut should_push: bool = true;
                for victim in queue.iter_mut() {
                    if victim.name == image_and_details.image_details.name {
                        should_push = false;
                    }
                }

                if should_push {
                    queue.push(Victim {
                        name: image_and_details.image_details.name.to_string(),
                        staged: false,
                    });
                }
            }
            Some(())
        };
        subfn()
    }

    fn update(
        deletion_queue: &mut DeletionQueue,
        all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
        modal: &String,
    ) {
        if let None = deletion_queue.queue {
            deletion_queue.queue = Some(DeletionQueue::build_deletion_queue(
                all_nether_portal_images,
                modal,
            ));
        }
        if let None = deletion_queue.update_deletion_queue(all_nether_portal_images, modal) {
            println!("update failed somehow?");
        }
        println!("we updated the queue!!!");
    }

    pub fn open_window_on_click(
        ui: &mut eframe::egui::Ui,
        name: &str,
        deletion_queue: &mut DeletionQueue,
        all_nether_portal_images: &mut Arc<Mutex<HashMap<String, StateOfImages>>>,
        modal: &String,
    ) {
        eframe::egui::Grid::new(935).show(ui, |ui| {
            if ui.button(name).clicked() {
                deletion_queue.try_to_open_window = true;
                DeletionQueue::update(deletion_queue, all_nether_portal_images, modal);
            }
            if deletion_queue.try_to_open_window {
                if deletion_queue.queue.is_none() {
                    ui.label("There is no deletion queue?");
                } else {
                    deletion_queue.is_window_open = true;
                    deletion_queue.try_to_open_window = false;
                }
            }
        });
    }

    fn deletion_queue_machine(&mut self, ui: &mut eframe::egui::Ui) {
        let mut subfn = || -> Option<()> {
            let queue = self.queue.as_mut()?;
            for victim in queue.iter_mut() {
                ui.horizontal(|ui| -> Option<()> {
                    ui.label(&victim.name);
                    let message = format!("Staged for deletion: |{}|", victim.staged.to_string());
                    if ui.button(message).clicked() {
                        victim.staged = !victim.staged;
                    }
                    Some(())
                });
            }
            Some(())
        };
        subfn();
    }
    pub fn display_deletion_window(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut is_window_open = self.is_window_open;
        eframe::egui::Window::new("Deletion Window...!")
            .open(&mut is_window_open)
            .show(ctx, |ui| {
                self.deletion_queue_machine(ui);
            });

        is_window_open
    }
}
