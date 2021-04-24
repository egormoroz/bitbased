use lazy_static::lazy_static;


use super::{bitbrd::*, defs::*, pos::*, magic_table::*};
use std::fmt;

lazy_static! {
    pub static ref ATTK_TBL: Box<AttackTable> = AttackTable::new();
}

pub struct KindBits(u8);

impl KindBits {
    pub fn castle(&self) -> bool { self.0 & 1 != 0 }
    pub fn en_passant(&self) -> bool { self.0 & 2 != 0 }
    pub fn long_push(&self) -> bool { self.0 & 4 != 0 }
}

#[derive(Clone, Copy)]
pub struct Move(u32);

impl Move {
    pub fn new() -> Self { Self(0) }

    pub fn new_long(from: u8, to: u8) -> Self {
        Self( 4 << 16
            | (to as u32) << 8
            | (from as u32))
    }

    pub fn new_usual(from: u8, to: u8, cap: bool) -> Self {
        Self((cap as u32) << 31 | (to as u32) << 8 | (from as u32))
    }

    pub fn new_prom(from: u8, to: u8, cap: bool, tp: u8) -> Self {
        Self( (cap as u32) << 31 
            | (tp as u32) << 24
            | (to as u32) << 8
            | (from as u32))
    }

    pub fn new_castle(from: u8, to: u8) -> Self {
        Self(1 << 16 | (to as u32) << 8 | (from as u32))
    }

    pub fn new_enpassant(from: u8, to: u8) -> Self {
        Self( 1 << 31 
            | 2 << 16
            | (to as u32) << 8
            | (from as u32))
    }

    pub fn from(&self) -> u8 { (self.0 & (0xFF)) as u8 }
    pub fn to(&self) -> u8 { ((self.0 >> 8) & 0xFF) as u8 }
    pub fn kind(&self) -> KindBits {
        KindBits(((self.0 >> 16) & 0xFF) as u8)
    }
    pub fn prom(&self) -> u8 { (self.0 >> 24 & 0b111) as u8 }
    pub fn cap(&self) -> bool { self.0 >> 31 != 0 }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            ('a' as u8 + self.from()%8) as char,
            ('1' as u8 + self.from()/8) as char,
            ('a' as u8 + self.to()%8) as char,
            ('1' as u8 + self.to()/8) as char,
        )?;
        write!(f, "{}", match self.prom() {
            KNIGHT => "n",
            BISHOP => "b",
            ROOK => "r",
            QUEEN => "q",
            _ => "",
        })
        // write!(f, "{}", match self.cap() {
            // true => "x",
            // false => "",
        // })
    }
}


pub struct MoveList {
    moves: [Move; MAX_MOVES],
    n: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [Move(0); 256],
            n: 0,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.moves[0..self.n].iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Move> {
        self.moves[0..self.n].iter_mut()
    }

    pub fn push(&mut self, m: Move) {
        self.moves[self.n] = m;
        self.n += 1;
    }

    pub fn clear(&mut self) { self.n = 0; }

    pub fn len(&self) -> usize { self.n }

    pub fn get(&self, idx: usize) -> &Move { 
        debug_assert!(idx < self.n);
        &self.moves[idx]
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut Move {
        debug_assert!(idx < self.n);
        &mut self.moves[idx]
    }
}

fn knight_attacks(knights: BitBoard) -> BitBoard {
   let l1 = (knights >> 1) & 0x7f7f7f7f7f7f7f7f;
   let l2 = (knights >> 2) & 0x3f3f3f3f3f3f3f3f;
   let r1 = (knights << 1) & 0xfefefefefefefefe;
   let r2 = (knights << 2) & 0xfcfcfcfcfcfcfcfc;
   let h1 = l1 | r1;
   let h2 = l2 | r2;
   return (h1<<16) | (h1>>16) | (h2<<8) | (h2>>8);
}

impl Position {
    fn gen_proms<const CAP: bool>(&self, from: u8, to: u8, moves: &mut MoveList) {
        for tp in KNIGHT..=QUEEN {
            moves.push(Move::new_prom(from, to, CAP, tp))
        }
    }

