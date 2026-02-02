use std::{collections::HashMap, fmt};

use enum_utils::IterVariants;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, IterVariants, Clone, Hash, Copy)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, IterVariants, Hash, Clone, Copy)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Card {
    pub rank: CardRank,
    pub suit: CardSuit,
}

impl Card {
    // helper to generate a basic card quickly
    pub fn card(rank: CardRank, suit: CardSuit) -> Card {
        Card { rank, suit }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum FiveCardRank {
    Straight(Card) = 0,
    Flush(Card) = 1,
    FullHouse(Card) = 2,
    FourOfAKind(Card) = 3,
    StraightFlush(Card) = 4,
}

//  NOTE: might be useful...
impl FiveCardRank {
    // Helper to extract the inner Card for comparison
    fn _lead_card(&self) -> Card {
        match *self {
            Self::Straight(c)
            | Self::Flush(c)
            | Self::FullHouse(c)
            | Self::FourOfAKind(c)
            | Self::StraightFlush(c) => c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandSize {
    Free = 0,
    Singles = 1,
    Doubles = 2,
    Triples = 3,
    FiveCardHand = 5,
}

impl TryFrom<usize> for HandSize {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(HandSize::Singles),
            2 => Ok(HandSize::Doubles),
            3 => Ok(HandSize::Triples),
            5 => Ok(HandSize::FiveCardHand),
            _ => Err(()),
        }
    }
}

//  TODO: need to implement wrap-around logic for straights
pub fn get_five_card_hand_rank(sub_hand: &[Card]) -> Option<FiveCardRank> {
    if sub_hand.len() != 5 {
        return None;
    }

    let is_flush = sub_hand.windows(2).all(|w| w[0].suit == w[1].suit);
    let is_straight = {
        let mut ranks: Vec<_> = sub_hand.iter().map(|c| c.rank as u32).collect();
        ranks.sort_unstable();
        ranks.windows(2).all(|w| w[1] == w[0] + 1)
    };

    let max_card = *sub_hand.iter().max()?;

    if is_straight && is_flush {
        return Some(FiveCardRank::StraightFlush(max_card));
    }
    if is_straight {
        return Some(FiveCardRank::Straight(max_card));
    }
    if is_flush {
        return Some(FiveCardRank::Flush(max_card));
    }

    // handle full house and four of a kind
    let mut counts = HashMap::new();
    for card in sub_hand {
        *counts.entry(card.rank).or_insert(0) += 1;
    }

    let (&lead_rank, &max_count) = counts.iter().max_by_key(|&(&r, &c)| (c, r))?;
    let lead_card = *sub_hand.iter().filter(|c| c.rank == lead_rank).max()?;

    match max_count {
        4 => Some(FiveCardRank::FourOfAKind(lead_card)),
        3 => Some(FiveCardRank::FullHouse(lead_card)),
        _ => None,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_five_card_rank_detection() {
        // straight: 3, 4, 5, 6, 7 mixed suits
        let straight = vec![
            Card::card(CardRank::Three, CardSuit::Clubs),
            Card::card(CardRank::Four, CardSuit::Diamonds),
            Card::card(CardRank::Five, CardSuit::Hearts),
            Card::card(CardRank::Six, CardSuit::Spades),
            Card::card(CardRank::Seven, CardSuit::Clubs),
        ];
        assert_eq!(
            get_five_card_hand_rank(&straight),
            Some(FiveCardRank::Straight(Card::card(
                CardRank::Seven,
                CardSuit::Clubs
            )))
        );

        // flush: all diamonds, non-sequential
        let flush = vec![
            Card::card(CardRank::Three, CardSuit::Diamonds),
            Card::card(CardRank::Five, CardSuit::Diamonds),
            Card::card(CardRank::Eight, CardSuit::Diamonds),
            Card::card(CardRank::Ten, CardSuit::Diamonds),
            Card::card(CardRank::King, CardSuit::Diamonds),
        ];
        assert_eq!(
            get_five_card_hand_rank(&flush),
            Some(FiveCardRank::Flush(Card::card(
                CardRank::King,
                CardSuit::Diamonds
            )))
        );

        // full house: three 8s, two 2s
        let full_house = vec![
            Card::card(CardRank::Eight, CardSuit::Clubs),
            Card::card(CardRank::Eight, CardSuit::Diamonds),
            Card::card(CardRank::Eight, CardSuit::Spades),
            Card::card(CardRank::Two, CardSuit::Hearts),
            Card::card(CardRank::Two, CardSuit::Clubs),
        ];
        // full house is identified by the triplet's rank
        match get_five_card_hand_rank(&full_house) {
            Some(FiveCardRank::FullHouse(c)) => assert_eq!(c.rank, CardRank::Eight),
            _ => panic!("Expected Full House"),
        }

        //  TODO: need four of a kind
    }
}
