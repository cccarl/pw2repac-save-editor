use std::{env, fs, str::from_utf8};
use enum_iterator::all;

use crate::save_data_info::{SaveDataIntType, SaveDataVar, SaveFileData, get_save_slot_base_add};

pub fn read_save_file() -> Result<Vec<u8>, String> {
    let pac_save_path_res = env::var("LOCALAPPDATA");

    let pac_save_path = match pac_save_path_res {
        Ok(mut local_app_data_path) => {
            local_app_data_path.insert_str(local_app_data_path.len(), "\\BANDAI NAMCO Entertainment\\PAC-MAN WORLD2 Re-Pac\\Saved\\SaveGames\\181732721\\DAT00000.dat");
            local_app_data_path
        }
        Err(var_err) => {
            return Err(format!("Error in local data path value: {}", var_err));
        }
    };

    let save_file_bytes = fs::read(pac_save_path);

    match save_file_bytes {
        Ok(vec) => Ok(vec),
        Err(err) => Err(format!("Error when reading bytes from save file: {}", err)),
    }
}

pub fn get_int_value_from_save_data(
    save_file_raw: Vec<u8>,
    slot_base: u32,
    offset: u32,
    int_type: &SaveDataIntType,
) -> i64 {
    match int_type {
        SaveDataIntType::Bool => save_file_raw[slot_base as usize + offset as usize].into(),
        SaveDataIntType::U32 => {
            let save_data_slice: &[u8] = &save_file_raw[(slot_base as usize + offset as usize)
                ..(slot_base as usize + offset as usize + 4)];
            let value_raw = u32::from_le_bytes(save_data_slice.try_into().unwrap_or_default());
            value_raw.into()
        }
        SaveDataIntType::I32 => {
            let save_data_slice: &[u8] = &save_file_raw[(slot_base as usize + offset as usize)
                ..(slot_base as usize + offset as usize + 4)];
            let value_raw = i32::from_le_bytes(save_data_slice.try_into().unwrap_or_default());
            value_raw.into()
        }
        SaveDataIntType::Arrayi32(_) | SaveDataIntType::ArrayText(_) | SaveDataIntType::Arrayu8(_) => {
            println!("Wrong function for this... offset: {}", offset);
            0
        },
    }
}

pub fn get_text_value_from_save_data(
    save_file_raw: Vec<u8>,
    slot_base: u32,
    offset: u32,
    int_type: &SaveDataIntType,
) -> String {
    match int_type {
        SaveDataIntType::ArrayText(len) => {
            let save_data_slice: &[u8] = &save_file_raw[(slot_base as usize + offset as usize)
                ..(slot_base as usize + offset as usize + *len as usize)];
            match from_utf8(save_data_slice) {
                Ok(str) => str.to_string(),
                Err(_) => {
                    String::from("Error")
                },
            }
        },
        _ => {
            println!("This isn't text!");
            String::from("Error")
        }
    }
}

pub fn get_all_save_file_vars(slot: u8) -> Vec<SaveFileData> {
    let all_vars_iter = all::<SaveDataVar>();
    let mut all_vars = vec![];
    for var in all_vars_iter {
        all_vars.push(get_save_file_variable(var, slot));
    }
    all_vars
}

pub fn get_save_file_variable(req_data: SaveDataVar, slot: u8) -> SaveFileData {
    let slot_base_add: u32 = get_save_slot_base_add(slot);

    match req_data {
        SaveDataVar::FileExists => SaveFileData {
            variable_name: "m_bExist".into(),
            variable_name_simple: "File Exists".into(),
            offset: 0x0,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::SaveDate => SaveFileData {
            variable_name: "m_bDateList".into(),
            variable_name_simple: "Last Save Date".into(),
            offset: 0x18,
            int_type: SaveDataIntType::ArrayText(16),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayTimeHours => SaveFileData {
            variable_name: "m_iPlayHours".into(),
            variable_name_simple: "File Hours".into(),
            offset: 0x28,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayTimeMinutes => SaveFileData {
            variable_name: "m_iPlayMinutes".into(),
            variable_name_simple: "File Minutes".into(),
            offset: 0x2C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayTimeSeconds => SaveFileData {
            variable_name: "m_iPlaySeconds".into(),
            variable_name_simple: "File Seconds".into(),
            offset: 0x30,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageFlagList => SaveFileData {
            variable_name: "m_bStageFlagList".into(),
            variable_name_simple: "Stage Flags".into(),
            offset: 0x34,
            int_type: SaveDataIntType::Arrayu8(40),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::ScoreList => SaveFileData {
            variable_name: "m_iStageScoreList".into(),
            variable_name_simple: "Scores List".into(),
            offset: 0x80,
            int_type: SaveDataIntType::Arrayi32(40),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::TimeTrialList => SaveFileData {
            variable_name: "m_iStageTimeList".into(),
            variable_name_simple: "Time Trials List".into(),
            offset: 0x1AC,
            int_type: SaveDataIntType::Arrayi32(60),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::TimeTrialCoopList => SaveFileData {
            variable_name: "m_iStageTimeListCoop".into(),
            variable_name_simple: "Time Trials Coop List".into(),
            offset: 0x210, // TODO check if this is true lollll
            int_type: SaveDataIntType::Arrayi32(60),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FriendsFlagList => SaveFileData {
            variable_name: "m_bFriendFlagList".into(),
            variable_name_simple: "Friends Flag List".into(),
            offset: 0x210, // TODO check if this is true lollll
            int_type: SaveDataIntType::Arrayi32(60),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::Lives => SaveFileData {
            variable_name: "m_iStockNum".into(),
            variable_name_simple: "Lives".into(),
            offset: 0x46C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
    }
}
