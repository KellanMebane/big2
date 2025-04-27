use crate::card::*;

pub struct Player {
    pub hand: Vec<Card>,
}

impl Player {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { hand: Vec::new() }
    }

    // [TODO] add more logic to play hand to ensure it's a valid hand

    pub fn play_hand(&mut self, sub_hand: &[Card]) -> Vec<Card> {
        let mut played = Vec::new();

        for card_to_play in sub_hand {
            let card_position = self.hand.iter().position(|el| el == card_to_play);
            match card_position {
                Some(pos) => played.push(self.hand.remove(pos)),
                None => {}
            };
        }

        played
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PlayerTurn {
    Player1 = 0,
    Player2 = 1,
    Player3 = 2,
    Player4 = 3,
}

impl PlayerTurn {
    pub fn next_player(&mut self) {
        use PlayerTurn::*;
        *self = match self {
            Player1 => Player2,
            Player2 => Player3,
            Player3 => Player4,
            Player4 => Player1,
        };
    }
}
