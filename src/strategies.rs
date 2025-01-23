use rand::Rng;

use crate::game::card::Card;
use crate::game::card::CardType;
use crate::game::state::State;
use crate::game::unordered_pile::UnorderedPile;
use crate::trial::Rand;

#[allow(unused)]
pub trait Strategy {
    fn mulligan_hand(&mut self, state: &State) -> bool { false }
    fn land_drop(&mut self, state: &State) -> Option<Card> { None }
    fn card_play(&mut self, state: &State) -> Option<Card> { None }
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

    fn land_drop(&mut self, state: &State) -> Option<Card> {
        land_drop_strategies::random_land(&mut self.rng, state)
    }

    fn card_play(&mut self, state: &State) -> Option<Card> { 
        card_play_strategies::random_nonland(&mut self.rng, state)
    }
}

mod mulligan_strategies {
    use super::*;

    pub fn between_3_and_4_lands(state: &State) -> bool {
        if state.num_mulligans_taken >= 3 {
            log::debug!("refusing to take a mulligan #{}", state.num_mulligans_taken);
            return false;
        }
        let land_count = state.num_lands_in_hand();
        let good = 3 <= land_count && land_count <= 4;
        log::debug!("saw hand with {} cards and {land_count} lands, on mulligan #{}, good={good}", state.hand.size(), state.num_mulligans_taken);

        !good
    }
}

mod land_drop_strategies {
    use super::*;


    pub fn random_land(rng: &mut Rand, state: &State) -> Option<Card> {
        pick_random_filtered(rng, &state.hand, |c| c.data().card_type == CardType::Land)
    }

}
mod card_play_strategies {
    use super::*;

    pub fn random_nonland(rng: &mut Rand, state: &State) -> Option<Card> {
        pick_random_filtered(rng, &state.hand, |c| c.data().card_type != CardType::Land)
    }

}



fn pick_random_filtered<F>(rng: &mut Rand, cards: &UnorderedPile, filter: F) -> Option<Card>
where F: Fn(&Card) -> bool,
{
    let count = cards.iter().filter(&filter).count();
    if count == 0 {
        return None;
    }
    let idx = rng.gen_range(0..count);
    cards
        .iter()
        .filter(&filter)
        .nth(idx)
}
