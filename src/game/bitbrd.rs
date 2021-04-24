use super::defs::*;

pub type BitBoard = u64;

pub trait BitHelpers {
    fn set(&mut self, sq: u8);
    fn clear(&mut self, sq: u8);
    fn chk(&self, sq: u8) -> bool;
    fn bits(&self) -> BitRunner;

    fn print(&self);
}

impl BitHelpers for BitBoard {
    fn set(&mut self, sq: u8) { *self |= 1 << sq; }
    fn clear(&mut self, sq: u8) { *self &= !(1 << sq); }

    fn bits(&self) -> BitRunner { BitRunner(*self) }
    fn chk(&self, sq: u8) -> bool { *self & (1 << sq) != 0 }

    fn print(&self) {
       for rank in (0..8).rev() {
            print!("{}  ", RANKC[rank as usize]);
            for file in 0..8 {
                let sq = file + rank * 8;
                print!("{} ", match self.chk(sq) {
                    true => '1',
                    false => '.',
                });
            }
            println!();
        }
        print!("   ");
        for fc in FILEC.iter() {
            print!("{} ", fc);
        } 
        println!()
    }
}

pub struct BitRunner(u64);

impl Iterator for BitRunner {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let n = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1;
            Some(n)
        } else {
            None
        }
    }
}
