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
    pub fn validate(&self, variables: &[&str]) -> Result<Vec<String>> {
        let StatFormula::Expr(src) = self else {
            return Ok(Vec::new());
        };

        let node = build_operator_tree::<DefaultNumericTypes>(src)
            .map_err(|e| DeepError::Formula(format!("{src:?}: {e}")))?;

        let mut read: Vec<String> = Vec::new();
        for ident in node.iter_variable_identifiers() {
            if variables.contains(&ident) {
                if !read.iter().any(|seen| seen == ident) {
                    read.push(ident.to_string());
                }
            } else if !identifiers().any(|known| known == ident) {
                return Err(DeepError::Formula(format!(
                    "{src:?}: unknown variable {ident:?}"
                )));
            }
        }

        Ok(read)
    }
}

impl Default for StatFormula {
    fn default() -> Self {
        StatFormula::Value(0.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Variable {
    Toggle {
        id: String,
        label: String,
        default: bool,
    },
    Slider {
        id: String,
        label: String,
        min: f64,
        max: f64,
        step: f64,
        default: f64,
    },
}

impl Variable {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Variable::Toggle { id, .. } | Variable::Slider { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StatContributions {
    pub stats: HashMap<String, StatFormula>,
    /// Multiplies its stat's total rather than adding. Caps still apply
    pub multiplicative_percents: HashMap<String, StatFormula>,
}

impl StatContributions {
    pub fn all(&self) -> impl Iterator<Item = &HashMap<String, StatFormula>> {
        [&self.stats, &self.multiplicative_percents].into_iter()
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
