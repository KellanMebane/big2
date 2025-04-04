use crate::card::*;

pub struct Player {
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new() -> Self {
        Self { hand: Vec::new() }
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
