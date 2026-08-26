use std::collections::HashMap;
use crate::card::{Card, Rank};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        self.cmp(other) == std::cmp::Ordering::Equal
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
        if self.type_ != other.type_ {
            return self.type_.cmp(&other.type_);
        }

        match self.tiebreak(other) {
            Some(winner) if std::ptr::eq(winner, self) => std::cmp::Ordering::Greater,
            Some(_) => std::cmp::Ordering::Less,
            None => std::cmp::Ordering::Equal,
        }
    }
}

// Returns the best hand given community cards and hole cards
pub fn best_hand(hole_cards: Vec<Card>, community_cards: Vec<Card>) -> Hand {
    let mut all_cards = hole_cards;
    all_cards.extend(community_cards);
    all_cards.sort_unstable_by_key(|c| c.rank());

    let mut rank_freq: HashMap<u8, u8> = HashMap::new();
    let mut suit_freq: HashMap<u8, u8> = HashMap::new();
    for card in all_cards.iter() {
        *rank_freq.entry(card.rank()).or_insert(0) += 1;
        *suit_freq.entry(card.suit()).or_insert(0) += 1;
    }

    let flush_suit = suit_freq.iter().find(|&(_, &freq)| freq >= 5).map(|(&suit, _)| suit);

    let all_ranks: Vec<u8> = all_cards.iter().map(|c| c.rank()).collect();
    let straight_high = longest_straight_high(&all_ranks);

    let straight_flush_high = flush_suit.and_then(|suit| {
        let flush_ranks: Vec<u8> = all_cards.iter().filter(|c| c.suit() == suit).map(|c| c.rank()).collect();
        longest_straight_high(&flush_ranks)
    });

    // Picks the 5 cards forming the straight ending at `high`, optionally
    // restricted to a single suit (for straight/royal flushes).
    let pick_straight_cards = |high: u8, suit: Option<u8>| -> [Card; 5] {
        let needed_ranks: Vec<u8> = if high == 3 {
            vec![12, 0, 1, 2, 3] // wheel: A,2,3,4,5 - Five (rank 3) is the high card
        } else {
            (high - 4..=high).collect()
        };
        let picked: Vec<Card> = needed_ranks.iter()
            .map(|&r| *all_cards.iter().find(|c| c.rank() == r && suit.map_or(true, |s| c.suit() == s)).unwrap())
            .collect();
        picked.try_into().unwrap()
    };

    // Takes up to `n` cards of a given rank (used for quads/trips/pairs).
    let cards_of_rank = |rank: u8, n: usize| -> Vec<Card> {
        all_cards.iter().filter(|c| c.rank() == rank).take(n).cloned().collect()
    };

    // The `n` highest-rank cards not belonging to any rank in `used_ranks`.
    let kickers = |used_ranks: &[u8], n: usize| -> Vec<Card> {
        let mut remaining: Vec<Card> = all_cards.iter().filter(|c| !used_ranks.contains(&c.rank())).cloned().collect();
        remaining.sort_unstable_by(|a, b| b.rank().cmp(&a.rank()));
        remaining.into_iter().take(n).collect()
    };

    if let Some(high) = straight_flush_high {
        let type_ = if high == Rank::Ace as u8 { HandType::RoyalFlush } else { HandType::StraightFlush };
        return Hand { type_, cards: pick_straight_cards(high, flush_suit) };
    }

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

    if let Some(&rank) = quads.first() {
        let mut cards = cards_of_rank(rank, 4);
        cards.extend(kickers(&[rank], 1));
        return Hand { type_: HandType::Quads, cards: cards.try_into().unwrap() };
    }

    // full house: best trip + a pair, or a second trip used as the pair
    if let Some(&trip_rank) = trips.first() {
        let pair_rank = if trips.len() >= 2 { Some(trips[1]) } else { pairs.first().copied() };
        if let Some(pair_rank) = pair_rank {
            let mut cards = cards_of_rank(trip_rank, 3);
            cards.extend(cards_of_rank(pair_rank, 2));
            return Hand { type_: HandType::FullHouse, cards: cards.try_into().unwrap() };
        }
    }

    if let Some(suit) = flush_suit {
        let mut suited: Vec<Card> = all_cards.iter().filter(|c| c.suit() == suit).cloned().collect();
        suited.sort_unstable_by(|a, b| b.rank().cmp(&a.rank()));
        let cards: [Card; 5] = suited.into_iter().take(5).collect::<Vec<Card>>().try_into().unwrap();
        return Hand { type_: HandType::Flush, cards };
    }

    if let Some(high) = straight_high {
        return Hand { type_: HandType::Straight, cards: pick_straight_cards(high, None) };
    }

    if let Some(&rank) = trips.first() {
        let mut cards = cards_of_rank(rank, 3);
        cards.extend(kickers(&[rank], 2));
        return Hand { type_: HandType::Trips, cards: cards.try_into().unwrap() };
    }

    if pairs.len() >= 2 {
        let mut cards = cards_of_rank(pairs[0], 2);
        cards.extend(cards_of_rank(pairs[1], 2));
        cards.extend(kickers(&[pairs[0], pairs[1]], 1));
        return Hand { type_: HandType::TwoPair, cards: cards.try_into().unwrap() };
    }

    if let Some(&rank) = pairs.first() {
        let mut cards = cards_of_rank(rank, 2);
        cards.extend(kickers(&[rank], 3));
        return Hand { type_: HandType::Pair, cards: cards.try_into().unwrap() };
    }

    Hand { type_: HandType::HighCard, cards: kickers(&[], 5).try_into().unwrap() }
}
