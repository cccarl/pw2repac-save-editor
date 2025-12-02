use enum_iterator::all;
use std::{env, fs, path::Path, str::from_utf8};

use crate::{
    EXPECTED_SAVE_FILE_SIZE,
    save_data_info::{SaveDataIntType, SaveDataVar, SaveFileData, get_save_slot_base_add},
};

const LEVELS_COUNT: u32 = 40;
const MAZES_COUNT: u32 = 15;

pub struct SFigureDisplayInfo {
    pub figure_id: i32,
    pub angle: f32,
}

fn get_file_path() -> Result<String, String> {
    let pac_save_path_res = env::var("LOCALAPPDATA");
    match pac_save_path_res {
        Ok(local_app_data_path_str) => {
            let local_data_path = Path::new(&local_app_data_path_str);
            let save_games_path = local_data_path
                .join("BANDAI NAMCO Entertainment")
                .join("PAC-MAN WORLD2 Re-Pac")
                .join("Saved")
                .join("SaveGames");

            println!("Reading path: {}", save_games_path.to_str().unwrap_or_default());
            let mut save_file_path = String::default();
            match fs::read_dir(save_games_path) {
                Ok(entries) => {
                    // just assume there's 1 folder so "last" element will be in the path
                    for entry_res in entries {
                        match entry_res {
                            Ok(entry) => {
                                save_file_path = entry
                                    .path()
                                    .join("DAT00000.dat")
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string();
                            }
                            Err(e) => eprintln!("Error reading save directory entry: {}", e),
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Error when reading SaveGames path: {}", e));
                },
            }

            println!("Final Path is: {}", save_file_path);
            Ok(save_file_path)
        }
        Err(var_err) => {
            return Err(format!("Error in local data path value: {}", var_err));
        }
    }
}

pub fn read_save_file() -> Result<Vec<u8>, String> {
    let pac_save_path = get_file_path();

    match pac_save_path {
        Ok(path) => {
            let save_file_bytes = fs::read(path);

            match save_file_bytes {
                Ok(vec) => Ok(vec),
                Err(err) => Err(format!("Error when reading bytes from save file: {}", err)),
            }
        }
        Err(e) => return Err(e),
    }
}

pub fn write_save_file(save_data: Vec<u8>) -> std::io::Result<()> {
    let pac_save_path = get_file_path();

    match pac_save_path {
        Ok(path) => {
            fs::write(path, &save_data)?;
            Ok(())
        }
        Err(e) => {
            println!("{}", e);
            Ok(())
        }
    }
}

pub fn get_int_value_from_save_data(
    save_file_raw: Vec<u8>,
    slot_base: u32,
    offset: u32,
    int_type: &SaveDataIntType,
) -> i64 {
    if save_file_raw.len() < EXPECTED_SAVE_FILE_SIZE {
        return 0;
    }
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
        SaveDataIntType::Arrayi32(_)
        | SaveDataIntType::ArrayText(_)
        | SaveDataIntType::Arrayu8(_)
        | SaveDataIntType::SFigureDisplayInfoArray(_) => {
            println!("Wrong function for this... offset: {}", offset);
            0
        }
    }
}

pub fn get_int_array_from_save_data(
    save_file_raw: Vec<u8>,
    slot_base: u32,
    offset: u32,
    int_type: &SaveDataIntType,
) -> Vec<i64> {
    if save_file_raw.len() < EXPECTED_SAVE_FILE_SIZE {
        return vec![];
    }
    match int_type {
        SaveDataIntType::Arrayi32(len) => {
            let mut final_vec: Vec<i64> = vec![];
            for i in 0..(*len as usize) {
                let save_data_slice: &[u8] =
                    &save_file_raw[(slot_base as usize + offset as usize + (i * 4))
                        ..(slot_base as usize + offset as usize + 4 + (i * 4))];
                let value_raw = i32::from_le_bytes(save_data_slice.try_into().unwrap_or_default());
                final_vec.push(value_raw as i64);
            }
            final_vec
        }
        SaveDataIntType::Arrayu8(len) => {
            let mut final_vec: Vec<i64> = vec![];
            for i in 0..(*len as usize) {
                let byte_found: u8 = save_file_raw[slot_base as usize + offset as usize + i];
                final_vec.push(byte_found as i64);
            }
            final_vec
        }
        SaveDataIntType::ArrayText(_) => {
            println!("Text not supported!");
            vec![]
        }
        SaveDataIntType::Bool
        | SaveDataIntType::U32
        | SaveDataIntType::I32
        | SaveDataIntType::SFigureDisplayInfoArray(_) => {
            println!("This isn't an array!");
            vec![]
        }
    }
}

pub fn get_figure_info_from_save_data(
    save_file_raw: Vec<u8>,
    slot_base: u32,
    offset: u32,
    len: u32,
) -> Vec<SFigureDisplayInfo> {
    let mut final_vec: Vec<SFigureDisplayInfo> = vec![];
    for i in 0..(len as usize) {
        let save_data_slice_id: &[u8] =
            &save_file_raw[(slot_base as usize + offset as usize + (i * 8))
                ..(slot_base as usize + offset as usize + 4 + (i * 8))];
        let figure_id = i32::from_le_bytes(save_data_slice_id.try_into().unwrap_or_default());
        let save_data_slice_angle: &[u8] =
            &save_file_raw[(slot_base as usize + offset as usize + 4 + (i * 8))
                ..(slot_base as usize + offset as usize + 8 + (i * 8))];
        let angle = f32::from_le_bytes(save_data_slice_angle.try_into().unwrap_or_default());

        final_vec.push(SFigureDisplayInfo { figure_id, angle });
    }
    final_vec
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
                Err(_) => String::from("Error"),
            }
        }
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

