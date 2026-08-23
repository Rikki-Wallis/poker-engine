use rand::RngExt;
use rand::rngs::StdRng;

use crate::card::Card;

pub struct Deck {
    live: Vec<Card>,
    used: Vec<Card>,
    seed: u64
}


impl Deck {
    pub fn new(seed: u64) -> Self {
        Self {
            live: (0u8..13).flat_map(|r| (0u8..4).map(move |s| Card::from_raw(r << 2 | s))).collect(),
            used: Vec::with_capacity(52),
            seed: seed
        }
    }


    pub fn shuffle(&mut self, rng: &mut StdRng) {
        let mut all = std::mem::take(&mut self.live);
        all.extend(std::mem::take(&mut self.used));

        while all.len() > 0 {
            let rand_i = rng.random_range(0..all.len());
            self.live.push(all.swap_remove(rand_i));
        }
    }


    pub fn next_card(&mut self) -> Card {
        self.live.pop().unwrap()
    }
}