    fn gen_pawn_moves<const TURN: u8>(&self, moves: &mut MoveList) {
        let brd = self.pieces[self.turn as usize][PAWNX];
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        let (r7, notr7) = (brd & RANK_7, brd & !RANK_7);
        let (r2, notr2) = (brd & RANK_2, brd & !RANK_2);
        let free = !self.all_ocupied();

        if TURN == WHITE {
            let proms = (r7 << 8) & free;
            let promcaps7 = (r7 & !FILE_A) << 7 & enemies;
            let promcaps9 = (r7 & !FILE_H) << 9 & enemies;
            let shorts = (notr7 << 8) & free;
            let longs = (((r2 << 8) & free) << 8) & free;
            let caps7 = (notr7 & !FILE_A) << 7 & enemies;
            let caps9 = (notr7 & !FILE_H) << 9 & enemies;

            for sq in proms.bits() {
                self.gen_proms::<false>(sq - 8, sq, moves);
            }
            for sq in promcaps7.bits() {
                self.gen_proms::<true>(sq - 7, sq, moves);
            }
            for sq in promcaps9.bits() {
                self.gen_proms::<true>(sq - 9, sq, moves);
            }
            for sq in shorts.bits() {
                moves.push(Move::new_usual(sq - 8, sq, false))
            }
            for sq in longs.bits() {
                moves.push(Move::new_long(sq - 16, sq));
            }
            for sq in caps7.bits() {
                moves.push(Move::new_usual(sq - 7, sq, true))
            }
            for sq in caps9.bits() {
                moves.push(Move::new_usual(sq - 9, sq, true))
            }

            if self.ep != NS {
                let b = ((1<<self.ep & !FILE_A) >> 9 | (1<<self.ep & !FILE_H) >> 7) & brd;
                for sq in b.bits() {
                    moves.push(Move::new_enpassant(sq, self.ep));
                }
            }
        } else {
            let proms = (r2 >> 8) & free;
            let promcaps7 = (r2 & !FILE_H) >> 7 & enemies;
            let promcaps9 = (r2 & !FILE_A) >> 9 & enemies;
            let shorts = (notr2 >> 8) & free;
            let longs = ((r7 >> 8) & free) >> 8 & free;
            let caps7 = (notr2 & !FILE_H) >> 7 & enemies;
            let caps9 = (notr2 & !FILE_A) >> 9 & enemies;

            for sq in proms.bits() {
                self.gen_proms::<false>(sq + 8, sq, moves);
            }
            for sq in promcaps7.bits() {
                self.gen_proms::<true>(sq + 7, sq, moves);
            }
            for sq in promcaps9.bits() {
                self.gen_proms::<true>(sq + 9, sq, moves);
            }
            for sq in shorts.bits() {
                moves.push(Move::new_usual(sq + 8, sq, false))
            }
            for sq in longs.bits() {
                moves.push(Move::new_long(sq + 16, sq));
            }
            for sq in caps7.bits() {
                moves.push(Move::new_usual(sq + 7, sq, true))
            }
            for sq in caps9.bits() {
                moves.push(Move::new_usual(sq + 9, sq, true))
            }
            if self.ep != NS {
                let b = ((1<<self.ep & !FILE_H) << 9 | (1<<self.ep & !FILE_A) << 7) & brd;
                for sq in b.bits() {
                    moves.push(Move::new_enpassant(sq, self.ep));
                }
            }
        }
    }

    pub fn gen_king_moves(&self, moves: &mut MoveList) {
        let free  = !self.all_ocupied();
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        for from in self.pieces[self.turn as usize][KINGX].bits() {
            let bb = ATTK_TBL.king_attacks(from as usize);
            for to in (bb & enemies).bits() {
                moves.push(Move::new_usual(from, to, true));
            }
            for to in (bb & free).bits() {
                moves.push(Move::new_usual(from, to, false));
            }
        }
    }

