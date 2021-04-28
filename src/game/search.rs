use super::{pos::*, movgen::*, pvtable::*};
use std::{fmt, time::{SystemTime, Duration}};

const INFINITY: i16 = i16::MAX;
const CHECKUP_INTERVAL_MASK: u32 = 2047;

struct Score(i16);

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 > INFINITY - 100 {
            write!(f, "mate {}", (INFINITY - self.0 + 1) / 2)
        } else if self.0 < -INFINITY + 100 {
            write!(f, "mate {}", (self.0 + INFINITY - 1) / 2)
        } else {
            write!(f, "cp {}", self.0)
        }
    }
}

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
    pub fn is_repetition(&self) -> bool {
        for i in self.hist_ply as usize - self.fty as usize..self.hist_ply as usize{
            if self.key == self.hist[i].key {
                return true
            }
        }
        false
    }

    pub fn search_reset(&mut self) {
        for i in self.search_hist.iter_mut() {
            for j in i.iter_mut() {
                *j = 0;
            }
        }
        for i in self.search_killers.iter_mut() {
            for j in i.iter_mut() {
                *j = Move::new();
            }
        }

        // self.pv_table.clear();
        self.ply = 0;
    }

    pub fn search(&mut self, info: &mut SearchInfo) {
        self.search_reset();
        for depth in 1..=info.depth {
	    info.nodes = 0;
            let best_score = self.alpha_beta(-INFINITY, INFINITY, depth, info, true);

            if info.stopped { break; }

            let ellapsed = info.start_time.elapsed().unwrap().as_millis();

            print!("info score {} depth {} nodes {} time {}",
                     Score(best_score), depth, info.nodes, ellapsed);

            print!(" pv");
            self.extract_pv_line(depth);
            for m in self.pv_line.iter() {
                print!(" {}", m);
            }
            println!(" // Ordering: {:.2}", info.fhf / info.fh);
        }
        if let Some(m) = self.pv_line.iter().next() {
            println!("bestmove {}", m);
        } else {
            println!("no legal moves");
        }
    }

    fn alpha_beta(&mut self, mut alpha: i16, beta: i16, mut depth: u8, info: &mut SearchInfo, null: bool) -> i16 {
        if self.ply > 0 && self.is_repetition() || self.fty >= 100 { return 0; }
        if depth == 0 {
            return self.quiescence(alpha, beta, info);
        }

        if info.checkup() { return 0; }
        if self.ply as usize >= MAX_DEPTH { return self.eval() }
        info.nodes += 1;

        let in_check = self.in_check(self.turnx());
        if in_check {
            depth += 1;
        }

        const R: u8 = 3;
        if null && !in_check && depth > R && !self.is_endgame() {
            self.make_null_move();
            let score = -self.alpha_beta(-beta, -beta+1, depth - 1 - R, info, false);
            self.unmake_null_move();
            if score >= beta {
                return beta;
            }
        }

        let mut pv_move = Move::new();
        if let Some(e) = self.pv_table.probe(self.key) {
            if e.depth >= depth {
                use EntryFlags::*;
                match e.flags {
                    Exact => return e.score,
                    Alpha if e.score <= alpha => return alpha,
                    Beta if e.score >= beta => return beta,
                    _ => (),
                }
            }
            pv_move = e.m;
        }

        let mut best_move = Move::new();
        let mut best_score = -INFINITY;
        let old_alpha = alpha;
        let mut legal = 0;
        let mut moves = MoveList::new();
        self.gen_moves::<false>(&mut moves);
        if !pv_move.is_null() {
            if let Some(fnd) = moves.iter_mut().find(|om|om.0 == pv_move) {
                fnd.1 = u16::MAX;
            }
        }

        const FULL_DEPTH_MOVES: u8 = 4;
        const REDUCTION_LIMIT: u8 = 3;
        let mut moves_searched = 0;

        let mut it = moves.iter_picky();
        while let Some(m) = it.next() {
            let m = m.0;
            if !self.make_move(m) { continue; }
            self.ply += 1;
            legal += 1;

            let score = if moves_searched == 0 {
                -self.alpha_beta(-beta, -alpha, depth-1, info, true)
            } else {
                let mut score;
                if moves_searched >= FULL_DEPTH_MOVES && depth >= REDUCTION_LIMIT
                    && !in_check && !m.cap() && m.prom() == 0 {
                    score = -self.alpha_beta(-alpha-1, -alpha, depth-2, info, true);
                } else {
                    score = alpha + 1;
                }
                if score > alpha {
                    score = -self.alpha_beta(-alpha-1, -alpha, depth-1, info, true);
                    if score > alpha && score < beta {
                        score = -self.alpha_beta(-beta, -alpha, depth-1, info, true);
                    }
                }
                score
            };
            
            self.unmake_move();
            self.ply -= 1;

            if info.stopped { return 0 }

            if score > best_score {
                best_score = score;
                best_move = m;
                if score > alpha {
                    if score >= beta {
                        if legal == 1 {
                            info.fhf += 1.;
                        }
                        info.fh += 1.;

                        if !m.cap() {
                            self.search_killers[1][self.ply as usize] = self.search_killers[0][self.ply as usize];
                            self.search_killers[0][self.ply as usize] = m;
                        }

                        self.pv_table.store(self.key, HashEntry {
                            depth, flags: EntryFlags::Beta, m, score: beta
                        });
                        return beta;
                    }
                    alpha = score;

                    if !m.cap() {
                        self.search_hist[self.board[m.from() as usize].id() as usize][m.to() as usize] += depth as u16;
                    }
                }
            }
            moves_searched += 1;
        }

        if legal == 0 {
            return match self.in_check(self.turnx()) {
                true => -INFINITY + self.ply as i16,
                false => 0,
            }
        }

        if alpha != old_alpha {
            self.pv_table.store(self.key, HashEntry {
                depth, flags: EntryFlags::Exact, m: best_move, score: best_score
            })
        } else {
            self.pv_table.store(self.key, HashEntry {
                depth, flags: EntryFlags::Alpha, m: best_move, score: alpha
            })
        }

        alpha
    }

    fn quiescence(&mut self, mut alpha: i16, beta: i16, info: &mut SearchInfo) -> i16 {
        if info.checkup() { return 0; }
        info.nodes += 1;

        // if self.is_repetition() || self.fty >= 100 { return 0; }
        if self.ply as usize >= MAX_DEPTH  { return self.eval() }


        let score = self.eval();
        if score >= beta { return beta; }
        if score > alpha { alpha = score; }

        let mut moves = MoveList::new();
        self.gen_moves::<true>(&mut moves);
        let mut legal = 0;

        let mut it = moves.iter_picky();
        while let Some(om) = it.next() {
            let m = om.0;
            if !self.make_move(m) { continue; }
            self.ply += 1;
            legal += 1;
            
            let score = -self.quiescence(-beta, -alpha, info);
            self.unmake_move();
            self.ply -= 1;

            if info.stopped { return 0 }

            if score > alpha {
                if score >= beta {
                    if legal == 1 {
                        info.fhf += 1.;
                    }
                    info.fh += 1.;

                    // self.store_pv_move(m);
                    return beta;
                }
                alpha = score;
            }
        }

        // if alpha != old_alpha {
            // self.store_pv_move(best_move);
        // }

        alpha
    }
}
