use itertools::Itertools;
use rand::Rng;

use crate::collection::Card;
use crate::game::card::CardType;
use crate::game::state::State;
use crate::game::unordered_pile::UnorderedPile;
use crate::game::ManaPool;
use crate::trial::Rand;

#[allow(unused)]
pub trait Strategy {
    fn mulligan_hand(&mut self, state: &State) -> bool { false }
    fn land_drop(&mut self, state: &State) -> Option<Card> { None }
    fn card_plays(&mut self, state: &State) -> Vec<(Card, ManaPool)> { vec![] }
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

    fn card_plays(&mut self, state: &State) -> Vec<(Card, ManaPool)> { 
        let available_mana = state.available_mana();
        let potential_plays = state.hand.iter().collect_vec();
        card_play_strategies::naive_greedy(available_mana, potential_plays, |card| {
            let cost = card.data().cost.unwrap_or_default();
            cost.mana_value() as u32
        })
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

mod land_drop_strategies {
    use super::*;


    pub fn random_land(rng: &mut Rand, state: &State) -> Option<Card> {
        pick_random_filtered(rng, &state.hand, |c| c.data().card_type == CardType::Land)
    }

}
#[allow(dead_code)]
mod card_play_strategies {
    use crate::game::ManaPool;

    use super::*;

    pub fn naive_greedy<F: FnMut(Card) -> u32>(mut available_mana: ManaPool, mut legal_plays: Vec<Card>, mut utility: F) -> Vec<(Card, ManaPool)> {
        let mut plays = vec![];

        loop {
            log::debug!("in naive greedy algorithm, available mana: {available_mana}");
            log::debug!("legal plays before filtering: {}", legal_plays.len());
            // filter pick the best thing to play first
            legal_plays.sort_by_key(|card| utility(*card));
            // pick a card to play
            let Some(candidate) = legal_plays.pop() else {
                log::debug!("can't pick a card to play, returnning");
                break;
            };
            let Some(mana_cost) = candidate.data().cost.clone() else {
                log::debug!("tried to cast {candidate:?} without a cost, skipping");
                continue;
            };
            let mut payment_options = available_mana.payment_methods_for(mana_cost);
            let Some(payment) = payment_options.next() else {
                log::debug!("no ways to pay for {mana_cost}, skipping");
                continue;
            };
            let Some(remaining_mana) = available_mana - payment else {
                log::warn!("tried to cast {candidate:?}, did not have enough available mana");
                continue;
            };
            available_mana = remaining_mana;
            plays.push((candidate, payment));
        }

        plays
    }

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
