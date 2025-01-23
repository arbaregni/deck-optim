use itertools::Itertools;
use rand::Rng;

use crate::game::card::Card;
use crate::game::card::CardType;
use crate::trial::Rand;
use crate::game::{
    Hand, Library, UnorderedPile
};

use super::ManaPool;


const PROB_OF_GOING_FIRST: f64 = 0.5;
const HAND_SIZE: u32 = 7;

pub struct State {
    pub turn: u32,
    pub draw_on_first_turn: bool,
    pub num_mulligans_taken: u32,
    pub game_loss: bool,
    pub turn_state: TurnState,

    pub max_land_drops_per_turn: u32,

    // 
    // ZONES
    //

    // Library
    pub library: Library,

    // Hand
    pub hand: Hand,
    
    // Battle field
    pub lands: UnorderedPile,
    pub permanents: UnorderedPile,

    // Graveyard
    pub graveyard: UnorderedPile, // technically it's ordered, but whatever

}

impl State {
    pub fn new(deck: UnorderedPile, rng: &mut Rand) -> State {
        State {
            library: deck.to_ordered(rng),
            hand: Hand::empty(),
            lands: UnorderedPile::empty(),
            permanents: UnorderedPile::empty(),
            graveyard: UnorderedPile::empty(),
            turn: 0,
            draw_on_first_turn: rng.gen_bool(PROB_OF_GOING_FIRST),
            num_mulligans_taken: 0,
            game_loss: false,
            max_land_drops_per_turn: 1,
            turn_state: TurnState::new(),
        }
    }
    pub fn draw_hand(&mut self) {
        if self.num_mulligans_taken >= HAND_SIZE {
            log::warn!("taking more mulligans than hand size allowed, ignoring extra mulligans");
            return;
        }
        let hand_size = HAND_SIZE - self.num_mulligans_taken;
        self.hand = self.library.draw_n(hand_size as usize).into();
    }
    pub fn shuffle_hand_into_library(&mut self, rng: &mut Rand) {
        self.library.add_to_top(&self.hand);
        self.library.shuffle(rng);
        self.hand = Hand::empty();
    }

    
    pub fn draw_to_hand(&mut self) {
        match self.library.draw() {
            Some(card) => {
                self.hand.add(card);
            }
            None => {
                self.game_loss = true;
            }
        }
    }

    pub fn play_card(&mut self, card: Card) {
        if !self.hand.remove(card) {
            log::error!("attempting to play card that is not present in hand");
            return;
        };

        match card.data().card_type {
            CardType::Land => {
                self.turn_state.land_drops_made += 1;
                self.lands.add(card);
            }
            CardType::Instant => {
                self.graveyard.add(card);
            }
        }
    }

    pub fn end_turn(&mut self) {
        self.turn_state.reset();
        self.turn += 1;
    }


    // some measures
    pub fn legal_card_plays(&self) -> impl Iterator<Item = Card> + '_ {
        self.hand
            .iter()
            .filter(|c| match c.data().card_type {
                // we can only play a land if we haven't used all our land drops yet
                CardType::Land => self.turn_state.land_drops_made < self.max_land_drops_per_turn,

                _ if c.data().cost.is_some() => {
                    // TODO: fix this hack
                    let mv = c.data().cost.as_ref().expect("just checked").mana_value() as usize;
                    let available_mana = self.lands.size() - self.turn_state.tapped.size();
                    mv <= available_mana
                }

                _ => {
                    log::error!("could not determine if playing card {c:?} was legal, with name {}", c.data().name);
                    false
                }
            })
    }
    pub fn available_mana(&self) -> ManaPool {
        self.lands
            .iter()
            .filter_map(|c| c.data().produces.clone())
            .sum()
    }
    pub fn num_lands_in_hand(&self) -> usize {
        self.hand
            .iter()
            .filter(|c| c.data().card_type == CardType::Land)
            .count()
    }

    pub fn num_lands_in_play(&self) -> usize {
        self.lands.size()
    }
}

/// For state that is reset every cleanup phase
pub struct TurnState {
    pub land_drops_made: u32,
    pub tapped: UnorderedPile,
}


impl TurnState {
    pub fn new() -> TurnState {
        TurnState {
            land_drops_made: 0,
            tapped: UnorderedPile::empty(),
        }
    }
    pub fn reset(&mut self) {
        self.land_drops_made = 0;
        self.tapped.clear();
    }
    pub fn mark_as_tapped(&mut self, card: Card) {
        if self.is_tapped(card) {
            log::error!("card is already tapped: {card:?}");
            return;
        }
        self.tapped.add(card);
    }
    pub fn is_tapped(&mut self, card: Card) -> bool {
        self.tapped.iter().contains(&card)
    }
}
