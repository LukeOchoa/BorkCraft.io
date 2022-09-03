use crate::login::login_page::login;
use eframe::egui;
use egui_extras::RetainedImage;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::{Arc, Mutex},
};

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

#[derive(Default)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}
impl Index<&'_ str> for LoginForm {
    type Output = String;
    fn index(&self, s: &str) -> &String {
        match s {
            "password" => &self.password,
            "username" => &self.username,
            _ => panic!("unknown field: {}", s),
        }
    }
}
impl IndexMut<&'_ str> for LoginForm {
    fn index_mut(&mut self, s: &str) -> &mut String {
        match s {
            "password" => &mut self.password,
            "username" => &mut self.username,
            _ => panic!("unknown field: {}", s),
        }
    }
}

impl LoginForm {
    pub fn get(&self, index: &str) -> Result<&String, String> {
        match index {
            "password" => Ok(&self.password),
            "username" => Ok(&self.username),
            _ => Err(format!("This struct member: |{}| does not exist", index)),
        }
    }
}

pub struct BorkCraft {
    pub image_cache: Arc<Mutex<ImageCache>>,
    pub login_form: LoginForm,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            image_cache: Arc::new(Mutex::new(ImageCache::default())),
            login_form: LoginForm::default(),
        }
    }
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            login(self, ui);
        });
    }
}