    pub fn gen_knight_moves(&self, moves: &mut MoveList) {
        let free  = !self.all_ocupied();
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        for from in self.pieces[self.turn as usize][KNIGHTX].bits() {
            let bb = ATTK_TBL.knight_attacks(from as usize);
            for to in (bb & enemies).bits() {
                moves.push(Move::new_usual(from, to, true));
            }
            for to in (bb & free).bits() {
                moves.push(Move::new_usual(from, to, false));
            }
        }
    }

    pub fn gen_bishop_moves(&self, moves: &mut MoveList) {
        let blockers = self.all_ocupied();
        let free  = !blockers;
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        for from in self.pieces[self.turn as usize][BISHOPX].bits() {
            let bb = ATTK_TBL.bishop_attacks(from as usize, blockers);
            for to in (bb & enemies).bits() {
                moves.push(Move::new_usual(from, to, true));
            }
            for to in (bb & free).bits() {
                moves.push(Move::new_usual(from, to, false));
            }
        }
    }

    pub fn gen_rook_moves(&self, moves: &mut MoveList) {
        let blockers = self.all_ocupied();
        let free  = !blockers;
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        for from in self.pieces[self.turn as usize][ROOKX].bits() {
            let bb = ATTK_TBL.rook_attacks(from as usize, blockers);
            for to in (bb & enemies).bits() {
                moves.push(Move::new_usual(from, to, true));
            }
            for to in (bb & free).bits() {
                moves.push(Move::new_usual(from, to, false));
            }
        }
    }


    pub fn gen_queen_moves(&self, moves: &mut MoveList) {
        let blockers = self.all_ocupied();
        let free  = !blockers;
        let enemies = self.occupied[(self.turn ^ 1) as usize];
        for from in self.pieces[self.turn as usize][QUEENX].bits() {
            let bb = ATTK_TBL.bishop_attacks(from as usize, blockers)
                | ATTK_TBL.rook_attacks(from as usize, blockers);
            for to in (bb & enemies).bits() {
                moves.push(Move::new_usual(from, to, true));
            }
            for to in (bb & free).bits() {
                moves.push(Move::new_usual(from, to, false));
            }
        }
    }

    fn gen_castling_moves<const TURN: u8>(&self, moves: &mut MoveList) {
        const KING_MASK: [u64; 2] = [ 3 << 5, 3 << 61 ]; //TODO: Check this!
        const QUEEN_MASK: [u64; 2] = [ 7 << 1, 7 << 57 ]; //and this!
        const KING_MOVE: [(u8, u8); 2] = [ (4, 6), (60, 62) ];
        const QUEEN_MOVE: [(u8, u8); 2] = [ (4, 2), (60, 58) ];
        let b = self.all_ocupied();
        let (f, tk) = KING_MOVE[TURN as usize];
        let tq = QUEEN_MOVE[TURN as usize].1;
        if self.cas.king(TURN) && b & KING_MASK[TURN as usize] == 0 {
            moves.push(Move::new_castle(f, tk))
        }
        if self.cas.queen(TURN) && b & QUEEN_MASK[TURN as usize] == 0 {
            moves.push(Move::new_castle(f, tq))
        }
    }


    pub fn gen_moves(&self, moves: &mut MoveList) {
        self.gen_knight_moves(moves);
        self.gen_bishop_moves(moves);
        self.gen_rook_moves(moves);
        self.gen_queen_moves(moves);
        self.gen_king_moves(moves);
        if self.turn == WHITE {
            self.gen_pawn_moves::<WHITE>(moves);
            self.gen_castling_moves::<WHITE>(moves);
        } else {
            self.gen_pawn_moves::<BLACK>(moves);
            self.gen_castling_moves::<BLACK>(moves);
        }
    }

