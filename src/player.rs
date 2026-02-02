use std::collections::HashMap;

use crate::card::*;

pub struct Player {
    pub hand: Vec<Card>,
}

impl Player {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { hand: Vec::new() }
    }

    //  TODO: decide whether hand validation should be done here or not
    //  could probably just include valid hand check before valid play check
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

    //  TODO: this only works for 1-3 card hands
    fn grab_smallest_hand_of_size(
        &self,
        hand_size: &HandSize,
        card_on_top: &Card,
    ) -> Option<Vec<Card>> {
        // temporarily ignores these hand sizes
        match hand_size {
            HandSize::Free | HandSize::FiveCardHand => return None,
            _ => {}
        }

        let mut possible_hands: Vec<Vec<Card>> = vec![];
        let mut rank_to_count: HashMap<CardRank, u8> = HashMap::new();

        for card in &self.hand {
            *rank_to_count.entry(card.rank).or_insert(0) += 1;
        }

        for (rank, count) in rank_to_count.iter() {
            //  NOTE: this means play doubles if triples are held
            //  maybe should rng whether this passes
            if *count >= *hand_size as u8 {
                // append all cards of this rank in hand to possible_hands
                let possible_hand: Vec<Card> = self
                    .hand
                    .iter()
                    .cloned()
                    .filter(|card| card.rank == *rank)
                    .take(*hand_size as usize) // <-- "play down" hand size
                    .collect();

                possible_hands.push(possible_hand);
            }
        }

        possible_hands.sort_by(|a, b| a[0].cmp(&b[0]));

        for possible_hand in possible_hands {
            if possible_hand[0] > *card_on_top {
                return Some(possible_hand);
            }
        }

        None
    }

    pub fn auto_pick_hand(&self, hand_size: &HandSize, card_on_top: &Card) -> Option<Vec<Card>> {
        let mut picked = Vec::new();

        /*
        ideas:
        - should do autopick first, then interactive pick
        - pass in hand type and latest card
        - if no playable hand, return None, which is a pass
        */

        /*
        should look for lowest playable hand per size.
        map ranks to counts and find all ranks that match.
        play lowest valid hand.
        if no lowest, return None.
        */
        //  TODO: fill out the other arms here
        match hand_size {
            HandSize::Free => {}
            HandSize::Singles | HandSize::Doubles | HandSize::Triples => {
                return self.grab_smallest_hand_of_size(hand_size, card_on_top);
            }
            HandSize::FiveCardHand => {}
        }

        Some(picked)
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
