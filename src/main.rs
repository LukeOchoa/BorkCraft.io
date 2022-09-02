use borkcraftclient::borkcraft_app::BorkCraft;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "BorkCraft.io",
        options,
        Box::new(|_cc| Box::new(BorkCraft::default())),
    );
}
