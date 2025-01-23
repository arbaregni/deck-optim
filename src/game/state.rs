use rand::Rng;

use crate::game::card::Card;
use crate::game::card::CardType;
use crate::trial::Rand;
use crate::game::{
    Hand, Library, UnorderedPile
};


const PROB_OF_GOING_FIRST: f64 = 0.5;
const HAND_SIZE: u32 = 7;

pub struct State {
    pub turn: u32,
    pub draw_on_first_turn: bool,
    pub num_mulligans_taken: u32,
    pub game_loss: bool,

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
                self.hand.add(card, 1);
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
                self.lands.add(card, 1);
            }
            CardType::Instant => {
                self.lands.add(card, 1);
            }
        }
    }


    // some measures

    pub fn num_lands_in_hand(&self) -> usize {
        self.hand
            .iter()
            .filter(|c| c.data().card_type == CardType::Land)
            .count()
    }
}
