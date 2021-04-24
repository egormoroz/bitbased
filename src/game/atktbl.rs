use super::bitbrd::*;
use bitintr::{Pdep, Pext};


pub const ROOK_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

pub const BISHOP_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
];

pub struct AttackTable {
    // pawns: [[BitBoard; 64]; 2],
    knights: [BitBoard; 64],
    kings: [BitBoard; 64],
    bishop_mask: [BitBoard; 64],
    rook_mask: [BitBoard; 64],
    rooks: [[BitBoard; 1<<12]; 64],
    bishops: [[BitBoard; 1<<9]; 64],
}

impl AttackTable {
    /* 
    fn init_pawns(&mut self) {
        //white pawns
        for rank in 0..7 {
            self.pawns[WHITEX][rank * 8].set((rank as u8 + 1) * 8 + 1);
            for file in 1..7 {
                let b = &mut self.pawns[WHITEX][rank * 8 + file];
                b.set(((rank + 1) * 8 + file - 1) as u8);
                b.set(((rank + 1) * 8 + file + 1) as u8);
            }
            self.pawns[WHITEX][rank * 8 + 7].set((rank as u8 + 1) * 8 - 1);
        }
        //black pawns
        for rank in 1..8 {
            self.pawns[BLACKX][rank * 8].set((rank as u8 - 1) * 8 + 1);
            for file in 1..7 {
                let b = &mut self.pawns[BLACKX][rank * 8 + file];
                b.set(((rank - 1) * 8 + file - 1) as u8);
                b.set(((rank - 1) * 8 + file + 1) as u8);
            }
            self.pawns[BLACKX][rank * 8 + 7].set((rank as u8 - 1) * 8 + 6);
        }
    }
    */

    fn init_knights(&mut self) {
        for rank in 0..8 {
            for file in 0..8 {
                let from = (rank * 8 + file) as usize;
                let possibles = [
                    (rank + 2, file + 1), (rank + 1, file + 2),
                    (rank - 1, file + 2), (rank - 2, file + 1),
                    (rank - 2, file - 1), (rank - 1, file - 2),
                    (rank + 1, file - 2), (rank + 2, file - 1),
                ];
                for m in possibles.iter() {
                    if m.0 < 0 || m.0 > 7 || m.1 < 0 || m.1 > 7 { continue; }
                    let to = (m.0 * 8 + m.1) as u8;
                    self.knights[from].set(to);
                }
            }
        }
    }

    fn init_kings(&mut self) {
        for rank in 0..8 {
            for file in 0..8 {
                let from = (rank * 8 + file) as usize;
                let possibles = [
                    (rank + 1, file), (rank + 1, file + 1),
                    (rank, file + 1), (rank - 1, file + 1),
                    (rank - 1, file), (rank - 1, file - 1),
                    (rank, file - 1), (rank + 1, file - 1),
                ];
                for m in possibles.iter() {
                    if m.0 < 0 || m.0 > 7 || m.1 < 0 || m.1 > 7 { continue; }
                    let to = (m.0 * 8 + m.1) as u8;
                    self.kings[from].set(to);
                }
            }
        }
    }

    fn init_bishops(&mut self) {
        //blocker masks
        for rank in 0..8 {
            for file in 0..8 {
                let from = rank * 8 + file;
                let (mut r, mut f) = (rank as i8 + 1, file as i8 + 1);
                while r < 7 && f < 7 {
                    self.bishop_mask[from].set((r * 8 + f) as u8);
                    r += 1;
                    f += 1;
                }
                let (mut r, mut f) = (rank as i8 - 1, file as i8 - 1);
                while r > 0 && f > 0 {
                    self.bishop_mask[from].set((r * 8 + f) as u8);
                    r -= 1;
                    f -= 1;
                }
                let (mut r, mut f) = (rank as i8 + 1, file as i8 - 1);
                while r < 7 && f > 0 {
                    self.bishop_mask[from].set((r * 8 + f) as u8);
                    r += 1;
                    f -= 1;
                }
                let (mut r, mut f) = (rank as i8 - 1, file as i8 + 1);
                while r > 0 && f < 7 {
                    self.bishop_mask[from].set((r * 8 + f) as u8);
                    r -= 1;
                    f += 1;
                }
            }
        }

        //all possible blockers
        for sq in 0..64 {
            for blocker_idx in 0..1<<BISHOP_BITS[sq] {
                let blockers = (blocker_idx as u64).pdep(self.bishop_mask[sq]);
                let key = blockers.pext(self.bishop_mask[sq]);
                self.bishops[sq][key as usize] = Self::calc_bishop_attacks(sq, blockers);
            }
        }
    }

