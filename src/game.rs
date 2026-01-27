use crate::card::*;
use crate::player::PlayerTurn::*;
use crate::player::*;

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

pub struct Game {
    pub discard: Vec<Card>,
    pub players: Vec<Player>,
    pub turn: PlayerTurn,
    pub hand_size: HandSize,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            discard: vec![],
            players: vec![],
            turn: Player1,
            hand_size: HandSize::Free,
        }
    }
}

impl Game {
    pub fn valid_discard(&self, sub_hand: &[Card]) -> bool {
        // 1. determine HandSize and validate it against the current game state
        let current_size = match HandSize::try_from(sub_hand.len()) {
            Ok(size) if self.hand_size == HandSize::Free || self.hand_size == size => size,
            _ => return false,
        };

        // 2. logic for N-of-a-kind (Singles, Doubles, Triples)
        if current_size != HandSize::FiveCardHand {
            let is_set = sub_hand.windows(2).all(|w| w[0].rank == w[1].rank);
            if !is_set {
                return false;
            }

            return self.hand_size == HandSize::Free
                || sub_hand.iter().max() > self.hand_on_top().iter().max();
        }

        // 3. logic for five card hands
        let Some(hand_rank) = get_five_card_hand_rank(sub_hand) else {
            return false;
        };

        if self.hand_size == HandSize::Free {
            return true;
        }

        let Some(discard_rank) = get_five_card_hand_rank(self.hand_on_top()) else {
            return false;
        };

        // compares enum variant first, then the inner Card rank/suit
        hand_rank > discard_rank
    }

