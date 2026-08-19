// cards, deck, hand evaluator
use rand::{RngExt, SeedableRng};
use rand::rngs::StdRng;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
}


#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Suit {
    Clubs, Diamonds, Hearts, Spades
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    PreDeal,
    Dealt,
    PreFlop,
    Flop,
    Turn,
    River
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
enum Action {
    Fold,
    Bet,
    Check
}


struct Card(u8);
impl Card {
    fn new(rank: Rank, suit: Suit) -> Self {
        Self(((rank as u8) << 2) | (suit as u8))
    }

    fn rank(&self) -> u8 {
        self.0 >> 2
    }

    fn suit(&self) -> u8 {
        self.0 & 0b11
    }
}


struct Deck {
    live: Vec<Card>,
    used: Vec<Card>,
    seed: u64
}
impl Deck {
    fn new(seed: u64) -> Self {
        Self {
            live: (0u8..13).flat_map(|r| (0u8..4).map(move |s| Card(r << 2 | s))).collect(),
            used: Vec::with_capacity(52),
            seed: seed
        }
    }

    
    fn shuffle(&mut self, rng: &mut StdRng) {
        let mut all = std::mem::take(&mut self.live);
        all.extend(std::mem::take(&mut self.used));

        while all.len() > 0 {
            let rand_i = rng.random_range(0..all.len());
            self.live.push(all.swap_remove(rand_i));
        }
    }


    fn next_card(&mut self) -> Card {
        self.live.pop().unwrap()
    } 
}


struct Player {
    name: String,
    chips: i32,
}


struct GameRunner {
    // cards
    community: Vec::<Card>,
    muck: Vec::<Card>,
    burn: Vec::<Card>,
    deck: Deck,

    // meta data
    phase: Phase,
    b_i: i32,
    sb_i: i32,
    bb_i: i32,
    action_i: i32,
    rng: StdRng,

    // money
    pot: i32,

    players: Vec::<Player>,

}
impl GameRunner {
    fn new(players: Vec::<Player>, seed: u64) -> Self {
        Self {
            community: Vec::with_capacity(5),
            muck: Vec::with_capacity(18), // max 9 players
            burn: Vec::with_capacity(5),
            deck: Deck::new(3385958),

            phase: Phase::PreDeal,
            b_i: 0,
            sb_i: 0,
            bb_i: 0,
            action_i: 0,
            rng: StdRng::seed_from_u64(seed),
            
            pot: 0,

            players: players
        }
    }

    fn next_phase(&mut self) {
        if self.phase == Phase::PreDeal {
            self.deck.shuffle(&mut self.rng);
        }
    }
}