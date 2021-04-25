use super::{defs::*, movgen::*, pos::*, zobrist::*};

const KING_MASK: [u64; 2] = [ 7 << 4, 7 << 60 ]; //TODO: Check this!
const QUEEN_MASK: [u64; 2] = [ 7 << 2, 7 << 58 ]; //and this!

impl Position {
    pub fn make_move(&mut self, m: Move) -> bool {
        // self.verify();
        let mut hist = Hist {
            m, cap: self.board[m.to() as usize], cas: self.cas, 
            ep: self.ep, fty: self.fty, key: self.key
        };

        let (f, t) = (m.from(), m.to());
        let turnx = self.turnx();
        let kind = m.kind();
        let mut ep = NS;
        if kind.en_passant() {
            assert!(self.ep != NS);
            let (rk, fl) = (f / 8, self.ep % 8);
            let sq = rk * 8 + fl;
            hist.cap = self.board[sq as usize];
            self.remove_piece(sq);
            self.move_piece(f, t);
            self.fty = 0;
        } else if kind.castle() {
            let atk = self.attacked(turnx ^ 1);
            if t > f {
                if atk & KING_MASK[turnx] != 0 { return false }
                self.do_castling(f, t);
            } else {
                if atk & QUEEN_MASK[turnx] != 0 { return false }
                // if self.turn == WHITE { println!("{}", self); }
                self.do_castling(f, t);
                // if self.turn == WHITE { println!("{}", self); }
            }
            self.fty += 1;
        } else if kind.long_push() {
            self.move_piece(f, t);
            if self.turn == WHITE { ep = f + 8; }
            else { ep = t + 8; }
            self.fty = 0;
        } else {
            self.fty += 1;
            if m.prom() != 0 {
                self.remove_piece(f);
                self.add_piece(turnx, m.prom() as usize, f);
                self.fty = 0;
            }
            if m.cap() {
                self.remove_piece(t);
                self.fty = 0;
            }
            self.move_piece(f, t);

            if f == 4 || t == 4 { self.cas.dis_both(WHITE); }
            if f == 60 || t == 60 { self.cas.dis_both(BLACK); }
            if f == 0 || t == 0 { self.cas.dis_queen(WHITE); }
            if f == 7 || t == 7 { self.cas.dis_king(WHITE); }
            if f == 56 || t == 56 { self.cas.dis_queen(BLACK); }
            if f == 63 || t == 63 { self.cas.dis_king(BLACK); }
        }
        self.ep = ep;
        self.hist[self.hist_ply as usize] = hist;
        self.hist_ply += 1;
        self.turn ^= 1;

        self.key ^= ZOBRIST.castling(hist.cas) ^ ZOBRIST.castling(self.cas)
            ^ ZOBRIST.side() ^ ZOBRIST.en_passant(hist.ep) ^ ZOBRIST.en_passant(self.ep);

        // self.verify();
        if self.in_check(turnx) { 
            self.unmake_move();
            false
        } else {
            true
        }
    }

    pub fn unmake_move(&mut self) {
        // self.verify();
        assert!(self.hist_ply != 0);
        self.turn ^= 1;
        self.hist_ply -= 1;
        let hist = self.hist[self.hist_ply as usize];
        let m  = hist.m;
        
        self.fty = hist.fty;
        self.ep = hist.ep;
        self.cas = hist.cas;

        let (kind, f, t) = (m.kind(), m.from(), m.to());
        let turnx = self.turnx();
        if kind.en_passant() {
            if self.turn == WHITE {
                self.add_piece(BLACKX, PAWNX, self.ep - 8);
            } else {
                self.add_piece(WHITEX, PAWNX, self.ep + 8);
            }
            self.move_piece(t, f);
        } else if kind.castle() {
            const ROOK_FROM: [[u8; 2]; 2] = [ [0, 7], [56, 63] ];
            const ROOK_TO: [[u8; 2]; 2] = [ [3, 5], [59, 61] ];
            let x = (t > f) as usize;
            self.move_piece(ROOK_TO[turnx][x], ROOK_FROM[turnx][x]);
            self.move_piece(t, f);
        } else if m.prom() != 0 {
            self.remove_piece(t);
            self.add_piece(turnx, PAWNX, f);
            if m.cap() {
                self.add_piece(turnx ^ 1, hist.cap.get_type() as usize, t);
            }
        } else {
            self.move_piece(t, f);
            if m.cap() {
                self.add_piece(turnx ^ 1, hist.cap.get_type() as usize, t);
            }
        }

        self.key = hist.key;
        // self.verify();
    }
}
