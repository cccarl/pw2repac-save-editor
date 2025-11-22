use eframe::egui::{self, CentralPanel, Context, FontId, TextStyle, TopBottomPanel};
use egui_extras::{Column, TableBuilder};

#[derive(Default)]
struct App {}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        set_styles(ctx);
        show_top_bar(ctx);
        show_central_panel(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    println!("Hello, world!");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([800., 600.]),
        ..Default::default()
    };

    eframe::run_native("Waka", options, Box::new(|_ctx| Ok(Box::<App>::default())))
}

fn set_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(30.0, eframe::egui::FontFamily::Monospace),
        ),
        (
            TextStyle::Body,
            FontId::new(18.0, eframe::egui::FontFamily::Monospace),
        ),
        (
            TextStyle::Button,
            FontId::new(22.0, eframe::egui::FontFamily::Monospace),
        ),
        (
            TextStyle::Small,
            FontId::new(14.0, eframe::egui::FontFamily::Monospace),
        ),
    ]
    .into();
    ctx.set_style(style);
}

fn show_top_bar(ctx: &Context) {
    TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                }
            });
        });
    });
}

fn show_central_panel(ctx: &Context) {
    CentralPanel::default().show(ctx, |ui| {
        let available_space = ui.available_size();
        ui.set_min_size(available_space);

        egui::Grid::new("FileSelectGrid")
            .min_col_width(available_space.x / 2.)
            .min_row_height(available_space.y / 2.)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label("Save 1");
                    generate_main_menu_table(ui, "Slot 1");
                });
                ui.vertical(|ui| {
                    ui.label("Save 2");
                    generate_main_menu_table(ui, "Slot 2");
                });
                ui.end_row();

                ui.vertical(|ui| {
                    ui.label("Save 3");
                    generate_main_menu_table(ui, "Slot 3");
                });
                ui.vertical(|ui| {
                    ui.label("Save 4");
                    generate_main_menu_table(ui, "Slot 4");
                });
            });
    });
}

fn generate_main_menu_table(ui: &mut eframe::egui::Ui, id: &str) {
    TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(
            Column::remainder()
                .at_least(40.0)
                .clip(true)
                .resizable(true),
        )
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Data Name");
                ui.set_width(50.);
            });
            header.col(|ui| {
                ui.strong("Data");
            });
        })
        .body(|mut body| {
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Lives");
                });
                row.col(|ui| {
                    ui.label("-");
                });
            });
        });

}
