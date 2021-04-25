extern crate bitintr;
extern crate lazy_static;
extern crate rand;
// extern crate static_init;

pub mod game;
use game::pos::*;
use game::movgen::*;
use game::search::*;
use game::uci;

use std::io::{self, Write as IOWrite};

pub fn perft_test() {
    use game::perft::*;
    use game::zobrist::*;
    use std::time::SystemTime;
    lazy_static::initialize(&ATTK_TBL);
    lazy_static::initialize(&ZOBRIST);
    let mut p = Position::from_fen(POSITIONS[1]).unwrap();
    let now = SystemTime::now();
    let n = perft(&mut p, 5);
    println!("{} mil nodes/s", n as u128 / now.elapsed().unwrap().as_micros());
}

 
fn main() {
    // perft_test();
    let mut b = Position::from_fen("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").unwrap();
    let mut buf = String::new();
    let mut mbuf = String::with_capacity(8);
    let mut moves = MoveList::new();
    loop {
        println!("{}", b);
        print!(">>> ");
        io::stdout().flush().unwrap();
        buf.clear();
        io::stdin().read_line(&mut buf).unwrap();
        match buf.trim() {
            "q" => break,
            "s" => b.search(&mut SearchInfo::new(8, None)),
            "t" => { b.unmake_move(); continue; },
            "uci" => uci::UCI::new().uci_loop(),
            "r" => { println!("{}", b.is_repetition()) }
            _ => (),
        }

        moves.clear();
        b.gen_moves::<false>(&mut moves);
        let mut n = 0;
        for m in moves.iter() {
            if b.make_move(m.0) {
                b.unmake_move();
                n += 1;
            }
        }
        println!("found {} moves ({} of them are legal)", moves.len(), n);
        buf.make_ascii_lowercase();
        let buf = buf.trim();
        if let Some(fnd) = moves.iter().find(|m| {
            use std::fmt::Write;
            mbuf.clear();
            write!(&mut mbuf, "{}", m.0).unwrap();
            mbuf.trim() == buf
        }) {
            b.make_move(fnd.0);
        }
    }
}
