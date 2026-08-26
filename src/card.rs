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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card(u8);
impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self(((rank as u8) << 2) | (suit as u8))
    }

    // Build card from a raw u8
    pub(crate) fn from_raw(byte: u8) -> Self {
        Self(byte)
    }

    pub fn rank(&self) -> u8 {
        self.0 >> 2
    }

    pub fn suit(&self) -> u8 {
        self.0 & 0b11
    }
}