    pub fn hand_on_top(&self) -> &[Card] {
        let size = self.hand_size as usize;
        if self.discard.len() < size {
            &[]
        } else {
            &self.discard[self.discard.len() - size..]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::card;

    use super::*;

    #[test]
    fn test_valid_discard() {
        let mut game = Game {
            hand_size: HandSize::Doubles,
            discard: vec![
                Card {
                    rank: CardRank::Three,
                    suit: CardSuit::Clubs,
                },
                Card {
                    rank: CardRank::Three,
                    suit: CardSuit::Diamonds,
                },
            ],
            ..Default::default()
        };

        let mut sub_hand = vec![
            Card {
                rank: CardRank::Three,
                suit: CardSuit::Spades,
            },
            Card {
                rank: CardRank::Three,
                suit: CardSuit::Hearts,
            },
        ];
        assert!(!game.valid_discard(&sub_hand));

        sub_hand[1].suit = CardSuit::Diamonds;
        game.discard[1].suit = CardSuit::Hearts;
        assert!(game.valid_discard(&sub_hand));
    }

    #[test]
    fn test_hand_size_transitions() {
        let game = Game::default(); // starts as HandSize::Free

        // branch: Free -> Singles (valid)
        assert!(game.valid_discard(&[card::tests::card(CardRank::Five, CardSuit::Clubs)]));

        // branch: Free -> invalid count (4 cards)
        let four_cards = vec![card::tests::card(CardRank::Five, CardSuit::Clubs); 4];
        assert!(!game.valid_discard(&four_cards));

        // branch: Free -> Doubles (valid)
        let pair = vec![
            card::tests::card(CardRank::Five, CardSuit::Clubs),
            card::tests::card(CardRank::Five, CardSuit::Diamonds),
        ];
        assert!(game.valid_discard(&pair));
    }

    #[test]
    fn test_rank_matching_for_multiples() {
        // branch: fixed HandSize::Doubles, but input has mismatched ranks
        let game = Game {
            hand_size: HandSize::Doubles,
            ..Default::default()
        };

        let mismatch = vec![
            card::tests::card(CardRank::Five, CardSuit::Clubs),
            card::tests::card(CardRank::Six, CardSuit::Clubs),
        ];
        assert!(
            !game.valid_discard(&mismatch),
            "doubles must have matching ranks"
        );

        let match_pair = vec![
            card::tests::card(CardRank::Five, CardSuit::Clubs),
            card::tests::card(CardRank::Five, CardSuit::Diamonds),
        ];
        assert!(
            game.valid_discard(&match_pair),
            "matching ranks should pass"
        );
    }

    #[test]
    fn test_comparison_logic() {
        // setup game with a 3 of spades on top (singles)
        let game = Game {
            hand_size: HandSize::Singles,
            discard: vec![card::tests::card(CardRank::Four, CardSuit::Spades)],
            ..Default::default()
        };

        // branch: lower rank played (invalid)
        assert!(!game.valid_discard(&[card::tests::card(CardRank::Three, CardSuit::Spades)]));

        // branch: same rank, higher suit played (valid)
        // (assuming Card implements Ord where Diamonds > Hearts > Spades > Clubs)
        assert!(game.valid_discard(&[card::tests::card(CardRank::Four, CardSuit::Hearts)]));

        // branch: higher rank played (valid)
        assert!(game.valid_discard(&[card::tests::card(CardRank::Five, CardSuit::Spades)]));

        // branch: same rank, lower suit played (invalid)
        assert!(!game.valid_discard(&[card::tests::card(CardRank::Four, CardSuit::Clubs)]));
    }

    #[test]
    fn test_free_hand_ignores_discard() {
        // branch: hand_size == HandSize::Free
        // even if the played card is lower than the discard, it should be valid
        let game = Game {
            hand_size: HandSize::Free,
            discard: vec![card::tests::card(CardRank::Ace, CardSuit::Spades)],
            ..Default::default()
        };

        let low_card = vec![card::tests::card(CardRank::Three, CardSuit::Clubs)];
        assert!(
            game.valid_discard(&low_card),
            "free turn should allow any valid hand size"
        );
    }

    #[test]
    fn test_invalid_five_card_combinations() {
        let game = Game::default();

        // not a straight (gap), not a flush (mixed), no triplets/quads
        let junk_hand = vec![
            card::tests::card(CardRank::Two, CardSuit::Clubs),
            card::tests::card(CardRank::Four, CardSuit::Diamonds),
            card::tests::card(CardRank::Six, CardSuit::Hearts),
            card::tests::card(CardRank::Eight, CardSuit::Spades),
            card::tests::card(CardRank::Ten, CardSuit::Clubs),
        ];

        assert!(!game.valid_discard(&junk_hand));
    }

    #[test]
    fn test_five_card_hierarchy_comparison() {
        let straight = vec![
            card::tests::card(CardRank::Ten, CardSuit::Clubs),
            card::tests::card(CardRank::Jack, CardSuit::Diamonds),
            card::tests::card(CardRank::Queen, CardSuit::Hearts),
            card::tests::card(CardRank::King, CardSuit::Spades),
            card::tests::card(CardRank::Ace, CardSuit::Clubs),
        ];

        let low_flush = vec![
            card::tests::card(CardRank::Two, CardSuit::Clubs),
            card::tests::card(CardRank::Four, CardSuit::Clubs),
            card::tests::card(CardRank::Five, CardSuit::Clubs),
            card::tests::card(CardRank::Seven, CardSuit::Clubs),
            card::tests::card(CardRank::Eight, CardSuit::Clubs),
        ];

        let game = Game {
            hand_size: HandSize::FiveCardHand,
            discard: straight,
            ..Default::default()
        };

        // a flush beats a straight even if the straight has higher ranks
        assert!(game.valid_discard(&low_flush));
    }

    #[test]
    fn test_weighted_five_card_comparison() {
        // top hand: full house (three 5s, two aces)
        let discard_fh = vec![
            card::tests::card(CardRank::Five, CardSuit::Clubs),
            card::tests::card(CardRank::Five, CardSuit::Diamonds),
            card::tests::card(CardRank::Five, CardSuit::Hearts),
            card::tests::card(CardRank::Ace, CardSuit::Spades),
            card::tests::card(CardRank::Ace, CardSuit::Clubs),
        ];

        let game = Game {
            hand_size: HandSize::FiveCardHand,
            discard: discard_fh,
            ..Default::default()
        };

        // player attempts full house (three 6s, two 2s)
        let player_fh = vec![
            card::tests::card(CardRank::Six, CardSuit::Clubs),
            card::tests::card(CardRank::Six, CardSuit::Diamonds),
            card::tests::card(CardRank::Six, CardSuit::Hearts),
            card::tests::card(CardRank::Two, CardSuit::Spades),
            card::tests::card(CardRank::Two, CardSuit::Clubs),
        ];
        // 6s > 5s: this should be valid
        assert!(game.valid_discard(&player_fh));

        // player attempts four of a kind (Four 3s, One 9)
        let player_foak = vec![
            card::tests::card(CardRank::Three, CardSuit::Clubs),
            card::tests::card(CardRank::Three, CardSuit::Diamonds),
            card::tests::card(CardRank::Three, CardSuit::Hearts),
            card::tests::card(CardRank::Three, CardSuit::Spades),
            card::tests::card(CardRank::Nine, CardSuit::Clubs),
        ];
        // four of a kind (rank 3) > full house (rank 2)
        assert!(game.valid_discard(&player_foak));
    }
}
