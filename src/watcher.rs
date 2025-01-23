use std::collections::HashMap;

use itertools::Itertools;

use crate::game::state::State;

pub trait Watcher {
    #[allow(unused)]
    fn observe_opening_hand(&self, state: &State, metrics: &mut MetricsData) { }
}

pub type MetricsKey = &'static str;
pub type Uint = u32;

#[derive(Debug)]
pub struct MetricsData {
    pub(crate) trials_seen: Uint,
    metrics: HashMap<MetricsKey, Uint>,
}

impl MetricsData {
    pub fn empty() -> Self {
        Self {
            trials_seen: 0,
            metrics: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: MetricsKey, was_present: bool) {
        let count = match was_present {
            true => 1,
            false => 0,
        };
        self.add_count(key, count);
    }
    pub fn add_count(&mut self, key: MetricsKey, count: Uint) {
        *self.metrics.entry(key).or_insert(0) += count;
    }

    pub fn join(mut left: Self, right: Self) -> Self {
        left.trials_seen += right.trials_seen;
        left.metrics.reserve(right.metrics.len());
        for (k, v) in right.metrics.iter() {
            left.add_count(*k, *v);
        }
        left
    }
    
    fn _get(&self, key: MetricsKey) -> Uint {
        self.metrics.get(&key).copied().unwrap_or(0)
    }


    pub fn keys(&self) -> impl Iterator<Item = MetricsKey> + '_ {
        self.metrics.keys().sorted().copied()
    }

    pub fn average(&self, key: MetricsKey) -> f32 {
        if self.trials_seen == 0 {
            return f32::NAN;
        }
        let sum = self._get(key) as f32;
        let total = self.trials_seen as f32;
        sum / total
    }
}

pub struct WatcherImpl;
impl Watcher for WatcherImpl {
    fn observe_opening_hand<'a>(&self, state: &State, metrics: &mut MetricsData) { 
        metrics.add_count("opening_hand::lands", state.num_lands_in_hand() as u32);
    }

}

