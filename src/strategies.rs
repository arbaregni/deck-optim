use itertools::Itertools;
use rand::Rng;

use crate::collection::Card;
use crate::game::card::CardType;
use crate::game::card_play::CardPlay;
use crate::game::state::State;
use crate::game::unordered_pile::UnorderedPile;
use crate::game::{ManaPool, Zone};
use crate::trial::Rand;

#[allow(unused)]
pub trait Strategy {
    fn mulligan_hand(&mut self, state: &State) -> bool { false }
    fn land_drop(&mut self, state: &State) -> Option<Card> { None }
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
        let available_mana = state.available_mana();
        let potential_plays = state.legal_card_plays().collect_vec();


        let mut plays = Vec::with_capacity(2);

        if let Some(land_drop) = land_drop_strategies::random_land(&mut self.rng, state) {
            let land_drop = CardPlay {
                card: land_drop,
                zone: Zone::Hand,
                payment: ManaPool::empty(),
            };
            plays.push(land_drop);
        }

        card_play_strategies::naive_greedy(
            &mut plays,
            available_mana,
            potential_plays, 
            |card| {
                let cost = card.data().cost.unwrap_or_default();
                cost.mana_value() as u32
            }
        );

        plays
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
    use crate::game::{card_play::CardPlay, ManaPool};
    use crate::opt_utils::OptExt;

    use super::*;

    pub fn naive_greedy<F: FnMut(Card) -> u32>(plays: &mut Vec<CardPlay>, mut available_mana: ManaPool, mut legal_plays: Vec<CardPlay>, mut utility: F) {
        log::debug!("begin naive greedy algorithm, available mana: {available_mana} and {} potential plays", legal_plays.len());
        loop {
            log::debug!("   picking from {} candidate card plays, available mana: {available_mana}", legal_plays.len());
            // filter pick the best thing to play first
            legal_plays.sort_by_key(|card_play| utility(card_play.card));
            // pick a card to play
            let Some(CardPlay { card: candidate, zone, payment: _ }) = legal_plays.pop() else {
                log::debug!("       can't pick a card to play, returning");
                break;
            };
            log::debug!("   evaluating candidate: {candidate:?} with cost {}", candidate.data().cost.display());
            let Some(mana_cost) = candidate.data().cost.clone() else {
                log::debug!("       candidate doesn't have a cost, can't play");
                continue;
            };
            let mut payment_options = available_mana.payment_methods_for(mana_cost);
            let Some(payment) = payment_options.next() else {
                log::debug!("       no ways to pay for {mana_cost} with {available_mana}, skipping");
                continue;
            };
            let Some(remaining_mana) = available_mana - payment else {
                log::warn!("       invalid payment option: can not take {payment} from {available_mana}, not enough mana to cast");
                continue;
            };
            available_mana = remaining_mana;
            log::debug!("       playing {candidate:?} with {payment}");
            plays.push(CardPlay {
                card: candidate, 
                zone,
                payment
            });
        }

        log::debug!("ending naive greedy algorithm, available mana: {available_mana}, cards being played: {}", plays.len());
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
