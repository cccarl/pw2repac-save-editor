use std::{env, fs, str::from_utf8};
use enum_iterator::all;

use crate::save_data_info::{SaveDataIntType, SaveDataVar, SaveFileData, get_save_slot_base_add};

const LEVELS_COUNT: u32 = 40;
const MAZES_COUNT: u32 = 15;

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
        SaveDataIntType::Arrayi32(_) | SaveDataIntType::Arrayu32(_) | SaveDataIntType::ArrayText(_) | SaveDataIntType::Arrayu8(_) => {
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
            int_type: SaveDataIntType::Arrayi32(60), // TODO if this really is 60 and not 40 hmmm
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
            offset: 0x404,
            int_type: SaveDataIntType::Arrayu8(9),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::FriendsTalkList => SaveFileData {
            variable_name: "m_uFriendTalkList".into(),
            variable_name_simple: "Friends Talk List".into(),
            offset: 0x414,
            int_type: SaveDataIntType::Arrayu32(34),
            slot_base_add,
            var: req_data,
        },
        SaveDataVar::StageMazeFlagList => SaveFileData {
            variable_name: "m_bStageMazeFlagList".into(),
            variable_name_simple: "Stage Maze Flag".into(),
            offset: 0x450,
            int_type: SaveDataIntType::Arrayu32(MAZES_COUNT),
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
    }
}
