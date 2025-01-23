use rand::SeedableRng;

use crate::game::Library;
use crate::game::Hand;
use crate::game::state::State;
use crate::game::unordered_pile::UnorderedPile;
use crate::strategies::Strategy;
use crate::watcher::Watcher;
use crate::metrics::MetricsData;

pub type Rand = rand::rngs::StdRng;


const MAX_TURN: u32 = 50;

pub struct Trial {
    pub rng: Rand,
    pub state: State,
    pub metrics: MetricsData,
}

impl Trial {
    pub fn new(deck: UnorderedPile, mut rng: Rand) -> Self {
        let state = State::new(
            deck,
            &mut rng
        );
        Trial {
            rng,
            state,
            metrics: MetricsData::empty()
        }
    }
    pub fn library(&self) -> &Library {
        &self.state.library
    }

    pub fn hand(&self) -> &Hand {
        &self.state.hand
    }

    pub fn turn(&self) -> u32 {
        self.state.turn
    }

    pub fn run<S, W>(mut self, strategies: &mut S, watcher: &W) -> MetricsData
    where S: Strategy,
          W: Watcher
    {
        self.state.library.shuffle(&mut self.rng);

        self.state.draw_hand();

        while strategies.mulligan_hand(&self.state) {
            self.state.shuffle_hand_into_library(&mut self.rng);

            self.state.num_mulligans_taken += 1;
            self.state.draw_hand();

            if self.state.num_mulligans_taken >= 6 {
                log::warn!("strategy used up all mulligans");
                break;
            }
            
            // TODO: mulligan with london mulligan
        };
        
        watcher.opening_hand(&self.state, &mut self.metrics);

        while self.turn() <= MAX_TURN && !self.state.game_loss {
            let draw = self.turn() > 0 || self.state.draw_on_first_turn;
            if draw {
                self.state.draw_to_hand();
            }

            if let Some(land_drop) = strategies.land_drop(&self.state) {
                log::debug!("playing land: {land_drop:?}");
                watcher.land_drop(land_drop, &self.state, &mut self.metrics);
                self.state.play_card(land_drop);
            }

            for card_play in strategies.card_plays(&self.state) {
                log::debug!("playing card: {card_play:?}");
                watcher.card_play(card_play, &self.state, &mut self.metrics);
                self.state.play_card(card_play);
            }

            watcher.turn_end(&self.state, &mut self.metrics);
            self.state.turn += 1;
        }

        watcher.game_end(&self.state, &mut self.metrics);

        self.metrics.trials_seen += 1;
        self.metrics
    }

}

pub fn run_trials<S, W>(num_trials: u32, deck: UnorderedPile, strategies: S, watcher: W) -> MetricsData
where S: Strategy + Clone + Sync,
      W: Watcher + Clone + Sync 
{
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    (0..num_trials)
        .into_iter()
        .into_par_iter()
        .map(|_| {
            let rng = rand::rngs::StdRng::from_entropy();
            let t = Trial::new(
                deck.clone(),
                rng,
            );
            t.run(&mut strategies.clone(), &watcher)
        })
        .reduce(|| MetricsData::empty(), MetricsData::join)
}
