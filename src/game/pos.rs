use super::{bitbrd::*, defs::*, movgen::*, zobrist::ZOBRIST};
use super::pvtable::*;
use std::{fmt, str::FromStr};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece(u8);

impl Piece {
    pub fn none() -> Self { Self(0xFF) }

    pub fn new(tp: u8, clr: u8) -> Self {
        Self(tp << 1 | clr)
    }

    pub fn get_type(&self) -> u8 { self.0 >> 1 }
    pub fn get_color(&self) -> u8 { self.0 & 1 }
    pub fn id(&self) -> u8 { self.0 }

    pub fn is_some(&self) -> bool { self.0 != 0xFF }
    pub fn is_none(&self) -> bool { self.0 == 0xFF }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?} {:?})", self.get_color(), self.get_type())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingPerm(u8);

impl CastlingPerm {
    pub fn new() -> Self { Self(0) }
    pub fn king(&self, c: u8) -> bool {
        self.0 & (1 << c) != 0
    }

    pub fn queen(&self, c: u8) -> bool {
        self.0 & (1 << (2 + c)) != 0
    }

    pub fn dis_king(&mut self, c: u8) {
        self.0 &= !(1 << c);
    }

    pub fn dis_queen(&mut self, c: u8) {
        self.0 &= !(1 << (2 + c));
    }

    pub fn dis_both(&mut self, c: u8) {
        self.dis_king(c);
        self.dis_queen(c)
    }

    pub fn any(&self, c: u8) -> bool {
        self.king(c) || self.queen(c)
    }

    pub fn id(&self) -> u8 { self.0 }
}

impl FromStr for CastlingPerm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut inst = Self::new();
        if s == "-" { return Ok(inst) }
        for i in s.chars() {
            match i {
                'K' => inst.0 |= 0b1,
                'k' => inst.0 |= 0b10,
                'Q' => inst.0 |= 0b100,
                'q' => inst.0 |= 0b1000,
                _ => return Err(())
            }
        }
        Ok(inst)
    }
}

impl fmt::Display for CastlingPerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.king(WHITE) { write!(f, "K")? }
        if self.queen(WHITE) { write!(f, "Q")? }
        if self.king(BLACK) { write!(f, "k")? }
        if self.queen(BLACK) { write!(f, "q")? }
        if self.0 == 0 { write!(f, "-")? }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Hist {
    pub m: Move,
    pub cas: CastlingPerm,
    pub cap: Piece,
    pub ep: u8,
    pub fty: u8,
    pub key: u64,
}

impl Hist {
    pub fn new() -> Self {
        Self {
            m: Move::new(),
            cas: CastlingPerm::new(),
            cap: Piece::none(),
            ep: 255,
            fty: 0,
            key: 0,
        }
    }
}

pub struct Position {
    pub board: [Piece; 64],
    pub pieces: [[BitBoard; 6]; 2],
    pub occupied: [BitBoard; 2],

    pub hist: [Hist; MAX_HIST],
    pub hist_ply: u16,
    pub ep: u8,
    pub turn: u8,
    pub cas: CastlingPerm,
    pub key: u64,

    pub fty: u8,
    pub ply: u16,
    pub pv_line: PVLine,
    pub pv_table: HashTable,

    pub search_hist: [[u16; 64]; 12],
    pub search_killers: [[Move; MAX_DEPTH]; 2],
    pub counter_moves: [[Move; 64]; 64],

    pub material: [i16; 2],
}

impl Position {
    pub fn new() -> Self {
        Self {
            board: [Piece::none(); 64],
            pieces: [[0; 6]; 2],
            occupied: [0; 2],
            
            hist: [Hist::new(); MAX_HIST],
            hist_ply: 0,
            ep: NS,
            turn: WHITE,
            cas: CastlingPerm::new(),
            key: 0,

            fty: 0,
            ply: 0,
            pv_line: PVLine::new(),
            pv_table: HashTable::new(),

            search_hist: [[0; 64]; 12],
            search_killers: [[Move::new(); MAX_DEPTH]; 2],
            counter_moves: [[Move::new(); 64]; 64],

            material: [0; 2],
        }
    }

    pub fn previous_move(&self) -> Move {
        match self.hist_ply {
            0 => Move::new(),
            idx => self.hist[idx as usize].m
        }
    }

    pub fn turnx(&self) -> usize {
        self.turn as usize
    }

    pub fn all_ocupied(&self) -> BitBoard {
        self.occupied[WHITEX] | self.occupied[BLACKX]
    }

    pub fn verify(&self) {
        for pt in PAWNX..=KINGX {
            for c in WHITEX..=BLACKX {
                for sq in self.pieces[c][pt].bits() {
                    assert_eq!(self.board[sq as usize], Piece::new(pt as u8, c as u8));
                    assert!(self.occupied[c].chk(sq));
                    assert!(!self.occupied[c^1].chk(sq), "{}", sq);
                }
            }
        }
        for sq in 0..64 {
            let p = self.board[sq as usize];
            let (cx, px) = (p.get_color() as usize, p.get_type() as usize);
            if p.is_some() {
                assert!(self.pieces[cx][px].chk(sq));
                assert!(self.occupied[cx].chk(sq));
                assert!(!self.occupied[cx^1].chk(sq));
            } else {
                for px in PAWNX..=KINGX {
                    assert!(!self.pieces[cx][px].chk(sq));
                }
                assert!(!self.all_ocupied().chk(sq));
            }
        }
        assert_eq!(self.key, ZOBRIST.gen_key(self));
    }

