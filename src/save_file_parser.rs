use std::{env, fs};

use crate::save_data_info::{SaveDataVar, SaveFileData, SaveDataIntType, get_save_slot_base_add};

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

pub fn get_int_value_from_save_data(save_file_raw: Vec<u8>, slot_base: u32, offset: u32, int_type: SaveDataIntType) -> i64 {

    match int_type {
        SaveDataIntType::Bool => {
            save_file_raw[slot_base as usize + offset as usize].into()
        },
        SaveDataIntType::U32 => {
            let save_data_slice: &[u8] = &save_file_raw[(slot_base as usize + offset as usize)..(slot_base as usize + offset as usize + 4)];
            let value_raw = u32::from_le_bytes(save_data_slice.try_into().unwrap_or_default());
            value_raw.into()
        },
        SaveDataIntType::I32 => {
            let save_data_slice: &[u8] = &save_file_raw[(slot_base as usize + offset as usize)..(slot_base as usize + offset as usize + 4)];
            let value_raw = i32::from_le_bytes(save_data_slice.try_into().unwrap_or_default());
            value_raw.into()
        },
        SaveDataIntType::Arrayi32(_) => {
            0
        },
    }

}

pub fn get_save_file_variable(
    req_data: SaveDataVar,
    slot: u8,
) -> SaveFileData {
    let slot_base_add: u32 = get_save_slot_base_add(slot);

    match req_data {
        SaveDataVar::FileExists => {
            SaveFileData {
                variable_name: "m_bExist".into(),
                variable_name_simple: "File Exists".into(),
                offset: 0x0,
                int_type: SaveDataIntType::Bool,
                slot_base_add,
                var: req_data,
            }
        },
        SaveDataVar::PlayTimeHours => {
            SaveFileData {
                variable_name: "m_iPlayHours".into(),
                variable_name_simple: "File Hours".into(),
                offset: 0x28,
                int_type: SaveDataIntType::I32,
                slot_base_add,
                var: req_data,
            }
        },
        SaveDataVar::PlayTimeMinutes => {
            SaveFileData {
                variable_name: "m_iPlayMinutes".into(),
                variable_name_simple: "File Minutes".into(),
                offset: 0x2C,
                int_type: SaveDataIntType::I32,
                slot_base_add,
                var: req_data,
            }
        },
        SaveDataVar::PlayTimeSeconds => {
            SaveFileData {
                variable_name: "m_iPlaySeconds".into(),
                variable_name_simple: "File Seconds".into(),
                offset: 0x30,
                int_type: SaveDataIntType::I32,
                slot_base_add,
                var: req_data,
            }
        },
        SaveDataVar::Lives => {
            SaveFileData {
                variable_name: "m_iStockNum".into(),
                variable_name_simple: "Lives".into(),
                offset: 0x46C,
                int_type: SaveDataIntType::I32,
                slot_base_add,
                var: req_data,
            }
        },
    }
}
