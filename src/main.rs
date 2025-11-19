use eframe::egui::CentralPanel;

#[derive(Default)]
struct App {
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| ui.heading("Hello from app"));
    }
}

fn main() -> Result<(), eframe::Error>{
    println!("Hello, world!");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_resizable(true).with_inner_size([320., 320.,]),
        ..Default::default()
    };

    eframe::run_native("Waka", options, Box::new(|_ctx| Ok(Box::<App>::default())))
}
