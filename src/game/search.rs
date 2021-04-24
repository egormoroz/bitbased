use super::{pos::*, movgen::*, pvtable::*};
use std::time::{SystemTime, Duration};

const INFINITY: i32 = i32::MAX;
const CHECKUP_INTERVAL_MASK: u32 = 0xFFF;

pub struct SearchInfo {
    start_time: SystemTime,
    move_time: Option<Duration>,
    depth: u8,
    // depth_set: u8,
    // time_set: u32,

    // move_to_go: u32,

    nodes: u32,

    fh: f32,
    fhf: f32,

    // quit: bool,
    stopped: bool,
}

impl SearchInfo {
    pub fn new(depth: u8, move_time: Option<Duration>) -> Self {
        let now = SystemTime::now();
        Self {
            start_time: now,
            depth, nodes: 0, fh: 0., fhf: 0.,
            move_time,
            stopped: false,
        }
    }

    fn checkup(&mut self) -> bool {
        if self.nodes & CHECKUP_INTERVAL_MASK == 0 {
            if let Some(move_time) = self.move_time {
                if self.start_time.elapsed().unwrap() >= move_time {
                    self.stopped = true;
                }
            }
        }

        self.stopped
    }
}

impl Position {
    fn is_repetition(&self) -> bool {
        for i in self.hist_ply as usize - self.fty as usize..self.hist_ply as usize{
            if self.key == self.hist[i].key {
                return true
            }
        }
        false
    }

    pub fn search_reset(&mut self) {
        /*
        for i in self.search_hist.iter_mut() {
            for j in i.iter_mut() {
                *j = 0;
            }
        }
        for i in self.search_killers.iter_mut() {
            for j in i.iter_mut() {
                *j = Move::none();
            }
        }
        */

        self.pv_table.clear();
        self.ply = 0;
    }

    pub fn search(&mut self, info: &mut SearchInfo) {
        self.search_reset();
        for depth in 1..=info.depth {
            let best_score = self.alpha_beta(-INFINITY, INFINITY, depth, info);

            if info.stopped { break; }

            let ellapsed = info.start_time.elapsed().unwrap().as_millis();

            print!("info score cp {} depth {} nodes {} time {}",
                     best_score, depth, info.nodes, ellapsed);

            print!(" pv");
            self.extract_pv_line(depth);
            for m in self.pv_line.iter() {
                print!(" {}", m);
            }
            println!()
        }
        if let Some(m) = self.pv_line.iter().next() {
            println!("bestmove {}", m);
        } else {
            println!("no legal moves");
        }
        println!("Ordering: {:.2}", info.fhf / info.fh);
    }

    fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: u8, info: &mut SearchInfo) -> i32 {
        if depth == 0 {
            // return self.quiescence(alpha, beta, info);
            info.nodes += 1;
            return self.eval();
        }

        if info.checkup() { return 0; }

        info.nodes += 1;

        if self.is_repetition() || self.fty >= 100 { return 0; }
        if self.ply as usize >= MAX_DEPTH -1 { return self.eval() }

        let mut moves = MoveList::new();
        self.gen_moves(&mut moves);
        let mut legal = 0;

        if let Some(m) = self.pv_table.probe(self.key) {
            if let Some(pv_move) = moves.iter_mut().find(|om|om.0 == m) {
                pv_move.1 = u16::MAX;
            }
        }

        let mut it = moves.iter_picky();
        while let Some(m) = it.next() {
            let m = m.0;
            if !self.make_move(m) { continue; }
            legal += 1;
            
            let score = -self.alpha_beta(-beta, -alpha, depth-1, info);
            self.unmake_move();

            if score > alpha {
                if score >= beta {
                    if legal == 1 {
                        info.fhf += 1.;
                    }
                    info.fh += 1.;

                    return beta;
                }
                alpha = score;
                self.store_pv_move(m);
            }
        }

        if legal == 0 {
            return match self.in_check(self.turnx()) {
                true => -INFINITY + self.ply as i32,
                false => 0,
            }
        }

        alpha
    }

    /*
    fn quiescence(&mut self, mut alpha: i32, beta: i32, info: &mut SearchInfo) -> i32 {
        if info.checkup() { return 0; }
        info.nodes += 1;

        if self.is_repetition() || self.fty >= 100 { return 0; }
        if self.ply as usize >= MAX_DEPTH -1 { return self.eval() }


        let score = self.eval();
        if score >= beta { return beta; }
        if score > alpha { alpha = score; }

        let mut moves = MoveBuf::new();
        self.gen_captures(&mut moves);
        let (mut legal, old_alpha) = (0, alpha);
        let mut best_move = Move::none();

        let mut it = moves.iter_picky();
        while let Some(m) = it.next() {
            let m = m.m;
            if !self.make_move(m) { continue; }
            legal += 1;
            
            let score = -self.quiescence(-beta, -alpha, info);
            self.unmake_move();

            if score > alpha {
                if score >= beta {
                    if legal == 1 {
                        info.fhf += 1.;
                    }
                    info.fh += 1.;

                    return beta;
                }
                alpha = score;
                best_move = m;
            }
        }

        if alpha != old_alpha {
            self.store_pv_move(best_move);
        }

        alpha
    }
    */
}
