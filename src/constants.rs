pub const ON: u16 = 1;
pub const OFF: u16 = 0;

// Consts to easily get the index of a given positions in a 3x3 array that's stored
// as an array. Implmetned as usize as they are used to lookup arrays.
#[allow(dead_code)]
pub const TOP_LFT: usize = 0;
#[allow(dead_code)]
pub const TOP_MID: usize = 1;
#[allow(dead_code)]
pub const TOP_RHT: usize = 2;
#[allow(dead_code)]
pub const MID_LFT: usize = 3;
#[allow(dead_code)]
pub const MID_MID: usize = 4;
#[allow(dead_code)]
pub const MID_RHT: usize = 5;
#[allow(dead_code)]
pub const BOT_LFT: usize = 6;
#[allow(dead_code)]
pub const BOT_MID: usize = 7;
#[allow(dead_code)]
pub const BOT_RHT: usize = 8;

#[allow(dead_code)]
pub const ARRAY_OF_9: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
