use std::collections::HashMap;
use crate::card::{Card, Rank};

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


fn longest_straight_high(ranks: &[u8]) -> Option<u8> {
    let mut unique: Vec<u8> = ranks.to_vec();
    unique.sort_unstable();
    unique.dedup();

    // refers to A being lowest card on flush. In this case the high cards is 5
    let has_wheel = [0u8, 1, 2, 3, 12].iter().all(|r| unique.contains(r));
    let mut best: Option<u8> = if has_wheel { Some(3) } else { None };

    let mut run_start = 0usize;
    for i in 1..unique.len() {
        if unique[i] != unique[i - 1] + 1 {
            run_start = i;
        }
        if i - run_start + 1 >= 5 {
            best = Some(unique[i]);
        }
    }

    best
}


pub fn evaluate_hand(hole_cards: Vec::<Card>, community_cards: Vec::<Card>) -> HandType {
    // Simple implementation for now, hard code checks.
    // Later use 2 + 2 or Cactus Kev alg

    // TODO: refactor Hand struct into this

    // collate all cards and sort in acsending order
    let mut all_cards = hole_cards;
    all_cards.extend(community_cards);
    all_cards.sort_unstable_by_key(|c| c.rank());

    // counters
    let mut rank_freq: HashMap<u8, u8> = HashMap::new();
    let mut suit_freq: HashMap<u8, u8> = HashMap::new();

    // build up counters
    for card in all_cards.iter() {
        *rank_freq.entry(card.rank()).or_insert(0) += 1;
        *suit_freq.entry(card.suit()).or_insert(0) += 1;
    }

    // check for flush
    let flush_suit = suit_freq.iter().find(|&(_, &freq)| freq >= 5).map(|(&suit, _)| suit);

    // check for straights
    let all_ranks: Vec<u8> = all_cards.iter().map(|c| c.rank()).collect();
    let straight_high = longest_straight_high(&all_ranks);

    // check for straight over suited cards - straight/royal flush
    let straight_flush_high = flush_suit.and_then(|suit| {
        let flush_ranks: Vec<u8> = all_cards.iter().filter(|c| c.suit() == suit).map(|c| c.rank()).collect();
        longest_straight_high(&flush_ranks)
    });

    if let Some(high) = straight_flush_high {
        if high == Rank::Ace as u8 {
            return HandType::RoyalFlush
        } else {
            return HandType::StraightFlush
        };
    }

    // look for quads, trips, pairs
    let mut quads: Vec<u8> = Vec::new();
    let mut trips: Vec<u8> = Vec::new();
    let mut pairs: Vec<u8> = Vec::new();
    for (&rank, &freq) in rank_freq.iter() {
        match freq {
            4 => quads.push(rank),
            3 => trips.push(rank),
            2 => pairs.push(rank),
            _ => {}
        }
    }
    quads.sort_unstable_by(|a, b| b.cmp(a));
    trips.sort_unstable_by(|a, b| b.cmp(a));
    pairs.sort_unstable_by(|a, b| b.cmp(a));

    if !quads.is_empty() {
        return HandType::Quads;
    }

    // full house, best trip + a pair
    // edge case if there are two trips and lower trip is used as a pair
    if !trips.is_empty() && (!pairs.is_empty() || trips.len() >= 2) {
        return HandType::FullHouse;
    }

    if flush_suit.is_some() {
        return HandType::Flush;
    }

    if straight_high.is_some() {
        return HandType::Straight;
    }

    if !trips.is_empty() {
        return HandType::Trips;
    }

    if pairs.len() >= 2 {
        return HandType::TwoPair;
    }

    if pairs.len() == 1 {
        return HandType::Pair;
    }

    // otherwise, high card
    HandType::HighCard
}


pub struct Hand {
    pub type_: HandType,
    pub cards: [Card; 5],
}
impl Hand {

    /// Compares two hands of the same 'HandType' and returns the higher one.
    ///
    /// Returns `None` if the hands are exactly tied
    pub fn tiebreak<'a>(&'a self, other: &'a Self) -> Option<&'a Self> {
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
