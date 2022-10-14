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
}
