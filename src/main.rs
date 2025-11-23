mod save_file_parser;
mod save_data_info;

use std::{sync::{LazyLock, Mutex}};
use eframe::egui::{self, CentralPanel, Context, FontId, TextStyle, TopBottomPanel};
use egui_extras::{Column, TableBuilder};
use save_file_parser::{read_save_file, get_save_file_variable};
use save_data_info::SaveDataVar;

use crate::{save_file_parser::get_int_value_from_save_data};

//const EXPECTED_SAVE_FILE_SIZE: usize = 176_608;
static SAVE_DATA: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| {
    // this closure runs only once, on the first access.
    // it initializes the data inside the Mutex.
    Mutex::new(Vec::new())
});

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
    let save_file_data_read_res = read_save_file();

    match save_file_data_read_res {
        Ok(raw_bytes) => {
            let mut vec_guard = SAVE_DATA.lock().unwrap();
            vec_guard.clear();
            vec_guard.extend(raw_bytes.iter());
            Some(raw_bytes)
        }
        Err(err_mess) => {
            println!("{}", err_mess);
            None
        },
    };
    
    // GUI starts here
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
            .max_col_width(available_space.x / 2.)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label("Save 1");
                    generate_main_menu_table(ui, 1);
                });
                ui.vertical(|ui| {
                    ui.label("Save 2");
                    generate_main_menu_table(ui, 2);
                });
                ui.end_row();

                ui.vertical(|ui| {
                    ui.label("Save 3");
                    generate_main_menu_table(ui, 3);
                });
                ui.vertical(|ui| {
                    ui.label("Save 4");
                    generate_main_menu_table(ui, 4);
                });
            });
    });
}

fn generate_main_menu_table(ui: &mut eframe::egui::Ui, save_slot: u8) {

    let id = match save_slot {
        1 => "Save Slot 1",
        2 => "Save Slot 2",
        3 => "Save Slot 3",
        4 => "Save Slot 4",
        _ => {
            println!("Wrong slot for table, using 1: {}", save_slot);
            "Save Slot 1"
        }
    };

    let save_data_guard = SAVE_DATA.lock().unwrap();

    let file_exists_data = get_save_file_variable(SaveDataVar::FileExists, save_slot);
    let file_exists = get_int_value_from_save_data(save_data_guard.to_vec(), file_exists_data.slot_base_add, file_exists_data.offset, file_exists_data.int_type);

    if file_exists == 0 {
        if ui.button("Create Save File").clicked() {
            println!("TODO!");
        }
        return;
    }

    let lives_data = get_save_file_variable(SaveDataVar::Lives, save_slot);
    let lives = get_int_value_from_save_data(save_data_guard.to_vec(), lives_data.slot_base_add.into(), lives_data.offset, lives_data.int_type);

    let hours_data = get_save_file_variable(SaveDataVar::PlayTimeHours, save_slot);
    let minutes_data = get_save_file_variable(SaveDataVar::PlayTimeMinutes, save_slot);
    let sec_data = get_save_file_variable(SaveDataVar::PlayTimeSeconds, save_slot);
    let hours = get_int_value_from_save_data(save_data_guard.to_vec(), hours_data.slot_base_add, hours_data.offset, hours_data.int_type);
    let minutes = get_int_value_from_save_data(save_data_guard.to_vec(), minutes_data.slot_base_add, minutes_data.offset, minutes_data.int_type);
    let seconds  = get_int_value_from_save_data(save_data_guard.to_vec(), sec_data.slot_base_add, sec_data.offset, sec_data.int_type);

    let hours_str = if hours <= 9 {
        format!("00{}", hours)
    } else if hours <= 99 {
        format!("0{}", hours)
    } else {
        hours.to_string()
    };
    let min_str = if minutes <= 9 {
        format!("0{}", minutes)
    } else {
        minutes.to_string()
    };
    let sec_str = if seconds <= 9 {
        format!("0{}", seconds)
    } else {
        seconds.to_string()
    };
    let final_time = format!("{}:{}:{}", hours_str, min_str, sec_str);

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
                    ui.label(format!("{}", lives));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Time");
                });
                row.col(|ui| {
                    ui.label(format!("{}", final_time));
                });
            });
        });
    if     ui.button("See details").clicked() {
        println!("TODO!");
    }
    if     ui.button("Edit").clicked() {
        println!("TODO!");
    }

}
