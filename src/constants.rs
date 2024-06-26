#[allow(dead_code)]
pub const ON: u16 = 1;
pub const OFF: u16 = 0;

// Consts to easily get the index of a given positions in a 3x3 array that's stored
// as an array. Implmetned as usize as they are used to lookup arrays.
pub const TOP_LFT: usize = 0;
pub const TOP_MID: usize = 1;
pub const TOP_RHT: usize = 2;
pub const MID_LFT: usize = 3;
pub const MID_MID: usize = 4;
pub const MID_RHT: usize = 5;
pub const BOT_LFT: usize = 6;
pub const BOT_MID: usize = 7;
pub const BOT_RHT: usize = 8;

pub const ARRAY_OF_9: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
