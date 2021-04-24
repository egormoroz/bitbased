extern crate bitintr;
extern crate lazy_static;

pub mod game;
use game::pos::*;
use game::movgen::*;

use std::io::{self, Write as IOWrite};

pub fn perft_test() {
    use game::perft::*;
    use std::time::SystemTime;
    lazy_static::initialize(&ATTK_TBL);
    let mut p = Position::from_fen(POSITIONS[1]).unwrap();
    let now = SystemTime::now();
    println!("{}: {}", perft(&mut p, 5), now.elapsed().unwrap().as_secs_f64());
}

 
fn main() {
    perft_test();
    let mut b = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
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
            "t" => { b.unmake_move(); continue; },
            _ => (),
        }

        moves.clear();
        b.gen_moves(&mut moves);
        let mut n = 0;
        for m in moves.iter() {
            if b.make_move(*m) {
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
            write!(&mut mbuf, "{}", m).unwrap();
            mbuf.trim() == buf
        }) {
            b.make_move(*fnd);
        }
    }
}
