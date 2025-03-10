use std::{default, fmt};
use std::collections::HashMap;

use crate::collection::Card;

pub type Uint = u32;

#[derive(Debug,Copy,Clone,Eq,PartialEq,Hash,PartialOrd,Ord)]
pub struct MetricsKey {
    metrics_name: &'static str,
    card: Option<Card>,
    turn_num: Option<u32>
}
impl MetricsKey {
    pub fn new(metrics_name: &'static str) -> Self {
        Self {
            metrics_name,
            card: None,
            turn_num: None,
        }
    }
    pub fn card(mut self, card: Card) -> Self {
        self.card = Some(card);
        self
    }
    pub fn turn_num(mut self, turn_num: u32) -> Self {
        self.turn_num = Some(turn_num);
        self
    }
}

impl From<&'static str> for MetricsKey {
    fn from(name: &'static str) -> Self {
        MetricsKey::new(name)
    }
}

impl fmt::Display for MetricsKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.metrics_name)?;
        if let Some(x) = self.card {
            write!(f, "::{}", x.name())?;
        }
        if let Some(x) = self.turn_num {
            write!(f, "::{x}")?;
        }

        Ok(())
    }
}

/// The metrics that are being tracked for a particular key.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    total: Uint,
    min: Uint,
    max: Uint,
    trials_seen: Uint,
    count: Uint
}
impl default::Default for Metrics {
    fn default() -> Self {
        Self {
            total: 0,
            min: 0,
            max: 0,
            trials_seen: 1,
            count: 0
        }
    }
}
impl Metrics {
    pub fn update_add(&mut self, value: Uint) {
        self.total += value;
        self.min += value;
        self.max += value;
        self.count += 1;
    }
    pub fn update_set(&mut self, value: Uint) {
        self.total = value;
        self.min = value;
        self.max = value;
        self.count += 1;
    }

    /// Merge two metrics
    pub fn merge_in(&mut self, other: Metrics) {
        // trick to ensure all fields are accounted for
        #[deny(unused_variables)]
        let Metrics { total, min, max, trials_seen, count } = other;

        self.total += total;
        self.min = std::cmp::min(self.min, min);
        self.max = std::cmp::max(self.max, max);
        self.trials_seen += trials_seen;
        self.count += count;
    }

    /// Averages this metrics key accross all of the trials it has seen
    pub fn average(&self) -> f32 {
        self.total as f32 / self.trials_seen as f32
    }
}

#[derive(Debug)]
/// Keep track of the metrics data for all keys.
pub struct MetricsData {
    pub(crate) trials_seen: Uint,
    metrics: HashMap<MetricsKey, Metrics>,
}

impl MetricsData {
    /// Creates an empty metrics data.
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let empty = MetricsData::empty();
    /// assert_eq!(empty.total("some random thing"), 0);
    /// ```
    pub fn empty() -> Self {
        Self {
            trials_seen: 0,
            metrics: HashMap::new(),
        }
    }

    /// Adds a single count of a metrics event
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// assert_eq!(metrics.total("cats seen"), 0);
    ///
    /// metrics.add("cats seen");
    /// assert_eq!(metrics.total("cats seen"), 1);
    /// ```
    pub fn add<K: Into<MetricsKey>>(&mut self, key: K) {
        self.add_count(key, 1);
    }
    
    /// Adds a count of a metrics event if `present` is true
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// assert_eq!(metrics.total("cats seen"), 0);
    /// assert_eq!(metrics.total("dogs seen"), 0);
    ///
    /// metrics.add_if("cats seen", true);
    /// assert_eq!(metrics.total("cats seen"), 1);
    /// assert_eq!(metrics.total("dogs seen"), 0);
    ///
    /// metrics.add_if("dogs seen", false);
    /// assert_eq!(metrics.total("cats seen"), 1);
    /// assert_eq!(metrics.total("dogs seen"), 0);
    /// ```
    pub fn add_if<K: Into<MetricsKey>>(&mut self, key: K, present: bool) {
        let count = match present {
            true => 1,
            false => 0,
        };
        self.add_count(key, count);
    }

    /// Adds the specified count to the metrics
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// assert_eq!(metrics.total("cats seen"), 0);
    ///
    /// metrics.add_count("cats seen", 120);
    /// assert_eq!(metrics.total("cats seen"), 120);
    /// ```
    pub fn add_count<K: Into<MetricsKey>>(&mut self, key: K, count: Uint) {
        self.metrics.entry(key.into())
            .or_default()
            .update_add(count);
    }

