use evalexpr::{
    ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value, build_operator_tree,
};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{DeepError, Result};
use crate::formulas::CombatState;
use crate::model::stat::{ATTUNEMENT, CORE, WEAPON};
use crate::util::statmap::StatMap;

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

    /// Evaluate this expression given supplied state
    pub fn eval(&self, stats: &StatMap, state: CombatState) -> Result<f64> {
        let src = match self {
            StatFormula::Value(value) => return Ok(*value),
            StatFormula::Expr(src) => src,
        };

        let node = build_operator_tree::<DefaultNumericTypes>(src)
            .map_err(|e| DeepError::Formula(format!("{src:?}: {e}")))?;

        node.eval_with_context(&context_for(stats, state))
            .and_then(|value| value.as_number())
            .map_err(|e| DeepError::Formula(format!("{src:?}: {e}")))
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

#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
fn context_for(stats: &StatMap, state: CombatState) -> HashMapContext<DefaultNumericTypes> {
    let mut ctx = HashMapContext::<DefaultNumericTypes>::new();

    for stat in CORE.iter().chain(WEAPON).chain(ATTUNEMENT) {
        let _ = ctx.set_value(stat.short_name().to_string(), Value::Float(stats.get(stat) as f64));
    }
    let _ = ctx.set_value("TTL".to_string(), Value::Float(stats.cost() as f64));
    let _ = ctx.set_value("PWR".to_string(), Value::Float(stats.level(None) as f64));
    let _ = ctx.set_value("PVP".to_string(), Value::Boolean(state == CombatState::Pvp));
    let _ = ctx.set_value("PVE".to_string(), Value::Boolean(state == CombatState::Pve));

    ctx
}
