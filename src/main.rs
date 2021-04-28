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
 
fn main() {
    use game::perft::POSITIONS;
    // perft_test();
    // let mut b = Position::from_fen("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1").unwrap();
    let mut b = Position::new();
    b.load_fen(POSITIONS[0]);
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
            "s" => b.search(&mut SearchInfo::new(16, None)),
            "t" => { b.unmake_move(); continue; },
            "uci" => { 
                uci::UCI::new(b).uci_loop();
                return
            }
            "r" => { println!("{}", b.is_repetition()) },
            "fen" => {
                buf.clear();
                io::stdin().read_line(&mut buf).unwrap();
                b.load_fen(buf.trim());
            }
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