    // #[inline(never)]
    pub fn in_check(&self, us: usize) -> bool {
        let them = us ^ 1;
        let mask = self.pieces[us][KINGX];
        let sq = mask.trailing_zeros() as usize;
        if them == WHITEX {
            let pwns = self.pieces[WHITEX][PAWNX];
            if ((pwns & !FILE_A) << 7 | (pwns & !FILE_H) << 9) & mask != 0 { return true }
        } else {
            let pwns = self.pieces[BLACKX][PAWNX];
            if ((pwns & !FILE_A) >> 9 | (pwns & !FILE_H) >> 7) & mask != 0 { return true }
        }

        // if knight_attacks(self.pieces[them][KNIGHTX]) & mask != 0 { return true }
        
        for sq in self.pieces[them][KNIGHTX].bits() {
            if ATTK_TBL.knight_attacks(sq as usize) & mask != 0 { return true }
        }
        for sq in self.pieces[them][KINGX].bits() {
            if ATTK_TBL.king_attacks(sq as usize) & mask != 0 { return true }
        }

        let blockers = self.all_ocupied();
        let bp = ATTK_TBL.bishop_attacks(sq, blockers);
        let rk = ATTK_TBL.rook_attacks(sq, blockers);
        (bp & self.pieces[them][BISHOPX] | rk & self.pieces[them][ROOKX]
        | ((bp | rk) & self.pieces[them][QUEENX])) != 0
    }
/* 
    pub fn sqattacked(&self, sidex: usize, sq: u8) -> bool {
        let mask = 1<<sq;
        if sidex == WHITEX {
            let pwns = self.pieces[WHITEX][PAWNX];
            if ((pwns & !FILE_A) << 7 | (pwns & !FILE_H) << 9) & mask != 0 { return true }
        } else {
            let pwns = self.pieces[BLACKX][PAWNX];
            if (pwns & !FILE_A) >> 9 | (pwns & !FILE_H) >> 7 != 0 { return true }
        }

        for sq in self.pieces[sidex][KNIGHTX].bits() {
            if ATTK_TBL.knight_attacks(sq as usize) & mask != 0 { return true }
        }
        for sq in self.pieces[sidex][KINGX].bits() {
            if ATTK_TBL.king_attacks(sq as usize) & mask != 0 { return true }
        }
        let blockers = self.all_ocupied();
        for sq in self.pieces[sidex][BISHOPX].bits() {
            if ATTK_TBL.bishop_attacks(sq as usize, blockers) & mask != 0 { return true }
        }
        for sq in self.pieces[sidex][ROOKX].bits() {
            if ATTK_TBL.rook_attacks(sq as usize, blockers) & mask != 0 { return true }
        }
        for sq in self.pieces[sidex][QUEENX].bits() {
            if (ATTK_TBL.bishop_attacks(sq as usize, blockers) 
                | ATTK_TBL.rook_attacks(sq as usize, blockers)) & mask != 0 { return true }
        }
        false
    }
*/

    pub fn attacked(&self, sidex: usize) -> BitBoard {
        let mut bb = 0;
        if sidex == WHITEX {
            let pwns = self.pieces[WHITEX][PAWNX];
            bb |=(pwns & !FILE_A) << 7 | (pwns & !FILE_H) << 9 ;
        } else {
            let pwns = self.pieces[BLACKX][PAWNX];
            bb |= (pwns & !FILE_A) >> 9 | (pwns & !FILE_H) >> 7;
        }

        for sq in self.pieces[sidex][KNIGHTX].bits() {
            bb |= ATTK_TBL.knight_attacks(sq as usize);
        }
        for sq in self.pieces[sidex][KINGX].bits() {
            bb |= ATTK_TBL.king_attacks(sq as usize);
        }
        let blockers = self.all_ocupied();
        for sq in self.pieces[sidex][BISHOPX].bits() {
            bb |= ATTK_TBL.bishop_attacks(sq as usize, blockers);
        }
        for sq in self.pieces[sidex][ROOKX].bits() {
            bb |= ATTK_TBL.rook_attacks(sq as usize, blockers);
        }
        for sq in self.pieces[sidex][QUEENX].bits() {
            bb |= ATTK_TBL.bishop_attacks(sq as usize, blockers);
            bb |= ATTK_TBL.rook_attacks(sq as usize, blockers);
        }

        bb
    }
}
