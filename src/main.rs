mod new_file;
mod save_data_info;
mod save_file_parser;

use eframe::egui::{self, CentralPanel, Context, FontId, IconData, TextStyle, TopBottomPanel, Ui};
use egui_extras::{Column, TableBuilder};
use save_data_info::SaveDataVar;
use save_file_parser::{get_save_file_variable, read_save_file};
use std::sync::{LazyLock, Mutex};

use crate::{
    new_file::get_new_save_file,
    save_data_info::{
        SaveDataIntType, SaveFileData, array_index_to_input_type, bgm_music_str_to_name,
        costume_int_to_name, get_save_slot_base_add, int_to_controller_btn, int_to_key,
        int_to_maze_name, int_to_stage_name,
    },
    save_file_parser::{
        get_all_save_file_vars, get_basic_save_file_vars, get_figure_info_from_save_data,
        get_int_array_from_save_data, get_int_value_from_save_data, get_text_value_from_save_data, write_save_file,
    },
};

//const EXPECTED_SAVE_FILE_SIZE: usize = 176_608;
static SAVE_DATA: LazyLock<Mutex<Vec<u8>>> = LazyLock::new(|| {
    // this closure runs only once, on the first access.
    // it initializes the data inside the Mutex.
    Mutex::new(Vec::new())
});

#[derive(Default)]
enum CurrentMenu {
    #[default]
    Main,
    FileDetails,
}

#[derive(Default)]
enum SaveFileCurrentView {
    #[default]
    AllVars,
    SingleArray(SaveFileData),
}

#[derive(Default)]
struct App {
    current_view: CurrentMenu,
    single_save_file_view: SaveFileCurrentView,
    show_save_code_variables: bool,
    show_addresses: bool,
    show_simple_data_only: bool,
    save_slot_chosen: u8,
    scroll_to_top: bool,
    edited_save_file: bool,
    show_confirm_exit_modal: bool,
    show_confirm_reload_modal: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        set_styles(ctx);
        self.show_top_bar(ctx);
        match self.current_view {
            CurrentMenu::Main => {
                self.show_main_menu(ctx);
            }
            CurrentMenu::FileDetails => {
                match &self.single_save_file_view {
                    SaveFileCurrentView::AllVars => self.show_details_save_file(ctx),
                    SaveFileCurrentView::SingleArray(var_data) => {
                        self.show_single_array_table(ctx, var_data.clone());
                    }
                };
            }
        };
    }
}

fn main() -> Result<(), eframe::Error> {
    println!("Hello, world!");
    load_save_file();

    let icon = load_icon();

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_resizable(true)
        .with_inner_size([900., 600.])
        .with_icon(icon);

    let options: eframe::NativeOptions = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Waka",
        options,
        Box::new(|_ctx| {
            Ok(Box::<App>::new(App {
                show_addresses: true,
                ..Default::default()
            }))
        }),
    )
}

fn load_save_file() {
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
        }
    };
}

fn load_icon() -> IconData {
    let image = image::load_from_memory(include_bytes!("../pacattack.png"))
        .expect("Failed to load icon")
        .to_rgba8();
    let width = image.width();
    let height = image.height();
    let rgba = image.into_raw(); // Vec<u8>

    IconData {
        rgba,
        width,
        height,
    }
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
    style.spacing.item_spacing.x = 20.;
    ctx.set_style(style);
}

impl App {
    fn reload_save_file(&mut self) {
        if self.edited_save_file {
            self.show_confirm_reload_modal = true;
        } else {
            load_save_file();
        }
    }

