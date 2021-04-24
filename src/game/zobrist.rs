use crate::{rand, lazy_static::lazy_static};
use super::{pos::*, defs::*};

lazy_static! {
    pub static ref ZOBRIST: Zobrist = Zobrist::new();
}

pub struct Zobrist {
    pckeys: [[u64; 64]; 13],
    cskeys: [u64; 16],
    skey: u64,
    no_ep: u64,
}

impl Zobrist {
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut inst = Zobrist {
            pckeys: [[0; 64]; 13],
            cskeys: [0; 16],
            skey: rng.gen(),
            no_ep: rng.gen()
        };
        for i in inst.pckeys.iter_mut() {
            for j in i.iter_mut() {
                *j = rng.gen();
            }
        }
        for i in inst.cskeys.iter_mut() {
            *i = rng.gen();
        }
        inst
    }

    pub fn piece(&self, sq: u8, p: Piece) -> u64 {
        self.pckeys[p.id() as usize][sq as usize]
    }

    pub fn side(&self) -> u64 { self.skey }

    pub fn castling(&self, cs: CastlingPerm) -> u64 {
        self.cskeys[cs.id() as usize]
    }

    pub fn en_passant(&self, ep: u8) -> u64 {
        match ep {
            NS => self.no_ep,
            ep => self.pckeys[12][ep as usize]
        }
    }

    pub fn gen_key(&self, b: &Position) -> u64 {
        let mut k = 0;
        for (sq, p) in b.board.iter().enumerate() {
            if p.is_none() { continue; }
            k ^= self.piece(sq as u8, *p);
        }
        if b.turn == WHITE { k ^= self.skey; }
        k ^= self.en_passant(b.ep); 
        k ^= self.cskeys[b.cas.id() as usize];
        k
    }
}
