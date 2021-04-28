use super::pos::*;
use super::defs::*;
use super::bitbrd::*;


const LOCATION_TABLE: [[i8; 64]; 4] = [
    [
        0	,	0	,	0	,	0	,	0	,	0	,	0	,	0	,
        10	,	10	,	0	,	-10	,	-10	,	0	,	10	,	10	,
        5	,	0	,	0	,	5	,	5	,	0	,	0	,	5	,
        0	,	0	,	10	,	20	,	20	,	10	,	0	,	0	,
        5	,	5	,	5	,	10	,	10	,	5	,	5	,	5	,
        10	,	10	,	10	,	20	,	20	,	10	,	10	,	10	,
        20	,	20	,	20	,	30	,	30	,	20	,	20	,	20	,
        0	,	0	,	0	,	0	,	0	,	0	,	0	,	0	
    ],
    [
        0	,	-10	,	0	,	0	,	0	,	0	,	-10	,	0	,
        0	,	0	,	0	,	5	,	5	,	0	,	0	,	0	,
        0	,	0	,	10	,	10	,	10	,	10	,	0	,	0	,
        0	,	0	,	10	,	20	,	20	,	10	,	5	,	0	,
        5	,	10	,	15	,	20	,	20	,	15	,	10	,	5	,
        5	,	10	,	10	,	20	,	20	,	10	,	10	,	5	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	0	,	0	,	0	,	0	,	0	,	0		
    ],
    [
        0	,	0	,	-10	,	0	,	0	,	-10	,	0	,	0	,
        0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
        0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
        0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
        0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
        0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
        0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
        0	,	0	,	0	,	0	,	0	,	0	,	0	,	0	
    ],
    [
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
        25	,	25	,	25	,	25	,	25	,	25	,	25	,	25	,
        0	,	0	,	5	,	10	,	10	,	5	,	0	,	0		
    ]
];


const KING_LOCATION: [[i8; 64]; 2] = [
[
	0	,	5	,	5	,	-10	,	-10	,	0	,	10	,	5	,
	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,
	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,
	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70		
],
[
	-50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50	,
	-10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
	0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
	0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
	0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
	0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
	-10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
	-50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50	
],
];

const KING_SHIELDED: i16 = 15;

const RANK: [u64; 8] = [
    0xFF      , 0xFF << 8 , 0xFF << 16, 0xFF << 24, 
    0xFF << 32, 0xFF << 40, 0xFF << 48, 0xFF << 56,
];

const FILE: [u64; 8] = [
    FILE_A     , FILE_A << 1, FILE_A << 2, FILE_A << 3, 
    FILE_A << 4, FILE_A << 5, FILE_A << 6, FILE_A << 7,
];

const PASSED_PAWN_SCORE: [i16; 8] = [
    0, 5, 10, 20, 35, 60, 100, 200
];

// const MOBILITY_MULT: i16 = 1;

const ISOLATED_PAWN_PENALTY: i16 = -5;
const DOUBLE_PAWN_PENALTY: i16 = -5;

const ROOK_OPEN_FILE: i16 = 5; 
const ROOK_SEMI_OPEN_FILE: i16 = 5;
const QUEEN_OPEN_FILE: i16 = 3;
const QUEEN_SEMI_OPEN_FILE: i16 = 2;

const CASTLE_PERM_SCORE: i16 = 10;

const BISHOP_PAIR: i16 = 30;

const ENDGAME_THRESHOLD: i16 
    = MATERIAL_TABLE[PAWNX] * 2
    + MATERIAL_TABLE[KNIGHTX] * 2
    + MATERIAL_TABLE[ROOKX] * 1;

const SAFE_THRESHOLD: i16 = -MATERIAL_TABLE[PAWNX] * 2;

fn north_one(x: u64) -> u64 { (x & !RANK[7]) << 8 }
fn south_one(x: u64) -> u64 { (x & !RANK[0]) >> 8 }
fn west_one(x: u64) -> u64 { (x & !FILE[0]) >> 1 }
fn east_one(x: u64) -> u64 { (x & !FILE[7]) << 1 }

fn north_fill(mut gen: u64) -> u64 {
    gen |= gen <<  8;
    gen |= gen << 16;
    gen |= gen << 32;
    gen
}

fn south_fill(mut gen: u64) -> u64 {
    gen |= gen >>  8;
    gen |= gen >> 16;
    gen |= gen >> 32;
    gen
}

fn file_fill(x: u64) -> u64 {
    north_fill(x) | south_fill(x)
}

fn west_attack_file_fill(x: u64) -> u64 { file_fill(west_one(x)) }
fn east_attack_file_fill(x: u64) -> u64 { file_fill(east_one(x)) }

fn no_neigh_on_west(x: u64) -> u64 { x & !west_attack_file_fill(x)}
fn no_neigh_on_east(x: u64) -> u64 { x & !east_attack_file_fill(x)}

fn isolanis(x: u64) -> u64 { no_neigh_on_west(x) & no_neigh_on_east(x) }

fn widen(gen: u64) -> u64 {
    gen | (gen & !FILE[0]) >> 1 | (gen & !FILE[7]) << 1
}

fn wking_shield(k: u64) -> u64 {
    north_one(widen(k))
}

fn bking_shield(k: u64) -> u64 {
    south_one(widen(k))
}


