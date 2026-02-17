use std::{collections::HashMap, ops::{Deref, DerefMut}};

use serde::{Serialize, Deserialize};

use crate::Stat;

/// Wrapper around a HashMap of stats to their values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatMap(pub HashMap<Stat, i64>);

impl StatMap {
    /// Creates a new empty Stats map.
    pub fn new() -> Self {
        StatMap(HashMap::new())
    }

    pub fn cost(&self) -> i64 {
        self.0.values().sum::<i64>()
            - (self.0.iter().filter(|(s, _)| s.is_attunement()).count() as i64 - 1).max(0)
    }

    pub fn get(&self, stat: &Stat) -> i64 {
        *self.0.get(stat).unwrap_or(&0)
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

impl Into<HashMap<Stat, i64>> for StatMap {
    fn into(self) -> HashMap<Stat, i64> {
        self.0
    }
}
