use crate::metrics::MetricsData;
use crate::game::{state::State, Card};


#[allow(unused)]
pub trait Watcher {
    fn opening_hand(&self, state: &State, metrics: &mut MetricsData) { }

    fn turn_end(&self, state: &State, metrics: &mut MetricsData) { }

    fn game_end(&self, state: &State, metrics: &mut MetricsData) { }

    fn land_drop(&self, land_drop: Card, state: &State, metrics: &mut MetricsData) { }

    fn card_play(&self, card_play: Card, state: &State, metrics: &mut MetricsData) { }
}

#[derive(Clone)]
pub struct WatcherImpl;
impl Watcher for WatcherImpl {
    fn opening_hand<'a>(&self, state: &State, metrics: &mut MetricsData) { 
        metrics.add_count("opening-hand::lands", state.num_lands_in_hand() as u32);
    }

    fn land_drop(&self, _card: Card, _state: &State, metrics: &mut MetricsData) {
        metrics.add("land-drops");
    }

    fn card_play(&self, card_play: Card, state: &State, metrics: &mut MetricsData) {
        metrics.add("card-plays");

        metrics.set(("turn-played", card_play), state.turn);

        if metrics.get("card-plays") == 7 {
            metrics.add_count("turn-to-reach-7-plays", state.turn);
        }

    }

    fn game_end(&self, state: &State, metrics: &mut MetricsData) {
        metrics.add_count("num-turns", state.turn);
        metrics.add_count("num-mulligans", state.num_mulligans_taken);
    }
}
