#[derive(Debug)]
pub enum SaveDataVar {
    Lives,
}

#[derive(Debug)]
pub enum SaveDataIntType {
    U32,
    I32,
    Arrayi32(u32), // stores its length
}

#[derive(Debug)]
pub struct SaveFileData {
    pub var: SaveDataVar,
    pub variable_name: String,
    pub variable_name_simple: String,
    pub slot_base_add: u32,
    pub offset: u32,
    pub int_type: SaveDataIntType,
}

pub fn get_save_slot_base_add(slot: u8) -> u32 {
    match slot {
        1 => 0x298,
        2 => 0x7BC8,
        3 => 0xF4F8,
        4 => 0x16E28,
        _ => {
            println!(
                "Invalid slot sent! Using slot 1. Slot in function: {}",
                slot
            );
            0x298
        }
    }
}