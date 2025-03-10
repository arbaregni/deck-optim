use itertools::Itertools;
use rand::Rng;

use crate::game::UnorderedPile;
use crate::opt_utils::OptExt;

use crate::strategies::payment_solver;
use crate::strategies::utility_functions::Utility;
use crate::collection::Card;
use crate::game::card::CardType;
use crate::game::card_play::CardPlay;
use crate::game::mana::ManaSource;
use crate::game::state::State;
use crate::trial::Rand;


#[derive(Debug,Clone)]
struct Soln {
    pub card_plays: Vec<CardPlay>,
    pub utility: Utility
}
impl Soln {
    fn replace_if_better(&mut self, other: Self) {
        if other.utility > self.utility {
            *self = other;
        }

    }
}

// The solution is a list of cards to play
pub type CardPlaySolution = Vec<CardPlay>;

pub fn play_a_land_and_a_card<F>(state: &State, utility_fn: &F) -> CardPlaySolution 
    where F: Fn(Card) -> Utility 
{
    let mut soln = Soln {
        card_plays: Vec::new(),
        utility: 0
    };

    state.legal_land_drops()
        .for_each(|land_drop| {
                
            let mut card_plays = vec![land_drop.clone()];
            let next = state.with_having_played(land_drop.clone());

            log::debug!("forecasting land drop - what if we played {:?}", land_drop.card);
            card_plays.extend(play_a_card(&next, utility_fn));

            let utility = card_plays
                .iter()
                .map(|card_play| card_play.card)
                .map(utility_fn)
                .sum();

            soln.replace_if_better(Soln {
                card_plays,
                utility
            });
    });

    soln.card_plays
}

pub fn play_a_card<F>(state: &State, utility_fn: &F) -> CardPlaySolution 
    where F: Fn(Card) -> Utility
{
    let mut plays = Vec::new();
    let available_mana = state.mana_sources().collect_vec();
    let legal_plays = state.legal_card_plays().collect_vec();

    naive_greedy(&mut plays, available_mana, legal_plays, utility_fn);

    plays
}

pub fn naive_greedy<F: Fn(Card) -> Utility>(plays: &mut Vec<CardPlay>, mut mana_sources: Vec<ManaSource>, mut legal_plays: Vec<CardPlay>, utility_fn: &F) {
    log::debug!("begin naive greedy algorithm, available mana: {mana_sources:?} and {} potential plays", legal_plays.len());
    loop {
        log::debug!("   picking from {} candidate card plays, available mana: {mana_sources:?}", legal_plays.len());
        // filter pick the best thing to play first
        legal_plays.sort_by_key(|card_play| utility_fn(card_play.card));
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

        // TODO: can we avoid this clone?
        let Some((payment, unused_mana)) = payment_solver::autotap_pay_for(mana_sources.clone(), &mana_cost) else {
            log::debug!("       no ways to pay for {mana_cost} with {mana_sources:?}, skipping");
            continue;
        };
        mana_sources = unused_mana;

        log::debug!("       playing {candidate:?} with {payment:?}");
        plays.push(CardPlay {
            card: candidate, 
            zone,
            payment: payment.mana_used
        });
    }

    log::debug!("ending naive greedy algorithm, available mana: {mana_sources:?}, cards being played: {}", plays.len());
}

#[allow(dead_code)]
pub fn random_nonland(rng: &mut Rand, state: &State) -> Option<Card> {
    pick_random_filtered(rng, &state.hand, |c| c.data().card_type != CardType::Land)
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
