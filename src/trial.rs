use crate::game::{Hand, Library, State, UnorderedPile};
use crate::strategies::Strategy;
use crate::watcher::{Watcher, MetricsData};

pub type Rand = rand::rngs::StdRng;


const MAX_TURN: u32 = 50;

pub struct Trial {
    pub rng: Rand,
    pub state: State,
    pub metrics_data: MetricsData,
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
            metrics_data: MetricsData::empty()
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
        
        log::info!("keeping hand");
        watcher.observe_opening_hand(&self.state, &mut self.metrics_data);

        while self.turn() <= MAX_TURN && !self.state.game_loss {
            let draw = self.turn() > 0 || self.state.draw_on_first_turn;
            if draw {
                self.state.draw_to_hand();
            }

            if let Some(land_drop) = strategies.land_drop(&self.state) {
                // TODO: add metrics on cards played
                self.state.play_card(land_drop);
            }

            self.state.turn += 1;
        }

        self.metrics_data.trials_seen += 1;
        self.metrics_data
    }

}