    pub fn do_castling(&mut self, f: u8, t: u8) {
        const ROOK_FROM: [[u8; 2]; 2] = [ [0, 7], [56, 63] ];
        const ROOK_TO: [[u8; 2]; 2] = [ [3, 5], [59, 61] ];
        let x = (t > f) as usize;
        let side = self.turn as usize;
        self.move_piece(f, t);
        self.move_piece(ROOK_FROM[side][x], ROOK_TO[side][x]);
        self.cas.dis_both(self.turn);
    }

    pub fn add_piece(&mut self, side: usize, px: usize, sq: u8) {
        let p = Piece::new(px as u8, side as u8);
        self.board[sq as usize] = p;
        self.occupied[side].set(sq);
        self.pieces[side][px].set(sq);
        self.material[side] += MATERIAL_TABLE[px];
        self.key ^= ZOBRIST.piece(sq, p);
    }

    pub fn remove_piece(&mut self, sq: u8) {
        let p = self.board[sq as usize];
        let (side, px) = (p.get_color() as usize, p.get_type() as usize);
        self.pieces[side][px].clear(sq);
        self.occupied[side].clear(sq);
        self.board[sq as usize] = Piece::none();
        self.material[side] -= MATERIAL_TABLE[px];
        self.key ^= ZOBRIST.piece(sq, p);
    }

    pub fn move_piece(&mut self, from: u8, to: u8) {
        let p = self.board[from as usize];
        let (side, px) = (p.get_color() as usize, p.get_type() as usize);
        self.pieces[side][px].clear(from);
        self.occupied[side].clear(from);
        self.board[from as usize] = Piece::none();
        self.board[to as usize] = p;
        self.occupied[side].set(to);
        self.pieces[side][px].set(to);
        self.key ^= ZOBRIST.piece(from, p);
        self.key ^= ZOBRIST.piece(to, p);
    }

    pub fn reset(&mut self) {
        for p in self.board.iter_mut() {
            *p = Piece::none();
        }
        for c in WHITEX..=BLACKX {
            for p in PAWNX..=KINGX {
                self.pieces[c][p] = 0;
            }
            self.occupied[c] = 0;
        }
        self.hist_ply = 0;
        self.ep = NS;
        self.turn = WHITE;
        self.cas = CastlingPerm::new();
        self.key = 0;
        self.fty = 0;
        self.ply = 0;
        self.material = [0; 2];
        //the rest is cleared automatically by search()
    }

    pub fn load_fen(&mut self, fen: &str) {
        self.reset();
        let mut ss = fen.split_whitespace();
        let (mut rank, mut file) = (7, 0);

        for ch in ss.next().unwrap().chars() {
            let c = ch.is_lowercase() as usize;
            let sq = rank * 8 + file;
            let px = match ch.to_ascii_lowercase() {
                'p' => PAWNX,
                'n' => KNIGHTX,
                'b' => BISHOPX,
                'r' => ROOKX,
                'q' => QUEENX,
                'k' => KINGX,
                '/' => { rank -= 1; file = 0; continue; },
                ' ' => break,
                c if c.is_digit(9) && c != '0' 
                    => { file += c.to_digit(9).unwrap() as u8; continue; }
                ch => panic!("unexpected char in fen string {}", ch),

            };
            self.add_piece(c, px, sq);
            file += 1;
        }
        self.turn = match ss.next().unwrap() {
            "w" => WHITE,
            "b" => BLACK,
            _ => panic!("invalid side character"),
        };
        self.cas = CastlingPerm::from_str(ss.next().unwrap()).unwrap();
        self.ep = match ss.next().unwrap() {
            "-" => NS,
            ep => (ep.as_bytes()[0] - b'a' + (ep.as_bytes()[1] - b'1')*8) 
        };
        self.fty = ss.next()
            .and_then(|s|s.parse::<u8>().ok())
            .unwrap_or(0);

        self.key ^= ZOBRIST.en_passant(self.ep);
        self.key ^= ZOBRIST.castling(self.cas);
        if self.turn == WHITE { self.key ^= ZOBRIST.side(); }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const S: [char; 7] = ['.', 'p', 'n', 'b', 'r', 'q', 'k'];
        const TURN: [&str; 2] = ["White", "Black"];
        for i in (0..8).rev() {
            write!(f, "{}    ", i+1)?;
            for p in self.board[i*8..(i+1)*8].iter() {
                let pdx = if p.is_none() { 0 } else { p.get_type() as usize + 1 };
                write!(f, "{}  ", match p.get_color() {
                    WHITE => S[pdx].to_ascii_uppercase(),
                    BLACK => S[pdx],
                    _ => unreachable!()
                })?;
            }
            writeln!(f)?;
        }
        write!(f, "\n     ")?;
        for i in ['a','b','c','d','e','f', 'g', 'h'].iter() {
            write!(f, "{}  ", i)?;
        }
        writeln!(f, "\nturn: {} // cas: {} // ep: {} // fty: {} //eval: {}", 
            TURN[self.turn as usize], self.cas, self.ep, self.fty, self.eval())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs_to_flags(cs: CastlingPerm) -> [bool; 4] {
        [cs.king(WHITE), cs.queen(WHITE), cs.king(BLACK), cs.queen(BLACK)]
    }

    #[test]
    fn castling_state() {
        assert_eq!(CastlingPerm::new(), CastlingPerm::from_str("-").unwrap());
        let mut cs = CastlingPerm::from_str("KQkq").unwrap();
        assert_eq!(cs_to_flags(cs), [true, true, true, true]);
        cs.dis_king(WHITE);
        assert_eq!(cs_to_flags(cs), [false, true, true, true]);
        cs.dis_king(BLACK);
        assert_eq!(cs_to_flags(cs), [false, true, false, true]);
        cs.dis_queen(WHITE);
        assert_eq!(cs_to_flags(cs), [false, false, false, true]);
        cs.dis_queen(BLACK);
        assert_eq!(cs_to_flags(cs), [false, false, false, false]);
    }
}