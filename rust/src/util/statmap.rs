use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{Stat, model::stat::MAX_TOTAL, util::algos};

/// Wrapper around a `HashMap` of stats to their values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatMap(pub HashMap<Stat, i64>);

impl StatMap {
    /// Creates a new empty Stats map.
    #[must_use]
    pub fn new() -> Self {
        StatMap(HashMap::new())
    }

    #[must_use]
    #[allow(
        clippy::cast_possible_wrap,
        reason = "we're never having too many stats in the statmap"
    )]
    pub fn cost(&self) -> i64 {
        self.0.values().sum::<i64>()
            - (self
                .0
                .iter()
                .filter(|(s, v)| s.is_attunement() && **v > 0)
                .count() as i64
                - 1)
            .max(0)
    }

    #[must_use]
    pub fn remaining(&self) -> i64 {
        MAX_TOTAL - self.cost()
    }

    #[must_use]
    pub fn level(&self) -> i64 {
        ((self.cost() - 15) / 15).max(0)
    }

    #[must_use]
    pub fn get(&self, stat: &Stat) -> i64 {
        *self.0.get(stat).unwrap_or(&0)
    }

    #[must_use]
    pub fn shrine_order(&self, racial: &StatMap) -> StatMap {
        algos::shrine_order_dwb(self, racial)
    }
}

impl Default for StatMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for StatMap {
    type Target = HashMap<Stat, i64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StatMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashMap<Stat, i64>> for StatMap {
    fn from(map: HashMap<Stat, i64>) -> Self {
        StatMap(map)
    }
}

#[allow(
    clippy::implicit_hasher,
    reason = "StatMap itself is not generic over hashers"
)]
impl From<StatMap> for HashMap<Stat, i64> {
    fn from(val: StatMap) -> Self {
        val.0
    }
}
