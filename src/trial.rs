use rand::SeedableRng;

use crate::game::annotations::AnnotationValue;
use crate::game::Deck;
use crate::game::Library;
use crate::game::Hand;
use crate::game::state::State;
use crate::strategies::Strategy;
use crate::watcher::Watcher;
use crate::metrics::MetricsData;

pub type Rand = rand::rngs::StdRng;

#[derive(Debug,Clone,Copy)]
pub struct Props {
    pub max_turn: u32,
    pub num_trials: u32,
}
impl Default for Props {
    fn default() -> Self {
        Self {
            max_turn: 12,
            num_trials: 1000,
        }
    }
}

/// Work needed for a particular run
pub struct Trial {
    pub rng: Rand,
    pub state: State,
    pub metrics: MetricsData,
    pub props: Props
}

impl Trial {
    pub fn new(deck: Deck, rng: Rand) -> Self {
        Self::from_props(deck, rng, Props::default())
    }
    pub fn from_props(deck: Deck, mut rng: Rand, props: Props) -> Self {
        let state = State::new(
            deck,
            &mut rng
        );
        Trial {
            rng,
            state,
            metrics: MetricsData::empty(),
            props,
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

        self.state.turn = 1;
        while self.state.turn() <= self.props.max_turn && !self.state.game_loss {
            let draw = self.turn() > 1 || self.state.draw_on_first_turn;
            if draw {
                self.state.draw_to_hand();
            }

            log::debug!("on turn {}, {} cards in hand, {} cards in play, {} cards in graveyard, {} cards in deck",
                        self.state.turn,
                        self.state.hand.size(),
                        self.state.permanents.size(),
                        self.state.graveyard.size(),
                        self.state.library.size()
            );


            for card_play in strategies.card_plays(&self.state) {
                log::debug!("playing card: {card_play:?}");
                watcher.card_play(card_play.card, &self.state, &mut self.metrics);

                card_play.card
                    .effects()
                    .iter()
                    .for_each(|effect| self.apply_card_effect(effect));

                self.state.play_card(card_play);
            }

            watcher.turn_end(&self.state, &mut self.metrics);

            self.state.end_turn();
        }

        watcher.game_end(&self.state, &mut self.metrics);

        self.metrics.trials_seen += 1;
        self.metrics
    }

    fn apply_card_effect(&mut self, effect: &AnnotationValue) {
        use AnnotationValue::*;
        match effect {
            String(s) if s == "fetches" => {
                todo!("apply game action: {s}")
            }
            _ => { /* nothing to do */}
        }



    }

}

pub fn run_trials<S, W>(deck: Deck, strategies: S, watcher: W, props: Props) -> MetricsData
where S: Strategy + Clone + Sync,
      W: Watcher + Clone + Sync 
{
    use rayon::iter::IntoParallelIterator;
    use rayon::iter::ParallelIterator;

    log::info!("beginning trial with props: {props:?}");

    (0..props.num_trials)
        .into_iter()
        .into_par_iter()
        .map(|_| {
            let rng = rand::rngs::StdRng::from_entropy();
            let t = Trial::from_props(
                deck.clone(),
                rng,
                props
            );
            t.run(&mut strategies.clone(), &watcher)
        })
        .reduce(|| MetricsData::empty(), MetricsData::join)
}
