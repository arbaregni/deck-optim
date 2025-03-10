use crate::game::CardType;
use crate::metrics::{MetricsData, MetricsKey};
use crate::collection::Card;
use crate::game::state::State;


#[allow(unused)]
pub trait Watcher {
    fn opening_hand(&self, state: &State, metrics: &mut MetricsData) { }

    fn turn_end(&self, state: &State, metrics: &mut MetricsData) { }

    fn game_end(&self, state: &State, metrics: &mut MetricsData) { }

    fn card_play(&self, card_play: Card, state: &State, metrics: &mut MetricsData) { }
}

#[derive(Clone)]
pub struct WatcherImpl;
impl Watcher for WatcherImpl {
    fn opening_hand<'a>(&self, state: &State, metrics: &mut MetricsData) { 
        metrics.add_count("opening-hand::lands", state.num_lands_in_hand() as u32);
    }

    fn card_play(&self, card_play: Card, state: &State, metrics: &mut MetricsData) {
        if card_play.data().card_type == CardType::Land {
            metrics.add("land-drops");
        } else {
             metrics.add("card-plays");
        }

        metrics.set(
            MetricsKey::from("turn-played").card(card_play),
            state.turn
        );

        if metrics.total("card-plays") == 7 {
            metrics.add_count("turn-to-reach-7-plays", state.turn);
        }

    }

    fn turn_end(&self, state: &State, metrics: &mut MetricsData) {
        let available_mana = state.available_mana() as u32;
        metrics.set(
            MetricsKey::from("mana_on_turn").turn_num(state.turn),
            available_mana
        );
    }

    fn game_end(&self, state: &State, metrics: &mut MetricsData) {
        metrics.add_count("num-turns", state.turn);
        metrics.add_count("num-mulligans", state.num_mulligans_taken);
    }
}