pub fn get_basic_save_file_vars(slot: u8) -> Vec<SaveFileData> {
    let mut basic_vars = vec![];
    basic_vars.push(get_save_file_variable(SaveDataVar::PlayTimeHours, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::PlayTimeMinutes, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::PlayTimeSeconds, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::ScoreList, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::TimeTrialList, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::StageMazeFlagList, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::MazesScoreList, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::Lives, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::MagicKeyUnlocked, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::JukeBoxBGM, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::JukeBoxBGMCollab, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::MedalNum, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::CameraSpeedX, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::CameraSpeedY, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::CameraControlX, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::CameraControlY, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::CameraAssistFlag, slot));
    basic_vars.push(get_save_file_variable(
        SaveDataVar::CameraYAutoRotateFlag,
        slot,
    ));
    basic_vars.push(get_save_file_variable(SaveDataVar::KeyConfigP1, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::PlayerSkinId, slot));
    basic_vars.push(get_save_file_variable(
        SaveDataVar::PlayerSkinIdCollab,
        slot,
    ));
    basic_vars.push(get_save_file_variable(SaveDataVar::OriginalHighScore, slot));
    basic_vars.push(get_save_file_variable(SaveDataVar::PacmaniaHighScore, slot));
    basic_vars.push(get_save_file_variable(
        SaveDataVar::PacAttackHighScore,
        slot,
    ));

    basic_vars
}

pub fn modify_save_data(
    save_data: &mut Vec<u8>,
    slot_base_add: u32,
    offset: u32,
    int_type: SaveDataIntType,
    value_to_write: i64,
) {
    let pos_to_write = (slot_base_add + offset) as usize;

    match int_type {
        SaveDataIntType::Bool => {
            let value_to_bool: u8 = if value_to_write > 0 { 1 } else { 0 };
            save_data[pos_to_write] = value_to_bool;
        }
        SaveDataIntType::U32 => {
            let value_to_bytes: [u8; 4] = (value_to_write as u32).to_le_bytes();
            for (i, byte) in value_to_bytes.iter().enumerate() {
                save_data[pos_to_write + i] = *byte;
            }
        }
        SaveDataIntType::I32
        | SaveDataIntType::Arrayi32(_)
        | SaveDataIntType::SFigureDisplayInfoArray(_) => {
            let value_to_bytes: [u8; 4] = (value_to_write as i32).to_le_bytes();
            for (i, byte) in value_to_bytes.iter().enumerate() {
                save_data[pos_to_write + i] = *byte;
            }
        }
        SaveDataIntType::Arrayu8(_) => {
            let value_to_u8 = value_to_write as u8;
            save_data[pos_to_write] = value_to_u8;
        }
        SaveDataIntType::ArrayText(_) => {
            // meh not worth it
        }
    };
}

