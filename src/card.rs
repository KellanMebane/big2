use std::fmt;

use enum_utils::IterVariants;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, IterVariants, Clone, Copy)]
pub enum CardSuit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl fmt::Display for CardSuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suit = match self {
            CardSuit::Clubs => "♣️",
            CardSuit::Spades => "♠️",
            CardSuit::Hearts => "♥️",
            CardSuit::Diamonds => "♦️",
        };

        write!(f, "{}", suit)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, IterVariants, Clone, Copy)]
pub enum CardRank {
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Two,
}

impl fmt::Display for CardRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank: &str = match self {
            CardRank::Three => "3",
            CardRank::Four => "4",
            CardRank::Five => "5",
            CardRank::Six => "6",
            CardRank::Seven => "7",
            CardRank::Eight => "8",
            CardRank::Nine => "9",
            CardRank::Ten => "10",
            CardRank::Jack => "J",
            CardRank::Queen => "Q",
            CardRank::King => "K",
            CardRank::Ace => "A",
            CardRank::Two => "2",
        };

        write!(f, "{}", rank)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct Card {
    pub rank: CardRank,
    pub suit: CardSuit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

pub fn get_deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for suit in CardSuit::iter() {
        for rank in CardRank::iter() {
            deck.push(Card { rank, suit });
        }
    }
    deck
}
