use eframe::egui;

pub struct BorkCraft {
    pub placeholder: String,
}

impl Default for BorkCraft {
    fn default() -> Self {
        let placeholder_string = String::from("Just a placeholder");

        Self {
            placeholder: placeholder_string,
        }
    }
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(&self.placeholder);
        });
    }
}
