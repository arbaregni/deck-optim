use std::fmt;
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

#[derive(Debug)]
pub struct MetricsData {
    pub(crate) trials_seen: Uint,
    metrics: HashMap<MetricsKey, Uint>,
}

impl MetricsData {
    /// Creates an empty metrics data.
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let empty = MetricsData::empty();
    /// assert_eq!(empty.get("some random thing"), 0);
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
    /// assert_eq!(metrics.get("cats seen"), 0);
    ///
    /// metrics.add("cats seen");
    /// assert_eq!(metrics.get("cats seen"), 1);
    /// ```
    pub fn add<K: Into<MetricsKey>>(&mut self, key: K) {
        self.add_count(key, 1);
    }
    
    /// Adds a count of a metrics event if `present` is true
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// assert_eq!(metrics.get("cats seen"), 0);
    /// assert_eq!(metrics.get("dogs seen"), 0);
    ///
    /// metrics.add_if("cats seen", true);
    /// assert_eq!(metrics.get("cats seen"), 1);
    /// assert_eq!(metrics.get("dogs seen"), 0);
    ///
    /// metrics.add_if("dogs seen", false);
    /// assert_eq!(metrics.get("cats seen"), 1);
    /// assert_eq!(metrics.get("dogs seen"), 0);
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
    /// assert_eq!(metrics.get("cats seen"), 0);
    ///
    /// metrics.add_count("cats seen", 120);
    /// assert_eq!(metrics.get("cats seen"), 120);
    /// ```
    pub fn add_count<K: Into<MetricsKey>>(&mut self, key: K, count: Uint) {
        *self.metrics.entry(key.into()).or_insert(0) += count;
    }

    /// Sets the specified value, if it's not already been set
    pub fn set<K: Into<MetricsKey>>(&mut self, key: K, value: Uint) {
        use std::collections::hash_map::Entry::*;
        match self.metrics.entry(key.into()) {
            Occupied(_) => {},
            Vacant(e) => { e.insert(value); }
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
    /// assert_eq!(joined.get("burgulars"), 3);
    /// assert_eq!(joined.get("cats"), 7);
    /// assert_eq!(joined.get("dogs"), 1);
    ///
    /// ```
    pub fn join(mut left: Self, right: Self) -> Self {
        left.trials_seen += right.trials_seen;
        left.metrics.reserve(right.metrics.len());
        for (k, v) in right.metrics.iter() {
            left.add_count(*k, *v);
        }
        left
    }
    
    /// Gets the total count for a specific metric
    /// ```
    /// use deck_optim::metrics::MetricsData;
    ///
    /// let mut metrics = MetricsData::empty();
    /// metrics.add_count("cats", 2);
    /// metrics.add_count("cats", 5);
    /// assert_eq!(metrics.get("cats"), 7);
    /// ```
    pub fn get<K: Into<MetricsKey>>(&self, key: K) -> Uint {
        self.metrics.get(&key.into()).copied().unwrap_or(0)
    }

    pub fn keys(&self) -> impl Iterator<Item = MetricsKey> + '_ {
        self.metrics.keys().copied()
    }

    pub fn num_trials(&self) -> u32 {
        self.trials_seen 
    }

    /// Averages the metrics over the number of trials this metrics data represents
    pub fn average<K: Into<MetricsKey>>(&self, key: K) -> f32 {
        if self.trials_seen == 0 {
            return f32::NAN;
        }
        let sum = self.get(key) as f32;
        let total = self.trials_seen as f32;
        sum / total
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

}
