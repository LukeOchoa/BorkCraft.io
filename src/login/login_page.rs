use crate::borkcraft_app::BorkCraft;
use eframe::egui;

const LOGIN_FORM: &'static [&'static str] = &["username", "password"];

pub fn login(the_self: &mut BorkCraft, ui: &mut egui::Ui) {
    egui::Grid::new(1).show(ui, |ui| {
        for item in LOGIN_FORM.iter() {
            ui.label(item.clone());
            ui.add(egui::TextEdit::singleline(&mut the_self.login_form[item]));
            ui.end_row();
        }
    });
}
