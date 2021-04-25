use super::{pos::*, movgen::*, search::*};
use super::defs::*;
use super::pvtable::MAX_DEPTH;
use std::io;

pub const START_FEN: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct UCI {
    pos: Position,
}

impl UCI {
    pub fn new() -> Self {
        Self { pos: Position::new() }
    }

    pub fn uci_loop(&mut self) {
        let mut line = String::new();
        println!("id name baka");
        println!("id author some_retard");
        println!("uciok");

        loop {
            line.clear();
            io::stdin().read_line(&mut line).unwrap();
            let line = line.trim();
            if line.is_empty() { continue; }
            let cmd = line.split_whitespace().next().unwrap_or("");

            match cmd {
                "isready" => println!("readyok"),
                "position" => self.parse_position(
                    line.strip_prefix("position").unwrap().trim()
                ),
                "ucinewgame" => self.parse_position("startpos"),
                "go" => self.parse_go(
                    line.strip_prefix("go").unwrap().trim()
                ),
                "quit" => break,
                "stop" => (),
                _ => panic!("undefined command"),
            }
            // println!("{}", self.board);
        }
    }

    fn parse_position(&mut self, line: &str) {
        let opt = line.split_whitespace().next().unwrap();
        if let Some(fen) = line.strip_prefix("fen") {
            self.pos.load_fen(fen);
        } else if opt == "startpos" {
            self.pos.load_fen(START_FEN);
        } else {
            panic!("invalid command {}", line);
        }

        if let Some(idx) = line.find("moves") {
            let rest = &line[idx..];
            for m in rest.trim().split_whitespace().skip(1) {
                self.make_move(m);
            }
        }
    }

    fn parse_go(&mut self, line: &str) {
        use std::collections::HashMap;
        use std::time::Duration;
        let mut opts = HashMap::new();
        let mut it = line.split_whitespace();
        while let Some(opt) = it.next() {
            let val = it.next().unwrap()
                .parse::<i32>().unwrap();
            opts.insert(opt, val);
        }

        let mut time = *opts.get(match self.pos.turn {
            WHITE => "wtime",
            BLACK => "btime",
            _ => unreachable!(),
        }).unwrap_or(&-1);
        let inc = *opts.get(match self.pos.turn {
            WHITE => "winc",
            BLACK => "binc",
            _ => unreachable!(),
        }).unwrap_or(&0);
        let mut movestogo = *opts.get("movestogo").unwrap_or(&30);
        let depth = *opts.get("depth").unwrap_or(&(MAX_DEPTH as i32));
        if let Some(mt) = opts.get("movetime") {
            time = *mt;
            movestogo = 1;
        }

        let mut info = SearchInfo::new(
            depth as u8,
            if time != -1 {
                Some(Duration::from_millis((time / movestogo + inc - 50) as u64))
            } else {
                None
            }
        );
        self.pos.search(&mut info);
        println!("{}", self.pos);
    }

    fn make_move(&mut self, m: &str) {
        use std::fmt::Write;
        let mut mbuf = String::new();
        let mut moves = MoveList::new();
        self.pos.gen_moves::<false>(&mut moves);
        if let Some(fnd) = moves.iter().find(|pm| {
            mbuf.clear();
            write!(&mut mbuf, "{}", pm.0).unwrap();
            mbuf.trim() == m
        }) {
            self.pos.make_move(fnd.0);
        }
    }
}
