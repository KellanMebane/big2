use card::*;
use itertools::Itertools;
use itertools::izip;
use player::PlayerTurn::*;
use player::*;
use rand::prelude::*;

mod card;
mod player;

struct Game {
    discard: Vec<Card>,
    players: Vec<Player>,
    turn: PlayerTurn,
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
        println!("{:?}", game.turn);

        game.discard
            .push(game.players[game.turn as usize].hand.pop().unwrap());

        // basic discard logic going for game struct
        // next should work on rules for discarding (start with singles)

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
