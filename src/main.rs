use card::*;
use itertools::Itertools;
use itertools::izip;
use player::PlayerTurn::*;
use player::*;
use rand::prelude::*;

mod card;
mod player;

#[derive(Clone, Copy)]
enum HandSize {
    Free = 0,
    Singles = 1,
    Doubles = 2,
    Triples = 3,
    FiveCardHand = 5,
}

struct Game {
    discard: Vec<Card>,
    players: Vec<Player>,
    turn: PlayerTurn,
    hand_size: HandSize,
}

impl Game {
    #[allow(dead_code)]
    fn valid_discard(&self, sub_hand: &[Card]) -> bool {
        match self.hand_size {
            HandSize::Free => return true,
            _ => {
                let biggest_played = sub_hand.iter().max();
                let biggest_discard = self.hand_on_top().iter().max();

                return biggest_played > biggest_discard;
            }
        };
        false
    }

    fn hand_on_top(&self) -> &[Card] {
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

        for _ in 0..3 {
            println!("----Turn: {:?}-----", game.turn);

            let turn_size = match game.hand_size {
                HandSize::Free => {
                    game.hand_size = HandSize::Doubles;
                    HandSize::Doubles as usize
                }
                hs => hs as usize,
            };

            let player = &game.players[game.turn as usize];

            let num_cards: usize = match player.hand.len() as isize - turn_size as isize {
                ..0 => 0,
                non_zero => non_zero as usize,
            };

            let player_hand = player.hand[..turn_size].to_vec();

            println!("{:?}", player_hand);
            println!("valid hand?: {}", game.valid_discard(&player_hand));

            if game.valid_discard(&player_hand) {
                let discards = game.players[game.turn as usize].play_hand(&player_hand);
                game.discard.extend(discards);
            }

            // discard logic is rudamentory, does not factor 5 card specifics,
            // but work for singles, doubles, and triples

            game.turn.next_player();
        }

        let s: String = game
            .discard
            .iter()
            .map(|card| format!("[{}]", card))
            .collect::<Vec<String>>()
            .join(" ");

        println!("{}", s);
    }
}
