use crate::PlayerTurn::Player1;
use card::*;
use game::*;
use itertools::Itertools;
use itertools::izip;
use player::*;
use rand::prelude::*;

mod card;
mod game;
mod player;

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
