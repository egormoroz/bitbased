use super::movgen::*;
use super::pos::*;
// use std::{collections::HashMap, hash::Hash};

pub const NUM_ENTRIES: u64 = 10*1024*1024;
pub const MAX_DEPTH: usize = 64;

#[derive(Clone, Copy)]
pub enum EntryFlags {
    None,
    Alpha,
    Beta,
    Exact,
}

#[derive(Clone, Copy)]
pub struct HashEntry {
    pub m: Move,
    pub score: i16,
    pub depth: u8,
    pub flags: EntryFlags,
}

impl HashEntry {
    fn new() -> Self {
        Self {
            m: Move::new(),
            score: 0,
            depth: 0,
            flags: EntryFlags::None,
        }
    }
}

pub struct HashTable {
    entries: Vec<(u64, HashEntry)>,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            entries: vec![(0, HashEntry::new()); NUM_ENTRIES as usize],
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn store(&mut self, pk: u64, e: HashEntry) {
        self.entries[(pk % NUM_ENTRIES) as usize] = (pk, e);
        // self.entries.insert(pk, e);
    }

    pub fn probe(&self, pk: u64) -> Option<HashEntry> {
        let e = &self.entries[(pk % NUM_ENTRIES) as usize];
        if e.0 == pk { Some(e.1) }
        else { None }
        // self.entries.get(&pk).and_then(|m|Some(*m))
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

    pub fn extract_pv_line(&mut self, mut depth: u8) {
        debug_assert!((depth as usize) < MAX_DEPTH);
        self.pv_line.clear();
        while let Some(e) = self.pv_table.probe(self.key) {
            if !self.move_exists(e.m) { break }
            self.make_move(e.m);
            self.pv_line.push(e.m);
            if depth <= 1 { break }
            depth -= 1;
        }

        for _ in 0..self.pv_line.n {
            self.unmake_move();
        }
    }
}