pub fn modify_save_data_float(
    save_data: &mut Vec<u8>,
    slot_base_add: u32,
    offset: u32,
    value_to_write: f32,
) {
    let pos_to_write = (slot_base_add + offset) as usize;
    let value_to_bytes: [u8; 4] = (value_to_write as f32).to_le_bytes();
    for (i, byte) in value_to_bytes.iter().enumerate() {
        save_data[pos_to_write + i] = *byte;
    }
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
            int_type: SaveDataIntType::Arrayu8(LEVELS_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::ScoreList => SaveFileData {
            variable_name: "m_iStageScoreList".into(),
            variable_name_simple: "Scores List".into(),
            offset: 0x80,
            int_type: SaveDataIntType::Arrayi32(LEVELS_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::TimeTrialList => SaveFileData {
            variable_name: "m_iStageTimeList".into(),
            variable_name_simple: "Time Trials List".into(),
            offset: 0x1AC,
            int_type: SaveDataIntType::Arrayi32(LEVELS_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::TimeTrialCoopList => SaveFileData {
            variable_name: "m_iStageTimeListCoop".into(),
            variable_name_simple: "Time Trials Coop List".into(),
            offset: 0x210, // TODO check if this is true lollll
            int_type: SaveDataIntType::Arrayi32(LEVELS_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FriendsFlagList => SaveFileData {
            variable_name: "m_bFriendFlagList".into(),
            variable_name_simple: "Friends Flag List".into(),
            offset: 0x404,
            int_type: SaveDataIntType::Arrayu8(9),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FriendsTalkList => SaveFileData {
            variable_name: "m_uFriendTalkList".into(),
            variable_name_simple: "Friends Talk List".into(),
            offset: 0x414,
            int_type: SaveDataIntType::Arrayi32(34), // game says u32 even tho theres clear negs
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageMazeFlagList => SaveFileData {
            variable_name: "m_bStageMazeFlagList".into(),
            variable_name_simple: "Maze Unlocked Flags".into(),
            offset: 0x450,
            int_type: SaveDataIntType::Arrayu8(MAZES_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LastStageId => SaveFileData {
            variable_name: "m_iLastStageId".into(),
            variable_name_simple: "Last Stage Id".into(),
            offset: 0x464,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LastGameModeId => SaveFileData {
            variable_name: "m_iLastGameModeId".into(),
            variable_name_simple: "Last Game Mode Id".into(),
            offset: 0x468,
            int_type: SaveDataIntType::I32,
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
        SaveDataVar::MazeFlagList => SaveFileData {
            variable_name: "m_bMazeFlagList".into(),
            variable_name_simple: "Maze Flag List".into(),
            offset: 0x470,
            int_type: SaveDataIntType::Arrayu8(MAZES_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MazesScoreList => SaveFileData {
            variable_name: "m_iMazeScoreList".into(),
            variable_name_simple: "Mazes Score List".into(),
            offset: 0x4A4,
            int_type: SaveDataIntType::Arrayi32(MAZES_COUNT),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FruitsGetNum => SaveFileData {
            variable_name: "m_iFruitsGetNum".into(),
            variable_name_simple: "Fruits Get Number".into(),
            offset: 0x56C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CapsuleGetNum => SaveFileData {
            variable_name: "m_iCapsuleGetNum".into(),
            variable_name_simple: "Capsule Get Number".into(),
            offset: 0x570,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MedalGetNum => SaveFileData {
            variable_name: "m_iMedalGetNum".into(),
            variable_name_simple: "Medal Get Number".into(),
            offset: 0x574,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::BombDotKillNum => SaveFileData {
            variable_name: "m_iBombDotKillNum".into(),
            variable_name_simple: "Bomb Dot Kill Number".into(),
            offset: 0x578,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::DotKillNum => SaveFileData {
            variable_name: "m_iPacDotKillNum".into(),
            variable_name_simple: "PacDot Kill Number".into(),
            offset: 0x57C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::RevRollKillNum => SaveFileData {
            variable_name: "m_iPacDashKillNum".into(),
            variable_name_simple: "Rev Roll Kill Number".into(),
            offset: 0x580,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::ButtBounceKillNum => SaveFileData {
            variable_name: "m_iHipKillNum".into(),
            variable_name_simple: "Butt Bounce Kill Number".into(),
            offset: 0x584,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::GhostKillNum => SaveFileData {
            variable_name: "m_iGhostKillNum".into(),
            variable_name_simple: "Ghost Kill Number".into(),
            offset: 0x588,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MagicKeyUnlocked => SaveFileData {
            variable_name: "m_bGetMagicKey".into(),
            variable_name_simple: "Magic Key Unlocked".into(),
            offset: 0x58C,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::TrueEnding => SaveFileData {
            variable_name: "m_bAllTrueEnding".into(),
            variable_name_simple: "True Ending (unused?)".into(),
            offset: 0x590,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MarathonUnlocked => SaveFileData {
            variable_name: "m_bUnlockMarathon".into(),
            variable_name_simple: "Marathon Unlocked (unused?)".into(),
            offset: 0x594,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MarathonHighScore => SaveFileData {
            variable_name: "m_iMarathonHighScore".into(),
            variable_name_simple: "Marathon High Score (unused?)".into(),
            offset: 0x598,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MarathonClear => SaveFileData {
            variable_name: "m_iMarathonClearFlag".into(),
            variable_name_simple: "Marathon Cleared (unused?)".into(),
            offset: 0x59C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LastGISaveArea => SaveFileData {
            variable_name: "m_iLastGISaveArea".into(),
            variable_name_simple: "Last GI Save Area (unused?)".into(),
            offset: 0x5A0,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LoadInitScene => SaveFileData {
            variable_name: "m_iLoadInitScene".into(),
            variable_name_simple: "Load Init Scene".into(),
            offset: 0x5A4,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::DLCApplyFlag => SaveFileData {
            variable_name: "m_iDLCApplyFlag".into(),
            variable_name_simple: "DLC Apply Flag".into(),
            offset: 0x5A8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxBGM => SaveFileData {
            variable_name: "m_iJukeBoxBGMKind".into(),
            variable_name_simple: "Jukebox Music".into(),
            offset: 0x5AC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxMode => SaveFileData {
            variable_name: "m_iJukeBoxMode".into(),
            variable_name_simple: "Jukebox Mode".into(),
            offset: 0x5B0,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxLoop => SaveFileData {
            variable_name: "m_iJukeBoxLoop".into(),
            variable_name_simple: "Jukebox Loop".into(),
            offset: 0x5B4,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxOrder => SaveFileData {
            variable_name: "m_iJukeBoxOrder".into(),
            variable_name_simple: "Jukebox Order".into(),
            offset: 0x5B8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxFlag => SaveFileData {
            variable_name: "m_iJukeBoxFlag".into(),
            variable_name_simple: "Jukebox Flag".into(),
            offset: 0x5BC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxUnlockFlagList => SaveFileData {
            variable_name: "m_bJukeBoxUnlockFlagList".into(),
            variable_name_simple: "Jukebox Flag".into(),
            offset: 0x5C2, // TODO check the exact start of this, len should be fine since it's 83 songs
            int_type: SaveDataIntType::Arrayu8(83),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::MedalNum => SaveFileData {
            variable_name: "m_iMedalNum".into(),
            variable_name_simple: "Medals".into(),
            offset: 0x640,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        // TODO Capsule flag here
        SaveDataVar::StageCherryFlag => SaveFileData {
            variable_name: "m_iStageCherryFlag".into(),
            variable_name_simple: "Cherries Obtained Bitfield".into(),
            offset: 0x89C,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageStrawberryFlag => SaveFileData {
            variable_name: "m_iStageStrawberryFlag".into(),
            variable_name_simple: "Strawberries Obtained Bitfield".into(),
            offset: 0x9C8,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageOrangeFlag => SaveFileData {
            variable_name: "m_iStageOrangeFlag".into(),
            variable_name_simple: "Oranges Obtained Bitfield".into(),
            offset: 0xAF4,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageAppleFlag => SaveFileData {
            variable_name: "m_iStageAppleFlag".into(),
            variable_name_simple: "Apples Obtained Bitfield".into(),
            offset: 0xC20,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageMelonFlag => SaveFileData {
            variable_name: "m_iStageMelonFlag".into(),
            variable_name_simple: "Melons Obtained Bitfield".into(),
            offset: 0xD4C,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageCherryNum => SaveFileData {
            variable_name: "m_iStageCherryGetNum".into(),
            variable_name_simple: "Cherry Per Level".into(),
            offset: 0xE78,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageStrawberryNum => SaveFileData {
            variable_name: "m_iStageStrawberryGetNum".into(),
            variable_name_simple: "Strawberry Per Level".into(),
            offset: 0xFA4,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageOrangeNum => SaveFileData {
            variable_name: "m_iStageOrangeGetNum".into(),
            variable_name_simple: "Orange Per Level".into(),
            offset: 0x10D0,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageAppleNum => SaveFileData {
            variable_name: "m_iStageAppleGetNum".into(),
            variable_name_simple: "Apple Per Level".into(),
            offset: 0x1FFC,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageMelonNum => SaveFileData {
            variable_name: "m_iStageMelonGetNum".into(),
            variable_name_simple: "Melon Per Level".into(),
            offset: 0x1328,
            int_type: SaveDataIntType::Arrayi32(35),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageCherryFlag => SaveFileData {
            variable_name: "m_iVillageCherryFlag".into(),
            variable_name_simple: "Village Cherry Flags".into(),
            offset: 0x1454,
            int_type: SaveDataIntType::Arrayi32(26),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageStrawberryFlag => SaveFileData {
            variable_name: "m_iVillageStrawberryFlag".into(),
            variable_name_simple: "Village Strawberry Flags".into(),
            offset: 0x01610,
            int_type: SaveDataIntType::Arrayi32(29),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageOrangeFlag => SaveFileData {
            variable_name: "m_iVillageOrangeFlag".into(),
            variable_name_simple: "Village Orange Flags".into(),
            offset: 0x17CC,
            int_type: SaveDataIntType::Arrayi32(17),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageAppleFlag => SaveFileData {
            variable_name: "m_iVillageAppleFlag".into(),
            variable_name_simple: "Village Apple Flags".into(),
            offset: 0x1988,
            int_type: SaveDataIntType::Arrayi32(22),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageMelonFlag => SaveFileData {
            variable_name: "m_iVillageMelonFlag".into(),
            variable_name_simple: "Village Melon Flags".into(),
            offset: 0x1B44,
            int_type: SaveDataIntType::Arrayi32(32),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageCherryGetNum => SaveFileData {
            variable_name: "m_iVillageCherryGetNum".into(),
            variable_name_simple: "Village Cherries".into(),
            offset: 0x1D00,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageStrawberryGetNum => SaveFileData {
            variable_name: "m_iVillageStrawberryGetNum".into(),
            variable_name_simple: "Village Strawberry".into(),
            offset: 0x1D04,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageOrangeGetNum => SaveFileData {
            variable_name: "m_iVillageOrangeGetNum".into(),
            variable_name_simple: "Village Oranges".into(),
            offset: 0x1D08,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageAppleGetNum => SaveFileData {
            variable_name: "m_iVillageAppleGetNum".into(),
            variable_name_simple: "Village Apples".into(),
            offset: 0x1D0C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageMelonGetNum => SaveFileData {
            variable_name: "m_iVillageMelonGetNum".into(),
            variable_name_simple: "Village Melons".into(),
            offset: 0x1D10,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageGFFlag => SaveFileData {
            variable_name: "m_iVillageGFFlag".into(),
            variable_name_simple: "Village GF Bitfield".into(),
            offset: 0x1D14,
            int_type: SaveDataIntType::U32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraMode => SaveFileData {
            variable_name: "m_iCameraMode".into(),
            variable_name_simple: "Camera Mode".into(),
            offset: 0x1D18,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraSpeedY => SaveFileData {
            variable_name: "m_iCameraSpeedUD".into(),
            variable_name_simple: "Camera Sensitivity Y".into(),
            offset: 0x1D1C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraSpeedX => SaveFileData {
            variable_name: "m_iCameraSpeedLR".into(),
            variable_name_simple: "Camera Sensitivity X".into(),
            offset: 0x1D20,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraControlY => SaveFileData {
            variable_name: "m_iCameraControlUD".into(),
            variable_name_simple: "Reverse Vertical Camera".into(),
            offset: 0x1D24,
            int_type: SaveDataIntType::Bool, // game code says i32
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraControlX => SaveFileData {
            variable_name: "m_iCameraControlLR".into(),
            variable_name_simple: "Reverse Horizontal Camera".into(),
            offset: 0x1D28,
            int_type: SaveDataIntType::Bool, // game code says i32
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraAssistFlag => SaveFileData {
            variable_name: "m_iCameraAssistFlag".into(),
            variable_name_simple: "Disable Camera Assist".into(),
            offset: 0x1D2C,
            int_type: SaveDataIntType::Bool, // game code says i32
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::CameraYAutoRotateFlag => SaveFileData {
            variable_name: "m_iCameraYAutoRotFlag".into(),
            variable_name_simple: "Disable Player Tracking Camera".into(),
            offset: 0x1D30,
            int_type: SaveDataIntType::Bool, // game code says i32
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::SwimControlY => SaveFileData {
            variable_name: "m_iSwimControlUD".into(),
            variable_name_simple: "Reverse Vertical Swim Control (?)".into(),
            offset: 0x1D34,
            int_type: SaveDataIntType::Arrayi32(2),
            slot_base_add,
            var: req_data,
        },
        // TODO mission progress here
        SaveDataVar::MissionRewardFlag => SaveFileData {
            variable_name: "m_iMissionRewardFlag".into(),
            variable_name_simple: "Mission Reward Flags".into(),
            offset: 0x21EC,
            int_type: SaveDataIntType::Arrayi32(LEVELS_COUNT + 1),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayerSkinId => SaveFileData {
            variable_name: "m_iPlayerSkinId".into(),
            variable_name_simple: "Player Skin".into(),
            offset: 0x23E0,
            // game says it's an array of 2 but it's easier here to make this 2 separate vars
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayerSkinId2 => SaveFileData {
            variable_name: "m_iPlayerSkinId".into(),
            variable_name_simple: "Player Skin P2".into(),
            offset: 0x23E4,
            // game says it's an array of 2 but it's easier here to make this 2 separate vars
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FigureInfo => SaveFileData {
            variable_name: "m_sFigureInfo".into(),
            variable_name_simple: "Figure Info".into(),
            offset: 0x23E8,
            int_type: SaveDataIntType::Arrayi32(20), // TODO find length by filling the village
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FigureDisplayInfo => SaveFileData {
            variable_name: "m_sFigureDisplayInfo".into(),
            variable_name_simple: "Figure Display Info".into(),
            offset: 0x2BE8,
            int_type: SaveDataIntType::SFigureDisplayInfoArray(50),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::GashaFlag => SaveFileData {
            variable_name: "m_iGashaFlag".into(),
            variable_name_simple: "Gasha Flag List".into(),
            offset: 0x2D78,
            int_type: SaveDataIntType::Arrayi32(100),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::GashaLotteryNum => SaveFileData {
            variable_name: "m_iGashaLotteryNum".into(),
            variable_name_simple: "Gasha Lottery Number".into(),
            offset: 0x2DA8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FlipKillNum => SaveFileData {
            variable_name: "m_iFlipKillNum".into(),
            variable_name_simple: "Flip Kill Number".into(),
            offset: 0x2DAC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::SupperHipStunNum => SaveFileData {
            variable_name: "m_iSuperHipStunNum".into(),
            variable_name_simple: "Super Butt Bounce Stun Number".into(),
            offset: 0x2DB0,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::SuperDolphinKillNum => SaveFileData {
            variable_name: "m_iSuperDolphinKillNum".into(),
            variable_name_simple: "Super Dolphin Kick Kill Number".into(),
            offset: 0x2DB4,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::UnlockStageSelectFlag => SaveFileData {
            variable_name: "m_uUnlockStageSelectFlag".into(),
            variable_name_simple: "Unlock Stage Select Bitfield".into(),
            offset: 0x2DB8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::GameLevel => SaveFileData {
            variable_name: "m_iGameLevel".into(),
            variable_name_simple: "Game Level".into(),
            offset: 0x2DBC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::EnterPast => SaveFileData {
            variable_name: "m_bEnterPast".into(),
            variable_name_simple: "Enter Past".into(),
            offset: 0x2DC0,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::OriginalFlag => SaveFileData {
            variable_name: "m_iOriginalFlag".into(),
            variable_name_simple: "Pac-Man Flags".into(),
            offset: 0x2DC0,
            int_type: SaveDataIntType::Arrayi32(3),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::OriginalStageNum => SaveFileData {
            variable_name: "m_iOriginalStageNum".into(),
            variable_name_simple: "Pac-Man Best Stage".into(),
            offset: 0x2DD0,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::OriginalHighScore => SaveFileData {
            variable_name: "m_iOriginalHighScore".into(),
            variable_name_simple: "Pac-Man High Score".into(),
            offset: 0x2DD4,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PacManiaStageNum => SaveFileData {
            variable_name: "m_iPacManiaStageNum".into(),
            variable_name_simple: "Pac-Mania Best Stage".into(),
            offset: 0x2DD8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PacmaniaHighScore => SaveFileData {
            variable_name: "m_iPacManiaHighScore".into(),
            variable_name_simple: "Pac-Mania High Score".into(),
            offset: 0x2DDC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PacAttackLevelNum => SaveFileData {
            variable_name: "m_iPacAttackLevelNum".into(),
            variable_name_simple: "Pac-Attack Best Level".into(),
            offset: 0x2DE0,
            int_type: SaveDataIntType::Arrayi32(4),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PacAttackHighScore => SaveFileData {
            variable_name: "m_iPacAttackHighScore".into(),
            variable_name_simple: "Pac-Attack High Score".into(),
            offset: 0x2DF0,
            int_type: SaveDataIntType::Arrayi32(4),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::HelpFlag => SaveFileData {
            variable_name: "m_iHelpFlag".into(),
            variable_name_simple: "Help Flags".into(),
            offset: 0x2E00,
            int_type: SaveDataIntType::Arrayi32(4), // TODO find length
            slot_base_add,
            var: req_data,
        },
        // TODO drone skin flag
        // TODO dron sking ID
        SaveDataVar::DroneReticleSpeed => SaveFileData {
            variable_name: "m_iDroneReticleSpeed".into(),
            variable_name_simple: "Drone Reticle Speed".into(),
            offset: 0x2EF4,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::DroneReticleColor => SaveFileData {
            variable_name: "m_iDroneReticleColor".into(),
            variable_name_simple: "Drone Reticle Color".into(),
            offset: 0x2EF8,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::DroneVacuumRangeFlag => SaveFileData {
            variable_name: "m_iDroneVacuumRangeFlag".into(),
            variable_name_simple: "Drone Vacuum Range Flag".into(),
            offset: 0x2EFC,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::EarlyBonusFigureFlag => SaveFileData {
            variable_name: "m_iEarlyBonusFigureFlag".into(),
            variable_name_simple: "Early Access Bonus Figure".into(),
            offset: 0x2F00,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::KeyConfigP1 => SaveFileData {
            variable_name: "m_keyconfigSave1P".into(),
            variable_name_simple: "Inputs Config".into(),
            offset: 0x2F04,
            int_type: SaveDataIntType::Arrayi32(672),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::KeyConfigP2 => SaveFileData {
            variable_name: "m_keyconfigSave2P".into(),
            variable_name_simple: "Inputs Config Player 2".into(),
            offset: 0x3984,
            int_type: SaveDataIntType::Arrayi32(672),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::GashaDLCFlag => SaveFileData {
            variable_name: "m_iGashaDLCFlag".into(),
            variable_name_simple: "Gasha DLC Flag".into(),
            offset: 0x4404,
            int_type: SaveDataIntType::Arrayi32(4),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FigureInfoDLC => SaveFileData {
            variable_name: "m_sFigureInfoDLC".into(),
            variable_name_simple: "Figure Info DLC".into(),
            offset: 0x4414,
            int_type: SaveDataIntType::Arrayi32(10), // TODO find this length
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::EnterSonic => SaveFileData {
            variable_name: "m_bEnterSonic".into(),
            variable_name_simple: "Enter Sonic".into(),
            offset: 0x4454,
            int_type: SaveDataIntType::Bool,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::VillageSonicFlag => SaveFileData {
            variable_name: "m_uVillageSonicFlag".into(),
            variable_name_simple: "Village Sonic Flag".into(),
            offset: 0x4458,
            int_type: SaveDataIntType::U32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LoadInitSceneCollab => SaveFileData {
            variable_name: "m_iLoadInitSceneCollabo".into(),
            variable_name_simple: "Load Init Scene (Sonic)".into(),
            offset: 0x445C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LoadInitSceneKind => SaveFileData {
            variable_name: "m_iLoadInitSceneKind".into(),
            variable_name_simple: "Load Init Scene Kind".into(),
            offset: 0x4560,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LastStageIdCollab => SaveFileData {
            variable_name: "m_iLastStageIdCollabo".into(),
            variable_name_simple: "Last Stage ID (Sonic)".into(),
            offset: 0x4564,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::LastStageIdKindCollab => SaveFileData {
            variable_name: "m_iLastStageIdKind".into(),
            variable_name_simple: "Last Stage ID Kind (Sonic)".into(),
            offset: 0x4568,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::PlayerSkinIdCollab => SaveFileData {
            variable_name: "m_iPlayerSkinIdCollabo".into(),
            variable_name_simple: "Player Skin (Sonic)".into(),
            offset: 0x456C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FigureDisplayInfoCollab => SaveFileData {
            variable_name: "m_iFigureDisplayInfoCollabo".into(),
            variable_name_simple: "Figure Display Info (Sonic)".into(),
            offset: 0x4570,
            int_type: SaveDataIntType::Arrayi32(114),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxBGMCollab => SaveFileData {
            variable_name: "m_iJukeBoxBGMKindCollabo".into(),
            variable_name_simple: "Jukebox Music (Sonic)".into(),
            offset: 0x4638,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::JukeBoxModeCollab => SaveFileData {
            variable_name: "m_iJukeBoxModeCollabo".into(),
            variable_name_simple: "Jukebox Mode (Sonic)".into(),
            offset: 0x463C,
            int_type: SaveDataIntType::I32,
            slot_base_add,
            var: req_data,
        },
    }
}
