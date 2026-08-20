// betting rules, game state machine, pot logic
use crate::core::{Card, HandType, Rank};
use std::collections::HashMap;


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

fn evaluate_hand(hole_cards: Vec::<Card>, community_cards: Vec::<Card>) -> HandType {
    // Simple implementation for now, hard code checks.
    // Later use 2 + 2 or Cactus Kev alg

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
