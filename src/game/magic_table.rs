use super::bitbrd::*;
use bitintr::Pdep;
use super::magics::*;


pub struct AttackTable {
    knights: [BitBoard; 64],
    kings: [BitBoard; 64],
    attacks: [BitBoard; 88772],
}

fn calc_rook_attacks(sq: usize, blockers: BitBoard) -> BitBoard {
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

impl AttackTable {
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



    fn init_rooks(&mut self) {
        for sq in 0..64 {
            let m = &ROOK_MAGICS[sq];
            for blocker_idx in 0..1<<12 {
                let blockers = blocker_idx.pdep(m.mask);
                let attack = calc_rook_attacks(sq, blockers);
                let idx = (m.factor.wrapping_mul(blockers) >> (64-12)) as usize + m.offset;
                assert!(self.attacks[idx] == 0 || self.attacks[idx] == attack);
                self.attacks[idx] = attack;
            }
        }
    }

    fn init_bishops(&mut self) {
        for sq in 0..64 {
            let m = &BISHOP_MAGICS[sq];
            for blocker_idx in 0..1<<9 {
                let blockers = blocker_idx.pdep(m.mask);
                let attack = calc_bishop_attacks(sq, blockers);
                let idx = (m.factor.wrapping_mul(blockers) >> (64-9)) as usize + m.offset;
                assert!(self.attacks[idx] == 0 || self.attacks[idx] == attack);
                self.attacks[idx] = attack;
            }
        }
    }

    pub fn new() -> Box<Self> {
        use std::alloc::{alloc, Layout};
        let layout = Layout::new::<AttackTable>();
        let mut inst = unsafe {
            let ptr = alloc(layout);
            std::ptr::write_bytes(ptr, 0, layout.size());
            Box::from_raw(ptr as *mut AttackTable)
        };
        inst.init_rooks();
        inst.init_bishops();
        inst.init_knights();
        inst.init_kings();

        inst
    }

    pub fn rook_attacks(&self, sq: usize, blockers: BitBoard) -> BitBoard {
        let m = &ROOK_MAGICS[sq];
        let idx = (m.factor.wrapping_mul(blockers & m.mask) >> (64-12)) as usize + m.offset;
        self.attacks[idx]
    }

    pub fn bishop_attacks(&self, sq: usize, blockers: BitBoard) -> BitBoard {
        let m = &BISHOP_MAGICS[sq];
        let idx = (m.factor.wrapping_mul(blockers & m.mask) >> (64-9)) as usize + m.offset;
        self.attacks[idx]
    }

    pub fn knight_attacks(&self, sq: usize) -> BitBoard { self.knights[sq] }
    pub fn king_attacks(&self, sq: usize) -> BitBoard { self.kings[sq] }
}

// #[test]
// fn create_magics() {
    // AttackTable::new();
// }