    fn show_top_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Reload Save Data").clicked() {
                        self.reload_save_file();
                    }
                    let save_text = if self.edited_save_file {
                        "Save Changes To File *"
                    } else {
                        "Save Changes To File  "
                    };
                    if ui.button(save_text).clicked() {
                        self.edited_save_file = false;
                        let save_data_guard = SAVE_DATA.lock().unwrap();
                        match write_save_file(save_data_guard.to_vec()) {
                            Ok(_) => {
                                println!("Save successful!");
                            },
                            Err(e) => {
                                println!("ERROR: {}", e);
                            },
                        }
                    }
                    if ui.button("Exit").clicked() {
                        if self.edited_save_file {
                            self.show_confirm_exit_modal = true;
                        } else {
                            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_simple_data_only, "Basic Data Only");
                    ui.checkbox(&mut self.show_addresses, "Show Addresses");
                    ui.checkbox(&mut self.show_save_code_variables, "Show Names in Code");
                });
            });
        });
    }

    fn show_main_menu(&mut self, ctx: &Context) {
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
                        self.generate_main_menu_table(ui, 1);
                    });
                    ui.vertical(|ui| {
                        ui.label("Save 2");
                        self.generate_main_menu_table(ui, 2);
                    });
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label("Save 3");
                        self.generate_main_menu_table(ui, 3);
                    });
                    ui.vertical(|ui| {
                        ui.label("Save 4");
                        self.generate_main_menu_table(ui, 4);
                    });
                });
            self.proceed_confirm_reload(ui);
            self.confirm_close_without_save(ctx, ui);
        });
    }

    fn generate_main_menu_table(&mut self, ui: &mut eframe::egui::Ui, save_slot: u8) {
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

        let mut save_data_guard = SAVE_DATA.lock().unwrap();

        let file_exists_data = get_save_file_variable(SaveDataVar::FileExists, save_slot);
        let file_exists = get_int_value_from_save_data(
            save_data_guard.to_vec(),
            file_exists_data.slot_base_add,
            file_exists_data.offset,
            &file_exists_data.int_type,
        );

        if file_exists == 0 {
            if ui.button("Create Save File").clicked() {
                self.edited_save_file = true;
                let new_save_file = get_new_save_file();
                let start_add = get_save_slot_base_add(save_slot);

                for (i, new_save_byte) in new_save_file.iter().enumerate() {
                    save_data_guard[start_add as usize + i] = *new_save_byte;
                }
            }
            return;
        }

        let lives_data = get_save_file_variable(SaveDataVar::Lives, save_slot);
        let lives = get_int_value_from_save_data(
            save_data_guard.to_vec(),
            lives_data.slot_base_add.into(),
            lives_data.offset,
            &lives_data.int_type,
        );

        let hours_data = get_save_file_variable(SaveDataVar::PlayTimeHours, save_slot);
        let minutes_data = get_save_file_variable(SaveDataVar::PlayTimeMinutes, save_slot);
        let sec_data = get_save_file_variable(SaveDataVar::PlayTimeSeconds, save_slot);
        let hours = get_int_value_from_save_data(
            save_data_guard.to_vec(),
            hours_data.slot_base_add,
            hours_data.offset,
            &hours_data.int_type,
        );
        let minutes = get_int_value_from_save_data(
            save_data_guard.to_vec(),
            minutes_data.slot_base_add,
            minutes_data.offset,
            &minutes_data.int_type,
        );
        let seconds = get_int_value_from_save_data(
            save_data_guard.to_vec(),
            sec_data.slot_base_add,
            sec_data.offset,
            &sec_data.int_type,
        );

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
        if ui.button("See details").clicked() {
            self.save_slot_chosen = save_slot;
            self.current_view = CurrentMenu::FileDetails;
        }
        if ui.button("Edit").clicked() {
            println!("TODO!");
        }
    }

    fn show_details_save_file(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            let available_space = ui.available_size();
            ui.set_min_size(available_space);

            ui.horizontal(|ui| {
                if ui.button("Go Back").clicked() {
                    self.current_view = CurrentMenu::Main;
                };
            });

            let mut extra_columns = 1;
            if self.show_addresses {
                extra_columns += 1;
            }
            if self.show_save_code_variables {
                extra_columns += 1;
            }

            let table = TableBuilder::new(ui)
                .id_salt("table_details")
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .columns(Column::auto(), extra_columns)
                .column(
                    Column::remainder()
                        .at_least(40.0)
                        .clip(true)
                        .resizable(true),
                )
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                        ui.set_width(50.);
                    });
                    if self.show_save_code_variables {
                        header.col(|ui| {
                            ui.strong("Name in Code");
                            ui.set_width(50.);
                        });
                    }
                    if self.show_addresses {
                        header.col(|ui| {
                            ui.strong("Address");
                            ui.set_width(50.);
                        });
                    }
                    header.col(|ui| {
                        ui.strong("Value");
                    });
                });

            let table_vars = if self.show_simple_data_only {
                get_basic_save_file_vars(self.save_slot_chosen)
            } else {
                get_all_save_file_vars(self.save_slot_chosen)
            };
            let save_data_guard = SAVE_DATA.lock().unwrap();

            table.body(|mut body| {
                for var_data in table_vars {
                    let value_str: String = match var_data.int_type {
                        save_data_info::SaveDataIntType::Bool => {
                            let val_int = get_int_value_from_save_data(
                                save_data_guard.to_vec(),
                                var_data.slot_base_add,
                                var_data.offset,
                                &var_data.int_type,
                            );
                            let val_bool_str: String;
                            if val_int == 0 {
                                val_bool_str = "False".to_string();
                            } else if val_int == 1 {
                                val_bool_str = "True".to_string();
                            } else {
                                val_bool_str = "Error: Not 0 or 1".to_string();
                            }
                            val_bool_str
                        }
                        save_data_info::SaveDataIntType::U32 => {
                            let val_int = get_int_value_from_save_data(
                                save_data_guard.to_vec(),
                                var_data.slot_base_add,
                                var_data.offset,
                                &var_data.int_type,
                            );
                            val_int.to_string()
                        }
                        save_data_info::SaveDataIntType::I32 => {
                            let val_int = get_int_value_from_save_data(
                                save_data_guard.to_vec(),
                                var_data.slot_base_add,
                                var_data.offset,
                                &var_data.int_type,
                            );
                            val_int.to_string()
                        }
                        save_data_info::SaveDataIntType::Arrayi32(_)
                        | save_data_info::SaveDataIntType::Arrayu8(_)
                        | save_data_info::SaveDataIntType::SFigureDisplayInfoArray(_) => {
                            "list".to_string()
                        }
                        save_data_info::SaveDataIntType::ArrayText(_) => {
                            get_text_value_from_save_data(
                                save_data_guard.to_vec(),
                                var_data.slot_base_add,
                                var_data.offset,
                                &var_data.int_type,
                            )
                        }
                    };

                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(var_data.variable_name_simple.clone());
                        });

                        if self.show_save_code_variables {
                            row.col(|ui| {
                                ui.label(var_data.variable_name.clone());
                            });
                        }
                        if self.show_addresses {
                            row.col(|ui| {
                                ui.label(format!("{:X}", var_data.slot_base_add + var_data.offset));
                            });
                        }
                        row.col(|ui| {
                            if value_str == "list" {
                                if ui.button("Open").clicked() {
                                    self.scroll_to_top = true;
                                    self.single_save_file_view =
                                        SaveFileCurrentView::SingleArray(var_data);
                                }
                            } else {
                                match var_data.var {
                                    SaveDataVar::JukeBoxBGM | SaveDataVar::JukeBoxBGMCollab => {
                                        ui.label(bgm_music_str_to_name(
                                            value_str.parse().unwrap_or(-100),
                                        ));
                                    }
                                    SaveDataVar::PlayerSkinId
                                    | SaveDataVar::PlayerSkinId2
                                    | SaveDataVar::PlayerSkinIdCollab => {
                                        ui.label(costume_int_to_name(
                                            value_str.parse().unwrap_or(-100),
                                        ));
                                    }
                                    _ => {
                                        ui.label(value_str);
                                    }
                                };
                            }
                        });
                    });
                }
            });
            self.proceed_confirm_reload(ui);
            self.confirm_close_without_save(ctx, ui);
        });
    }

    fn show_single_array_table(&mut self, ctx: &Context, var_data: SaveFileData) {
        CentralPanel::default().show(ctx, |ui| {
            let available_space = ui.available_size();
            ui.set_min_size(available_space);

            ui.horizontal(|ui| {
                if ui.button("Go Back To All Data").clicked() {
                    self.single_save_file_view = SaveFileCurrentView::AllVars;
                };
            });

            let table = TableBuilder::new(ui)
                .id_salt(var_data.variable_name)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::auto())
                .column(
                    Column::remainder()
                        .at_least(40.0)
                        .clip(true)
                        .resizable(true),
                )
                .header(20.0, |mut header| {
                    match var_data.var {
                        SaveDataVar::StageFlagList
                        | SaveDataVar::ScoreList
                        | SaveDataVar::TimeTrialList => {
                            header.col(|ui| {
                                ui.strong("Level");
                                ui.set_width(50.);
                            });
                        }
                        _ => {
                            header.col(|ui| {
                                ui.strong("Position");
                                ui.set_width(50.);
                            });
                        }
                    }

                    if self.show_addresses {
                        header.col(|ui| {
                            ui.strong("Address");
                            ui.set_width(50.);
                        });
                    }
                    header.col(|ui| {
                        ui.strong("Value");
                    });
                });

            let save_data_guard = SAVE_DATA.lock().unwrap();

            match var_data.int_type {
                SaveDataIntType::Arrayi32(_) | SaveDataIntType::Arrayu8(_) => {
                    let vec_for_table = get_int_array_from_save_data(
                        save_data_guard.to_vec(),
                        var_data.slot_base_add,
                        var_data.offset,
                        &var_data.int_type,
                    );

                    table.body(|mut body| {
                        for (i, var) in vec_for_table.iter().enumerate() {
                            // TODO somehow sort by name in input config view
                            if self.show_simple_data_only
                                && (var_data.var == SaveDataVar::KeyConfigP1
                                    || var_data.var == SaveDataVar::KeyConfigP2)
                            {
                                let indexes_with_main_config = [
                                    72, 73, 75, 74, 456, 458, 509, 519, 503, 77, 81, 122, 48, 49,
                                    51, 50, 432, 434, 485, 495, 480, 53, 57, 98,
                                ];
                                if !indexes_with_main_config.contains(&i) {
                                    continue;
                                }
                            }

                            body.row(30.0, |mut row| {
                                match var_data.var {
                                    SaveDataVar::ScoreList
                                    | SaveDataVar::TimeTrialList
                                    | SaveDataVar::TimeTrialCoopList
                                    | SaveDataVar::StageFlagList
                                    | SaveDataVar::StageCherryFlag
                                    | SaveDataVar::StageStrawberryFlag
                                    | SaveDataVar::StageOrangeFlag
                                    | SaveDataVar::StageAppleFlag
                                    | SaveDataVar::StageMelonFlag
                                    | SaveDataVar::StageCherryNum
                                    | SaveDataVar::StageStrawberryNum
                                    | SaveDataVar::StageOrangeNum
                                    | SaveDataVar::StageAppleNum
                                    | SaveDataVar::StageMelonNum => {
                                        let level_name = int_to_stage_name(i, false);
                                        row.col(|ui| {
                                            ui.label(level_name);
                                        });
                                    }
                                    SaveDataVar::StageMazeFlagList
                                    | SaveDataVar::MazeFlagList
                                    | SaveDataVar::MazesScoreList => {
                                        let maze = int_to_maze_name(i);
                                        row.col(|ui| {
                                            ui.label(maze);
                                        });
                                    }
                                    SaveDataVar::MissionRewardFlag => {
                                        let level_name = int_to_stage_name(i, true);
                                        row.col(|ui| {
                                            ui.label(level_name);
                                        });
                                    }
                                    SaveDataVar::KeyConfigP1 | SaveDataVar::KeyConfigP2 => {
                                        let movement_type = array_index_to_input_type(i);
                                        row.col(|ui| {
                                            ui.label(movement_type);
                                        });
                                    }
                                    _ => {
                                        row.col(|ui| {
                                            ui.label(i.to_string());
                                        });
                                    }
                                }

                                if self.show_addresses {
                                    row.col(|ui| {
                                        let bytes_amount: u32 = match var_data.int_type {
                                            save_data_info::SaveDataIntType::Arrayi32(_) => 4,
                                            save_data_info::SaveDataIntType::Arrayu8(_) => 1,
                                            _ => 0, // should never happen
                                        };
                                        ui.label(format!(
                                            "{:X}",
                                            (var_data.slot_base_add
                                                + var_data.offset
                                                + (i as u32 * bytes_amount))
                                        ));
                                    });
                                }
                                row.col(|ui| match var_data.var {
                                    SaveDataVar::TimeTrialList | SaveDataVar::TimeTrialCoopList => {
                                        let seconds_total = var / 100;
                                        let ms = var - seconds_total * 100;
                                        let minutes = seconds_total / 60;
                                        let seconds = seconds_total % 60;

                                        let ms_str: String;
                                        if ms < 10 {
                                            ms_str = format!("0{}", ms);
                                        } else {
                                            ms_str = ms.to_string();
                                        }

                                        let seconds_str: String;
                                        if seconds < 10 {
                                            seconds_str = format!("0{}", seconds);
                                        } else {
                                            seconds_str = seconds.to_string();
                                        }

                                        ui.label(format!("{}:{}.{}", minutes, seconds_str, ms_str));
                                    }
                                    SaveDataVar::StageFlagList | SaveDataVar::MazeFlagList => {
                                        match var {
                                            0 => ui.label(format!("0 Locked")),
                                            1 => ui.label(format!("1 Unlocked")),
                                            2 => ui.label(format!("2 Entered")),
                                            3 => ui.label(format!("3 Complete")),
                                            _ => ui.label(var.to_string()),
                                        };
                                    }
                                    SaveDataVar::KeyConfigP1 | SaveDataVar::KeyConfigP2 => {
                                        let is_controller =
                                            array_index_to_input_type(i).contains("Controller");
                                        if is_controller {
                                            ui.label(int_to_controller_btn(*var));
                                        } else {
                                            ui.label(int_to_key(*var));
                                        }
                                    }

                                    SaveDataVar::StageCherryFlag
                                    | SaveDataVar::StageStrawberryFlag
                                    | SaveDataVar::StageOrangeFlag
                                    | SaveDataVar::StageAppleFlag
                                    | SaveDataVar::StageMelonFlag => {
                                        ui.label(format!("{} ({:b})", var, var));
                                    }
                                    SaveDataVar::StageMazeFlagList => {
                                        match var {
                                            0 => ui.label(format!("0 Locked")),
                                            1 => ui.label(format!("1 Unlocked")),
                                            _ => ui.label(var.to_string()),
                                        };
                                    }
                                    _ => {
                                        ui.label(var.to_string());
                                    }
                                });
                            });
                        }
                    });
                }

                SaveDataIntType::SFigureDisplayInfoArray(array_len) => {
                    let vec_for_table = get_figure_info_from_save_data(
                        save_data_guard.to_vec(),
                        var_data.slot_base_add,
                        var_data.offset,
                        array_len,
                    );

                    table.body(|mut body| {
                        for (i, figure_info) in vec_for_table.iter().enumerate() {
                            let bytes_amount: u32 = 8;

                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{} ID", i));
                                });

                                if self.show_addresses {
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{:X}",
                                            (var_data.slot_base_add
                                                + var_data.offset
                                                + (i as u32 * bytes_amount))
                                        ));
                                    });
                                }
                                row.col(|ui| {
                                    ui.label(figure_info.figure_id.to_string());
                                });
                            });

                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(format!("{} Angle", i));
                                });

                                if self.show_addresses {
                                    row.col(|ui| {
                                        ui.label(format!(
                                            "{:X}",
                                            (var_data.slot_base_add
                                                + var_data.offset
                                                + 4
                                                + (i as u32 * bytes_amount))
                                        ));
                                    });
                                }
                                row.col(|ui| {
                                    ui.label(format!("{:.1}", figure_info.angle));
                                });
                            });
                        }
                    });
                }
                SaveDataIntType::Bool
                | SaveDataIntType::U32
                | SaveDataIntType::I32
                | SaveDataIntType::ArrayText(_) => {
                    println!("Not an array!");
                }
            }
            self.proceed_confirm_reload(ui);
            self.confirm_close_without_save(ctx, ui);
        });
    }

    fn confirm_close_without_save(&mut self, ctx: &Context, ui: &mut Ui) {
        if !self.show_confirm_exit_modal {
            return;
        }

        let modal =
            eframe::egui::Modal::new(eframe::egui::Id::new("Confirm Exit")).show(ui.ctx(), |ui| {
                ui.set_width(350.0);

                ui.heading("Changes Not Saved, Exit Anyway?");

                ui.separator();

                let close_clicked = egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Exit").clicked() {
                            return true;
                        }
                        if ui.button("Cancel").clicked() {
                            // This causes the current modals `should_close` to return true
                            ui.close();
                        }
                        return false;
                    },
                );
                return close_clicked.1;
            });

        if modal.should_close() {
            self.show_confirm_exit_modal = false;
        }
        if modal.inner {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        };
    }

    fn proceed_confirm_reload(&mut self, ui: &mut Ui) {
        if !self.show_confirm_reload_modal {
            return;
        }

        let modal = eframe::egui::Modal::new(eframe::egui::Id::new("Confirm Reload")).show(
            ui.ctx(),
            |ui| {
                ui.set_width(350.0);

                ui.heading("Changes Not Saved, Reload Anyway?");

                ui.separator();

                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Reload").clicked() {
                            load_save_file();
                            self.edited_save_file = false;
                            ui.close();
                        }
                        if ui.button("Cancel").clicked() {
                            // This causes the current modals `should_close` to return true
                            ui.close();
                        }
                    },
                );
            },
        );

        if modal.should_close() {
            self.show_confirm_reload_modal = false;
        }
    }
}