    pub fn calc_rook_attacks(sq: usize, blockers: BitBoard) -> BitBoard {
        let mut atks = 0;
        let (rank, file) = ((sq as u8)/8, (sq as u8)%8);

        for r in rank+1..8 {
            atks.set(r*8+file);
            if blockers.chk(r*8+file) { break; }
        }
        for r in (0..rank).rev() {
            atks.set(r*8+file);
            if blockers.chk(r*8+file) { break; }
        }
        for f in file+1..8 {
            atks.set(rank*8+f);
            if blockers.chk(rank*8+f) { break; }
        }
        for f in (0..file).rev() {
            atks.set(rank*8+f);
            if blockers.chk(rank*8+f) { break; }
        }

        atks
    }

    fn calc_bishop_attacks(sq: usize, blockers: BitBoard) -> BitBoard {
        let mut atks = 0;
        let (rank, file) = (sq / 8, sq % 8);
        let (mut r, mut f) = (rank as i8 + 1, file as i8 + 1);
        while r < 8 && f < 8  {
            atks.set((r * 8 + f) as u8);
            if blockers.chk((r*8+f) as u8) { break }
            r += 1;
            f += 1;
        }
        let (mut r, mut f) = (rank as i8 - 1, file as i8 - 1);
        while r >= 0 && f >= 0 {
            atks.set((r * 8 + f) as u8);
            if blockers.chk((r*8+f) as u8) { break }
            r -= 1;
            f -= 1;
        }
        let (mut r, mut f) = (rank as i8 + 1, file as i8 - 1);
        while r < 8 && f >= 0 {
            atks.set((r * 8 + f) as u8);
            if blockers.chk((r*8+f) as u8) { break }
            r += 1;
            f -= 1;
        }
        let (mut r, mut f) = (rank as i8 - 1, file as i8 + 1);
        while r >= 0 && f < 8 {
            atks.set((r * 8 + f) as u8);
            if blockers.chk((r*8+f) as u8) { break }
            r -= 1;
            f += 1;
        }

        atks
    }

    fn init_rooks(&mut self) {
        //blocker masks
        for rank in 0..8 {
            for file in 0..8 {
                let from = rank*8+file;
                for f in 1..7 {
                    if f != file {
                        self.rook_mask[from].set((rank*8+f) as u8);
                    }
                }
                for r in 1..7 {
                    if r != rank {
                        self.rook_mask[from].set((r*8+file) as u8);
                    }
                }
            }
        }

        //all possible blockers
        for sq in 0..64 {
            for blocker_idx in 0..1<<ROOK_BITS[sq] {
                let blockers = (blocker_idx as u64).pdep(self.rook_mask[sq]);
                let key = blockers.pext(self.rook_mask[sq]);
                self.rooks[sq][key as usize] = Self::calc_rook_attacks(sq, blockers);
            }
        }
    }

    pub fn new() -> Box<Self> {
        use std::alloc::{alloc, Layout};
        let layout = Layout::new::<AttackTable>();
        let mut inst = unsafe {
            let ptr = alloc(layout) as *mut AttackTable;
            Box::from_raw(ptr)
        };
        // inst.init_pawns();
        inst.init_knights();
        inst.init_kings();
        inst.init_bishops();
        inst.init_rooks();
        inst
    }
    pub fn knight_attacks(&self, sq: usize) -> BitBoard { self.knights[sq] }
    pub fn king_attacks(&self, sq: usize) -> BitBoard { self.kings[sq] }

    pub fn rook_attacks(&self, sq: usize, mut blockers: BitBoard) -> BitBoard {
        blockers &= self.rook_mask[sq];
        let key = blockers.pext(self.rook_mask[sq]);
        self.rooks[sq][key as usize]
    }

    pub fn bishop_attacks(&self, sq: usize, mut blockers: BitBoard) -> BitBoard {
        blockers &= self.bishop_mask[sq];
        let key = blockers.pext(self.bishop_mask[sq]);
        self.bishops[sq][key as usize]
    }

    pub fn queen_attacks(&self, sq: usize, blockers: BitBoard) -> BitBoard {
        self.bishop_attacks(sq, blockers) | self.rook_attacks(sq, blockers)
    }
}
