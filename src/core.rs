// cards, deck, hand evaluator
use rand::{RngExt, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Suit {
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

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum HandType {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
    RoyalFlush
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Card(u8);
impl Card {
    fn new(rank: Rank, suit: Suit) -> Self {
        Self(((rank as u8) << 2) | (suit as u8))
    }

    pub fn rank(&self) -> u8 {
        self.0 >> 2
    }

    pub fn suit(&self) -> u8 {
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


struct Hand {
    type_: HandType,
    cards: [Card; 5],
}
impl Hand {
    
    /// Compares two hands of the same 'HandType' and returns the higher one.
    ///
    /// Returns `None` if the hands are exactly tied
    fn tiebreak<'a>(&'a self, other: &'a Self) -> Option<&'a Self> {
        match self.type_ {
            HandType::RoyalFlush  => {
                None
            }
            HandType::StraightFlush => {
                let high_ref = if self.cards[4].rank() > other.cards[4].rank() {self} else {other};
                Some(high_ref)
            }
            HandType::Quads => {
                let (mut self_high, mut self_kicker): (u8, u8) = (0, 0);
                let (mut other_high, mut other_kicker): (u8, u8) = (0, 0);
                let (mut self_freq, mut other_freq) = ([0u8; 13], [0u8; 13]);

                for card in self.cards.iter() {
                    self_freq[card.rank() as usize] += 1;
                }
                for card in other.cards.iter() {
                    other_freq[card.rank() as usize] += 1;
                }

                for (rank, freq) in self_freq.iter().enumerate() {
                    if *freq == 4 {
                        self_high = rank as u8
                    } else if *freq > 0 {
                        self_kicker = rank as u8
                    }
                }

                for (rank, freq) in other_freq.iter().enumerate() {
                    if *freq == 4 {
                        other_high = rank as u8
                    } else if *freq > 0 {
                        other_kicker = rank as u8
                    }
                }

                match (self_high, self_kicker).cmp(&(other_high, other_kicker)) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
            HandType::FullHouse => {
                let (mut self_trip, mut self_pair, mut other_trip, mut other_pair): (u8, u8, u8, u8) = (0, 0, 0, 0);
                let (mut self_freq, mut other_freq) = ([0u8; 13], [0u8; 13]);

                for card in self.cards.iter() {
                    self_freq[card.rank() as usize] += 1;
                }
                for card in other.cards.iter() {
                    other_freq[card.rank() as usize] += 1;
                }

                for (rank, freq) in self_freq.iter().enumerate() {
                    if *freq == 3 {
                        self_trip = rank as u8;
                    } else if *freq == 2 {
                        self_pair = rank as u8;
                    }
                }

                for (rank, freq) in other_freq.iter().enumerate() {
                    if *freq == 3 {
                        other_trip = rank as u8;
                    } else if *freq == 2 {
                        other_pair = rank as u8;
                    }
                }

                match (self_trip, self_pair).cmp(&(other_trip, other_pair)) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
            HandType::Flush => {
                if self.cards[4].rank() > other.cards[4].rank() {
                    Some(self)
                } else if other.cards[4].rank() > self.cards[4].rank() {
                    Some(other)
                } else {
                    None
                }
            }
            HandType::Straight => {
                if self.cards[4].rank() > other.cards[4].rank() {
                    Some(self)
                } else if other.cards[4].rank() > self.cards[4].rank() {
                    Some(other)
                } else {
                    None
                }
            }
            HandType::Trips => {
                let (mut self_trip, mut other_trip) = (0u8, 0u8);
                let (mut self_kickers, mut other_kickers) = ([0u8; 2], [0u8; 2]);
                let (mut self_freq, mut other_freq) = ([0u8; 13], [0u8; 13]);

                for card in self.cards.iter() {
                    self_freq[card.rank() as usize] += 1;
                }
                for card in other.cards.iter() {
                    other_freq[card.rank() as usize] += 1;
                }

                let mut i = 0;
                for (rank, freq) in self_freq.iter().enumerate().rev() {
                    if *freq == 3 {
                        self_trip = rank as u8;
                    } else if *freq == 1 {
                        self_kickers[i] = rank as u8;
                        i += 1;
                    }
                }

                let mut i = 0;
                for (rank, freq) in other_freq.iter().enumerate().rev() {
                    if *freq == 3 {
                        other_trip = rank as u8;
                    } else if *freq == 1 {
                        other_kickers[i] = rank as u8;
                        i += 1;
                    }
                }

                match (self_trip, self_kickers).cmp(&(other_trip, other_kickers)) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
            HandType::TwoPair => {
                let (mut self_pairs, mut other_pairs) = ([0u8; 2], [0u8; 2]);
                let (mut self_kicker, mut other_kicker) = (0u8, 0u8);
                let (mut self_freq, mut other_freq) = ([0u8; 13], [0u8; 13]);

                for card in self.cards.iter() {
                    self_freq[card.rank() as usize] += 1;
                }
                for card in other.cards.iter() {
                    other_freq[card.rank() as usize] += 1;
                }

                let mut i = 0;
                for (rank, freq) in self_freq.iter().enumerate().rev() {
                    if *freq == 2 {
                        self_pairs[i] = rank as u8;
                        i += 1;
                    } else if *freq == 1 {
                        self_kicker = rank as u8;
                    }
                }

                let mut i = 0;
                for (rank, freq) in other_freq.iter().enumerate().rev() {
                    if *freq == 2 {
                        other_pairs[i] = rank as u8;
                        i += 1;
                    } else if *freq == 1 {
                        other_kicker = rank as u8;
                    }
                }

                match (self_pairs, self_kicker).cmp(&(other_pairs, other_kicker)) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
            HandType::Pair => {
                let (mut self_pair, mut other_pair) = (0u8, 0u8);
                let (mut self_kickers, mut other_kickers) = ([0u8; 3], [0u8; 3]);
                let (mut self_freq, mut other_freq) = ([0u8; 13], [0u8; 13]);

                for card in self.cards.iter() {
                    self_freq[card.rank() as usize] += 1;
                }
                for card in other.cards.iter() {
                    other_freq[card.rank() as usize] += 1;
                }

                let mut i = 0;
                for (rank, freq) in self_freq.iter().enumerate().rev() {
                    if *freq == 2 {
                        self_pair = rank as u8;
                    } else if *freq == 1 {
                        self_kickers[i] = rank as u8;
                        i += 1;
                    }
                }

                let mut i = 0;
                for (rank, freq) in other_freq.iter().enumerate().rev() {
                    if *freq == 2 {
                        other_pair = rank as u8;
                    } else if *freq == 1 {
                        other_kickers[i] = rank as u8;
                        i += 1;
                    }
                }

                match (self_pair, self_kickers).cmp(&(other_pair, other_kickers)) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
            HandType::HighCard => {
                let mut self_ranks = [0u8; 5];
                let mut other_ranks = [0u8; 5];

                for (i, card) in self.cards.iter().enumerate() {
                    self_ranks[i] = card.rank();
                }
                for (i, card) in other.cards.iter().enumerate() {
                    other_ranks[i] = card.rank();
                }
                self_ranks.sort_unstable_by(|a, b| b.cmp(a));
                other_ranks.sort_unstable_by(|a, b| b.cmp(a));

                match self_ranks.cmp(&other_ranks) {
                    std::cmp::Ordering::Greater => Some(self),
                    std::cmp::Ordering::Less => Some(other),
                    std::cmp::Ordering::Equal => None,
                }
            }
        }
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        if self.type_ == other.type_ {
            return true
        }

        false
    }
}
impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare `type_` first, then fall back to `tiebreakers()` on a tie.
        todo!()
    }
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