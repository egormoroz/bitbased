pub const FILEC: [char; 8] = [ 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h' ];
pub const RANKC: [char; 8] = [ '1', '2', '3', '4', '5', '6', '7', '8' ];

pub const PAWN: u8 = 0;
pub const PAWNX: usize = 0;
pub const KNIGHT: u8 = 1;
pub const KNIGHTX: usize = 1;
pub const BISHOP: u8 = 2;
pub const BISHOPX: usize = 2;
pub const ROOK: u8 = 3;
pub const ROOKX: usize = 3;
pub const QUEEN: u8 = 4;
pub const QUEENX: usize = 4;
pub const KING: u8 = 5;
pub const KINGX: usize = 5;

pub const WHITE: u8 = 0;
pub const WHITEX: usize = 0;
pub const BLACK: u8 = 1;
pub const BLACKX: usize = 1;


pub const RANK_2: u64 = 0xFF << 8;
pub const RANK_4: u64 = 0xFF << 24;
pub const RANK_5: u64 = 0xFF << 32;
pub const RANK_7: u64 = 0xFF << 48;

pub const FILE_H: u64 = 0x8080808080808080;
pub const FILE_A: u64 = 0x101010101010101;
pub const FILE_B: u64 = 0x4040404040404040;
pub const FILE_G: u64 = 0x101010101010101;

pub const PAWN_LOOKUP: bool = false;
pub const KNIGHT_LOOKUP: bool = false;
pub const KING_LOOKUP: bool = false;

pub const MAX_HIST: usize = 256;
pub const MAX_MOVES: usize = 256;
pub const NS: u8 = 255;

pub fn piece_id(tp: u8, clr: u8) -> u8 {
    tp << 1 | clr
}