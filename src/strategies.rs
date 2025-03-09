use crate::game::card_play::CardPlay;
use crate::game::state::State;
use crate::trial::Rand;

pub mod payment_solver;

#[allow(unused)]
pub trait Strategy {
    fn mulligan_hand(&mut self, state: &State) -> bool { false }
    fn card_plays(&mut self, state: &State) -> Vec<CardPlay> { vec![] }
}

#[derive(Clone)]
pub struct DefaultStrategy;
impl Strategy for DefaultStrategy { }

#[derive(Clone)]
pub struct StrategyImpl {
    pub rng: Rand
}
impl Strategy for StrategyImpl {
    fn mulligan_hand(&mut self, state: &State) -> bool { 
        mulligan_strategies::between_3_and_4_lands(state)
    }
    fn card_plays(&mut self, state: &State) -> Vec<CardPlay> { 
        let plays = card_play_strategies::play_a_land_and_a_card(
            state, 
            &utility_functions::mana_value_or_fixed_land,
        );
        plays
    }
}

mod utility_functions {
    use crate::{collection::Card, game::CardType};

    pub type Utility = u32;

    pub fn mana_value(card: Card) -> Utility {
        let Some(cost) = &card.data().cost else {
            return 0;
        };
        cost.mana_value() as _
    }

    const UTILITY_OF_LAND_DROP: Utility = 1;

    pub fn mana_value_or_fixed_land(card: Card) -> Utility {
        if CardType::Land == card.data().card_type {
            return UTILITY_OF_LAND_DROP
        } 
        mana_value(card)
    }
}

#[allow(dead_code)]
mod mulligan_strategies {
    use super::*;

    pub fn between_3_and_4_lands(state: &State) -> bool {
        if state.num_mulligans_taken >= 3 {
            log::debug!("refusing to take a mulligan #{}", state.num_mulligans_taken);
            return false;
        }
        let land_count = state.num_lands_in_hand();
        let good = 3 <= land_count && land_count <= 5;
        log::debug!("saw hand with {} cards and {land_count} lands, on mulligan #{}, good={good}", state.hand.size(), state.num_mulligans_taken);

        !good
    }
}

mod card_play_strategies;

