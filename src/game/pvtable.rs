use super::movgen::*;
use super::pos::*;
use std::collections::HashMap;

// pub const MAX_ENTRIES: u64 = 1024*1024;
pub const MAX_DEPTH: usize = 64;

pub struct PVTable {
    // entries: Vec<(u64, Move)>,
    entries: HashMap<u64, Move>,
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::with_capacity(1024*32),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn store(&mut self, pk: u64, m: Move) {
        self.entries.insert(pk, m);
    }

    pub fn probe(&self, pk: u64) -> Option<Move> {
        self.entries.get(&pk).and_then(|m|Some(*m))
    }
}

pub struct PVLine {
    data: [Move; MAX_DEPTH],
    n: usize,
}

impl PVLine {
    pub fn new() -> Self {
        Self {
            data: [Move::new(); MAX_DEPTH as usize],
            n: 0,
        }
    }

    pub fn push(&mut self, m: Move) {
        debug_assert!(self.n < MAX_DEPTH);
        self.data[self.n] = m;
        self.n += 1;
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.data[0..self.n].iter()
    }

    pub fn clear(&mut self) { 
        self.n = 0;
    }

    pub fn len(&self) -> usize { self.n }
}

impl Position {
    fn move_exists(&mut self, m: Move) -> bool {
        let mut moves = MoveList::new();
        self.gen_moves::<false>(&mut moves);
        for om in moves.iter() {
            if self.make_move(m) {
                self.unmake_move();
                if om.0 == m { return true }
            }
        }
        false
    }

    pub fn extract_pv_line(&mut self, depth: u8) {
        debug_assert!((depth as usize) < MAX_DEPTH);
        self.pv_line.clear();
        while let Some(m) = self.pv_table.probe(self.key) {
            if !self.move_exists(m) { break }
            self.make_move(m);
            self.pv_line.push(m);
        }

        for _ in 0..self.pv_line.n {
            self.unmake_move();
        }
    }

    pub fn store_pv_move(&mut self, m: Move) {
        self.pv_table.store(self.key, m);
    }
}