    /// Sets the specified value, if it's not already been set
    pub fn set<K: Into<MetricsKey>>(&mut self, key: K, value: Uint) {
        use std::collections::hash_map::Entry::*;
        match self.metrics.entry(key.into()) {
            Occupied(_) => {},
            Vacant(e) => { 
                let mut entry = Metrics::default();
                entry.update_set(value);
                e.insert(entry);
            }
        }
    }

    /// Joins two metrics together
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut left = MetricsData::empty();
    /// left.add_count("burgulars", 3);
    /// left.add_count("cats", 2);
    ///
    /// let mut right = MetricsData::empty();
    /// right.add_count("cats", 5);
    /// right.add_count("dogs", 1);
    ///
    /// let joined = MetricsData::join(left, right);
    /// assert_eq!(joined.total("burgulars"), 3);
    /// assert_eq!(joined.total("cats"), 7);
    /// assert_eq!(joined.total("dogs"), 1);
    ///
    /// ```
    pub fn join(mut left: Self, right: Self) -> Self {
        use std::collections::hash_map::Entry::Occupied;
        use std::collections::hash_map::Entry::Vacant;

        left.trials_seen += right.trials_seen;
        left.metrics.reserve(right.metrics.len());
        for (key, metrics) in right.metrics.into_iter() {

            // We do not want to merge in empty metrics, that messes up `min` as it defaults to zero
            // (this could also be fixed by using an Optional for min and max)
            match left.metrics.entry(key) {
                Occupied(mut left) => {
                    left.get_mut().merge_in(metrics);
                }
                Vacant(left) => {
                    left.insert(metrics);
                }
            }
        }
        left
    }
       pub fn keys(&self) -> impl Iterator<Item = MetricsKey> + '_ {
        self.metrics.keys().copied()
    }

    pub fn num_trials(&self) -> u32 {
        self.trials_seen 
    }

    pub fn get<K: Into<MetricsKey>>(&self, key: K) -> Metrics {
        self.metrics.get(&key.into())
            .copied()
            .unwrap_or_default()
    }
 
    /// Gets the total count for a specific metric
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// metrics.add_count("cats", 2);
    /// metrics.add_count("cats", 5);
    /// assert_eq!(metrics.total("cats"), 7);
    /// ```
    pub fn total<K: Into<MetricsKey>>(&self, key: K) -> Uint {
        self.get(key)
            .total
    }

    /// Averages the metrics over the number of trials this metrics data represents
    pub fn average<K: Into<MetricsKey>>(&self, key: K) -> f32 {
        self.get(key)
            .average()
    }

    /// Returns the minimum ever seen for this metric.
    pub fn min<K: Into<MetricsKey>>(&self, key: K) -> Uint {
        self.get(key)
            .min
    }
    /// Returns the maximum ever seen for this metric.
    pub fn max<K: Into<MetricsKey>>(&self, key: K) -> Uint {
        self.get(key)
            .max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_sums_number_of_trials() {
        let mut left = MetricsData::empty();
        left.trials_seen = 77;

        let mut right = MetricsData::empty();
        right.trials_seen = 13;

        let both = MetricsData::join(left, right);
        assert_eq!(both.num_trials(), 90); // 77 + 13 = 90
    }

    #[test]
    fn joins_respect_different_trials_seen() {
        let mut left = MetricsData::empty();
        left.add_count("cats", 3);
        left.add_count("dogs", 5);
        left.add_count("snakes", 14);
        
        let mut right = MetricsData::empty();
        right.add_count("cats", 1);
        right.add_count("snakes", 0);

        let both = MetricsData::join(left, right);

        assert_eq!(both.total("cats"), 4);
        assert_eq!(both.total("dogs"), 5);
        assert_eq!(both.total("snakes"), 14);

        assert_eq!(both.average("cats"), 2.0);
        assert_eq!(both.average("dogs"), 5.0); // it's 5 becasue we exclude the trial where we didn't see any dogs
        assert_eq!(both.average("snakes"), 7.0); // this one gets averaged out

        assert_eq!(both.min("cats"), 1);
        assert_eq!(both.min("dogs"), 5);
        assert_eq!(both.min("snakes"), 0);
    }

}
