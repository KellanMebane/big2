use card::*;
use itertools::Itertools;
use itertools::izip;
use player::PlayerTurn::*;
use player::*;
use rand::prelude::*;
use std::collections::HashMap;

mod card;
mod player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum HandSize {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum FiveCardRank {
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

struct Game {
    discard: Vec<Card>,
    players: Vec<Player>,
    turn: PlayerTurn,
    hand_size: HandSize,
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
        let hand_rank = match self.get_five_card_hand_rank(sub_hand) {
            Some(r) => r,
            None => return false,
        };

        if self.hand_size == HandSize::Free {
            return true;
        }

        let discard_rank = self
            .get_five_card_hand_rank(self.hand_on_top())
            .expect("top of discard must be valid 5-card hand");

        // compares enum variant first, then the inner Card rank/suit
        hand_rank > discard_rank
    }

    fn get_five_card_hand_rank(&self, sub_hand: &[Card]) -> Option<FiveCardRank> {
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

    fn hand_on_top(&self) -> &[Card] {
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

    // helper to generate a basic card quickly
    fn card(rank: CardRank, suit: CardSuit) -> Card {
        Card { rank, suit }
    }

    #[test]
    fn test_hand_size_transitions() {
        let game = Game::default(); // starts as HandSize::Free

        // branch: Free -> Singles (valid)
        assert!(game.valid_discard(&[card(CardRank::Five, CardSuit::Clubs)]));

        // branch: Free -> invalid count (4 cards)
        let four_cards = vec![card(CardRank::Five, CardSuit::Clubs); 4];
        assert!(!game.valid_discard(&four_cards));

        // branch: Free -> Doubles (valid)
        let pair = vec![
            card(CardRank::Five, CardSuit::Clubs),
            card(CardRank::Five, CardSuit::Diamonds),
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
            card(CardRank::Five, CardSuit::Clubs),
            card(CardRank::Six, CardSuit::Clubs),
        ];
        assert!(
            !game.valid_discard(&mismatch),
            "doubles must have matching ranks"
        );

        let match_pair = vec![
            card(CardRank::Five, CardSuit::Clubs),
            card(CardRank::Five, CardSuit::Diamonds),
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
            discard: vec![card(CardRank::Four, CardSuit::Spades)],
            ..Default::default()
        };

        // branch: lower rank played (invalid)
        assert!(!game.valid_discard(&[card(CardRank::Three, CardSuit::Spades)]));

        // branch: same rank, higher suit played (valid)
        // (assuming Card implements Ord where Diamonds > Hearts > Spades > Clubs)
        assert!(game.valid_discard(&[card(CardRank::Four, CardSuit::Hearts)]));

        // branch: higher rank played (valid)
        assert!(game.valid_discard(&[card(CardRank::Five, CardSuit::Spades)]));

        // branch: same rank, lower suit played (invalid)
        assert!(!game.valid_discard(&[card(CardRank::Four, CardSuit::Clubs)]));
    }

    #[test]
    fn test_free_hand_ignores_discard() {
        // branch: hand_size == HandSize::Free
        // even if the played card is lower than the discard, it should be valid
        let game = Game {
            hand_size: HandSize::Free,
            discard: vec![card(CardRank::Ace, CardSuit::Spades)],
            ..Default::default()
        };

        let low_card = vec![card(CardRank::Three, CardSuit::Clubs)];
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
            card(CardRank::Two, CardSuit::Clubs),
            card(CardRank::Four, CardSuit::Diamonds),
            card(CardRank::Six, CardSuit::Hearts),
            card(CardRank::Eight, CardSuit::Spades),
            card(CardRank::Ten, CardSuit::Clubs),
        ];

        assert!(!game.valid_discard(&junk_hand));
    }

    #[test]
    fn test_five_card_rank_detection() {
        let game = Game::default();

        // straight: 3, 4, 5, 6, 7 mixed suits
        let straight = vec![
            card(CardRank::Three, CardSuit::Clubs),
            card(CardRank::Four, CardSuit::Diamonds),
            card(CardRank::Five, CardSuit::Hearts),
            card(CardRank::Six, CardSuit::Spades),
            card(CardRank::Seven, CardSuit::Clubs),
        ];
        assert_eq!(
            game.get_five_card_hand_rank(&straight),
            Some(FiveCardRank::Straight(card(
                CardRank::Seven,
                CardSuit::Clubs
            )))
        );

        // flush: all diamonds, non-sequential
        let flush = vec![
            card(CardRank::Three, CardSuit::Diamonds),
            card(CardRank::Five, CardSuit::Diamonds),
            card(CardRank::Eight, CardSuit::Diamonds),
            card(CardRank::Ten, CardSuit::Diamonds),
            card(CardRank::King, CardSuit::Diamonds),
        ];
        assert_eq!(
            game.get_five_card_hand_rank(&flush),
            Some(FiveCardRank::Flush(card(
                CardRank::King,
                CardSuit::Diamonds
            )))
        );

        // full house: three 8s, two 2s
        let full_house = vec![
            card(CardRank::Eight, CardSuit::Clubs),
            card(CardRank::Eight, CardSuit::Diamonds),
            card(CardRank::Eight, CardSuit::Spades),
            card(CardRank::Two, CardSuit::Hearts),
            card(CardRank::Two, CardSuit::Clubs),
        ];
        // full house is identified by the triplet's rank
        match game.get_five_card_hand_rank(&full_house) {
            Some(FiveCardRank::FullHouse(c)) => assert_eq!(c.rank, CardRank::Eight),
            _ => panic!("Expected Full House"),
        }

        //  TODO: need four of a kind
    }

    #[test]
    fn test_five_card_hierarchy_comparison() {
        let straight = vec![
            card(CardRank::Ten, CardSuit::Clubs),
            card(CardRank::Jack, CardSuit::Diamonds),
            card(CardRank::Queen, CardSuit::Hearts),
            card(CardRank::King, CardSuit::Spades),
            card(CardRank::Ace, CardSuit::Clubs),
        ];

        let low_flush = vec![
            card(CardRank::Two, CardSuit::Clubs),
            card(CardRank::Four, CardSuit::Clubs),
            card(CardRank::Five, CardSuit::Clubs),
            card(CardRank::Seven, CardSuit::Clubs),
            card(CardRank::Eight, CardSuit::Clubs),
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
            card(CardRank::Five, CardSuit::Clubs),
            card(CardRank::Five, CardSuit::Diamonds),
            card(CardRank::Five, CardSuit::Hearts),
            card(CardRank::Ace, CardSuit::Spades),
            card(CardRank::Ace, CardSuit::Clubs),
        ];

        let game = Game {
            hand_size: HandSize::FiveCardHand,
            discard: discard_fh,
            ..Default::default()
        };

        // player attempts full house (three 6s, two 2s)
        let player_fh = vec![
            card(CardRank::Six, CardSuit::Clubs),
            card(CardRank::Six, CardSuit::Diamonds),
            card(CardRank::Six, CardSuit::Hearts),
            card(CardRank::Two, CardSuit::Spades),
            card(CardRank::Two, CardSuit::Clubs),
        ];
        // 6s > 5s: this should be valid
        assert!(game.valid_discard(&player_fh));

        // player attempts four of a kind (Four 3s, One 9)
        let player_four = vec![
            card(CardRank::Three, CardSuit::Clubs),
            card(CardRank::Three, CardSuit::Diamonds),
            card(CardRank::Three, CardSuit::Hearts),
            card(CardRank::Three, CardSuit::Spades),
            card(CardRank::Nine, CardSuit::Clubs),
        ];
        // four of a kind (rank 3) > full house (rank 2)
        assert!(game.valid_discard(&player_four));
    }
}

fn main() {
    let mut deck = get_deck();

    let mut rng = rand::rng();

    deck.shuffle(&mut rng);

    let mut game = Game {
        discard: Vec::new(),
        players: deck
            .into_iter()
            .chunks(13)
            .into_iter()
            .map(|chunk| chunk.collect())
            .map(|hand| Player { hand })
            .collect(),
        turn: Player1,
        hand_size: HandSize::Free,
    };

    game.players
        .iter_mut()
        .enumerate()
        .for_each(|(player_num, player)| {
            if player_num == 0 {
                player.hand.sort_by_key(|hand| (hand.suit, hand.rank));
            } else {
                player.hand.sort();
            }
        });

    for (p1_card, p2_card, p3_card, p4_card) in izip!(
        &game.players[0].hand,
        &game.players[1].hand,
        &game.players[2].hand,
        &game.players[3].hand
    ) {
        println!("{}\t{}\t{}\t{}", p1_card, p2_card, p3_card, p4_card);
    }

    println!();

    //  NOTE: this loop is only for testing valid_discard()
    for _ in 0..3 {
        println!("----|  Turn: {:?}  |-----", game.turn);

        let turn_size = match game.hand_size {
            //  NOTE: this defaults to doubles for testing valid_discard() only
            HandSize::Free => {
                game.hand_size = HandSize::Doubles;
                HandSize::Doubles as usize
            }
            hs => hs as usize,
        };

        game.hand_size = HandSize::Free;

        let player = &game.players[game.turn as usize];

        let player_hand = player.hand[..turn_size].to_vec();

        // println!("{:?}", player_hand);
        for card in &player_hand {
            print!("[{}] ", card);
        }
        println!();
        println!("valid hand?: {}", game.valid_discard(&player_hand));

        if game.valid_discard(&player_hand) {
            let discards = game.players[game.turn as usize].play_hand(&player_hand);
            game.hand_size = match discards.len() {
                1 => HandSize::Singles,
                2 => HandSize::Doubles,
                3 => HandSize::Triples,
                5 => HandSize::FiveCardHand,
                _ => HandSize::Free,
            };
            game.discard.extend(discards);
        }
        println!("hand size is {:?}", game.hand_size);

        game.turn.next_player();

        println!();
    }

    // print discard pile
    let s: String = game
        .discard
        .iter()
        .map(|card| format!("[{}]", card))
        .collect::<Vec<String>>()
        .join(" ");

    println!("{}", s);
}
