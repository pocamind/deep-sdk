use evalexpr::{DefaultNumericTypes, build_operator_tree};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{DeepError, Result};
use crate::model::stat::{ATTUNEMENT, CORE, WEAPON};

/// A stat contribution that is either a constant or an expression over the build's
/// invested attributes.
///
/// See docs/stat_expressions.md
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StatFormula {
    Value(f64),
    Expr(String),
}

impl StatFormula {
    /// Parses the expression and checks every identifier resolves
    pub fn validate(&self) -> Result<()> {
        let StatFormula::Expr(src) = self else {
            return Ok(());
        };

        let node = build_operator_tree::<DefaultNumericTypes>(src)
            .map_err(|e| DeepError::Formula(format!("{src:?}: {e}")))?;

        for ident in node.iter_variable_identifiers() {
            if !identifiers().any(|known| known == ident) {
                return Err(DeepError::Formula(format!("{src:?}: unknown stat {ident:?}")));
            }
        }

        Ok(())
    }
}

impl Default for StatFormula {
    fn default() -> Self {
        StatFormula::Value(0.0)
    }
}

/// The four ways any source can contribute to a build's stats
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StatContributions {
    /// Always applies
    pub stats: HashMap<String, StatFormula>,
    /// Only applies on some condition we don't know. Stored at its maximum
    /// and only counted under 'optimistic' circumstances 
    pub conditional_stats: HashMap<String, StatFormula>,
    /// Multiplies its stat's total rather than adding. Caps still apply
    pub multiplicative_percents: HashMap<String, StatFormula>,
    pub conditional_multiplicative_percents: HashMap<String, StatFormula>,
}

impl StatContributions {
    /// The additive maps that apply in this mode
    pub fn additive(&self, optimistic: bool) -> impl Iterator<Item = &HashMap<String, StatFormula>> {
        std::iter::once(&self.stats).chain(optimistic.then_some(&self.conditional_stats))
    }

    /// The multiplicative maps that apply in this mode
    pub fn multiplicative(
        &self,
        optimistic: bool,
    ) -> impl Iterator<Item = &HashMap<String, StatFormula>> {
        std::iter::once(&self.multiplicative_percents)
            .chain(optimistic.then_some(&self.conditional_multiplicative_percents))
    }

    /// Every map regardless of mode
    pub fn all(&self) -> impl Iterator<Item = &HashMap<String, StatFormula>> {
        [
            &self.stats,
            &self.conditional_stats,
            &self.multiplicative_percents,
            &self.conditional_multiplicative_percents,
        ]
        .into_iter()
    }
}

impl From<f64> for StatFormula {
    fn from(value: f64) -> Self {
        StatFormula::Value(value)
    }
}

fn identifiers() -> impl Iterator<Item = &'static str> {
    CORE.iter()
        .chain(WEAPON)
        .chain(ATTUNEMENT)
        .map(|stat| stat.short_name())
        .chain(["TTL", "PWR", "PVP", "PVE"])
}