impl Position {
    fn is_material_draw(&self) -> bool {
        let wbs = self.pieces[WHITEX][BISHOPX].count_ones();
        let bbs = self.pieces[BLACKX][BISHOPX].count_ones();
        let wns = self.pieces[WHITEX][KNIGHTX].count_ones();
        let bns = self.pieces[BLACKX][KNIGHTX].count_ones();

        if (self.pieces[WHITEX][ROOKX] | self.pieces[WHITEX][QUEENX]
            | self.pieces[BLACKX][ROOKX] | self.pieces[BLACKX][QUEENX] 
            | self.pieces[WHITEX][PAWNX] | self.pieces[BLACKX][PAWNX]) == 0 {
            return false;
        }
        if wbs == 0 && bbs == 0 {
            if wns < 3 && bns < 3 { return true }
        } else if wns == 0 && bns == 0 {
            if (wbs as i32 - bbs as i32).abs() < 2 { return true }
        } else if wns < 3 && wbs == 0 || wbs == 1 && wns == 0 {
            if bns < 3 && bbs == 0 || bbs == 1 && bns == 0 { return true }
        }
        false
    }

    pub fn eval(&self) -> i16 {
        // let mat_white = self.material(WHITEX);
        // let mat_black = self.material(BLACKX);
        // if self.is_material_draw() { return 0 }
        let mat_white = self.material[WHITEX];
        let mat_black = self.material[BLACKX];
        let mat_diff = mat_white - mat_black;
        let max_mat = mat_white.max(mat_black);

        let mut score = mat_diff;

        for tp in PAWNX..QUEENX {
            for sq in self.pieces[WHITEX][tp].bits() {
                score += LOCATION_TABLE[tp as usize][sq as usize] as i16;
            }
            for sq in self.pieces[BLACKX][tp].bits() {
                score -= LOCATION_TABLE[tp as usize][(sq ^ 56) as usize] as i16;
            }
        }

        // score += MOBILITY_MULT * self.mobility::<WHITEX>() as i16;
        // score -= MOBILITY_MULT * self.mobility::<BLACKX>() as i16;

        //pawns
        let (wps, bps) = (self.pieces[WHITEX][PAWNX], self.pieces[BLACKX][PAWNX]);
        let wfront = north_fill(north_one(wps));
        let bfront = south_fill(south_one(bps));

        for wpasser in (wps & !widen(bfront)).bits() {
            let rank = wpasser / 8;
            score += PASSED_PAWN_SCORE[rank as usize];
        }
        for bpasser in (bps & !widen(wfront)).bits() {
            let rank = 7 - bpasser / 8;
            score -= PASSED_PAWN_SCORE[rank as usize];
        }

        score += DOUBLE_PAWN_PENALTY * (wfront & wps).count_ones() as i16;
        score -= DOUBLE_PAWN_PENALTY * (bfront & bps).count_ones() as i16;

        score += ISOLATED_PAWN_PENALTY * isolanis(wps).count_ones() as i16;
        score -= ISOLATED_PAWN_PENALTY * isolanis(bps).count_ones() as i16;

        //rooks
        let occupied = self.all_ocupied();
        let wrks = self.pieces[WHITEX][ROOKX];
        let brks = self.pieces[BLACKX][ROOKX];
        let wocc_files = file_fill(occupied & !wrks);
        let bocc_files = file_fill(occupied & !brks);

        let wof = (!wocc_files & wrks).count_ones();
        let bof = (!bocc_files & brks).count_ones();

        score += ROOK_OPEN_FILE * wof as i16;
        score += ROOK_SEMI_OPEN_FILE * (!file_fill(wps) & wrks).count_ones() as i16;
        score -= ROOK_OPEN_FILE * bof as i16;
        score -= ROOK_SEMI_OPEN_FILE * (!file_fill(bps) & brks).count_ones() as i16;

        //queens
        let wqs = self.pieces[WHITEX][QUEENX];
        let bqs = self.pieces[BLACKX][QUEENX];

        score += QUEEN_OPEN_FILE * (!file_fill(occupied & !wqs) & wqs).count_ones() as i16;
        score += QUEEN_SEMI_OPEN_FILE * (!file_fill(wps) & wqs).count_ones() as i16;
        score -= QUEEN_OPEN_FILE * (!file_fill(occupied & !bqs) & bqs).count_ones() as i16;
        score -= QUEEN_SEMI_OPEN_FILE * (!file_fill(bps) & bqs).count_ones() as i16;
        
        
        let wking_tactic = ((mat_white - mat_black >= SAFE_THRESHOLD) 
            && self.is_endgame()) as usize;
        let bking_tactic = ((mat_black - mat_white >= SAFE_THRESHOLD) 
            && self.is_endgame()) as usize;

        let wk = self.pieces[WHITEX][KINGX].trailing_zeros() as usize;
        let bk = self.pieces[BLACKX][KINGX].trailing_zeros() as usize;
        score += KING_LOCATION[wking_tactic][wk] as i16;
        score -= KING_LOCATION[bking_tactic][bk ^ 56] as i16;

       if max_mat > ENDGAME_THRESHOLD {
            score += CASTLE_PERM_SCORE * self.cas.any(WHITE) as i16;
            score -= CASTLE_PERM_SCORE * self.cas.any(BLACK) as i16;

            let wkmask = 1<<wk;
            let bkmask = 1<<bk;

            score += KING_SHIELDED * (wking_shield(wkmask) & wps).count_ones() as i16;
            score -= KING_SHIELDED * (bking_shield(bkmask) & bps).count_ones() as i16;
        }

        score += BISHOP_PAIR * (self.pieces[WHITEX][BISHOPX].count_ones() >= 2) as i16;
        score -= BISHOP_PAIR * (self.pieces[BLACKX][BISHOPX].count_ones() >= 2) as i16;

        if self.turn == BLACK { score *= -1; }
        score
    }

    pub fn is_endgame(&self) -> bool {
        self.material[WHITEX].min(self.material[BLACKX]) < ENDGAME_THRESHOLD
    }
}
