use super::{movgen::*, pos::*};

pub const POSITIONS: [&'static str; 6] = [
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
];

pub const RESULTS: [[u64; 6]; 6] = [
    [20, 400, 8902, 197281, 4_865_609, 119_060_324],
    [48, 2039, 97_862, 4_085_603, 193_690_690, 8_031_647_685],
    [14, 191, 2_812, 43_238, 674_624, 11_030_083],
    [6, 264, 9_467, 422_333, 15_833_292, 706_045_033],
    [44, 1_486, 62_379, 2_103_487, 89_941_194, 0],
    [46, 2_079, 89_890, 3_894_594, 164_075_551, 6_923_051_137],
];



pub fn perft(p: &mut Position, depth: u8) -> u64 {
    if depth == 0 { return 1 }

    let mut moves = MoveList::new();
    let mut nodes = 0;

    p.gen_moves(&mut moves);
    for m in moves.iter() {
        if p.make_move(m.0) {
            nodes += perft(p, depth - 1);
            p.unmake_move();
        }
    }
    
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;
    fn perft(p: usize, depth: usize) -> u64 {
        super::perft(&mut Position::from_fen(POSITIONS[p]).unwrap(), depth as u8)
    }

    const N: usize = 5;

    #[test]
    fn position_one() {
        assert_eq!(perft(0, N), RESULTS[0][N-1]);
    }

    #[test]
    fn position_two() {
        assert_eq!(perft(1, N), RESULTS[1][N-1]);
    }

    #[test]
    fn position_three() {
        assert_eq!(perft(2, N), RESULTS[2][N-1]);
    }

    #[test]
    fn position_four() {
        assert_eq!(perft(3, N), RESULTS[3][N-1]);
    }

    #[test]
    fn position_five() {
        assert_eq!(perft(4, N), RESULTS[4][N-1]);
    }

    #[test]
    fn position_six() {
        assert_eq!(perft(5, N), RESULTS[5][N-1]);
    }
}
