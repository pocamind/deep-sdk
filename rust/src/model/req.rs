use core::fmt;
use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{Stat, error, util::statmap::StatMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reducability {
    Reducible,
    Strict,
}

impl fmt::Display for Reducability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Reducability::Reducible => write!(f, "r"),
            Reducability::Strict => write!(f, "s"),
        }
    }
}

pub type StatSet = BTreeSet<Stat>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Atom {
    pub reducability: Reducability,
    pub value: i64,
    /// Stats to sum up to meet value (mostly will be a singular stat)
    pub stats: StatSet,
}

impl Atom {
    #[must_use]
    pub fn new(r: Reducability) -> Self {
        Self {
            reducability: r,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn strict() -> Self {
        Self {
            reducability: Reducability::Strict,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn reducible() -> Self {
        Self {
            reducability: Reducability::Reducible,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn value(mut self, v: i64) -> Self {
        self.value = v;
        self
    }

    #[must_use]
    pub fn reducability(mut self, r: Reducability) -> Self {
        self.reducability = r;
        self
    }

    #[must_use]
    /// Adds a stat to the stat summation requirement.
    pub fn stat(mut self, stat: Stat) -> Self {
        self.stats.insert(stat);
        self
    }

    pub fn add_stat(&mut self, stat: Stat) {
        self.stats.insert(stat);
    }

    #[must_use]
    pub fn satisfied_by(&self, stats: &StatMap) -> bool {
        let sum: i64 = self
            .stats
            .iter()
            .map(|s| {
                if s == &Stat::Total {
                    stats.cost()
                } else {
                    stats.get(s)
                }
            })
            .sum();

        sum >= self.value
    }

    #[must_use]
    // is it trivially satisfied
    pub fn is_empty(&self) -> bool {
        self.stats.is_empty() && self.value == 0
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.stats.len() == 1 {
            write!(
                f,
                "{}{} {}",
                self.value,
                self.reducability,
                self.stats.first().unwrap().short_name()
            )
        } else {
            // multi-stat (display as expr)
            let sum_expr = self
                .stats
                .iter()
                .map(|s| s.short_name().to_string())
                .collect::<Vec<String>>()
                .join(" + ");

            write!(f, "{} = {}{}", sum_expr, self.value, self.reducability)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClauseType {
    And,
    Or,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Clause {
    pub clause_type: ClauseType,
    pub atoms: BTreeSet<Atom>,
}

impl Clause {
    #[must_use]
    pub fn new(clause_type: ClauseType) -> Self {
        Self {
            clause_type,
            atoms: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn and() -> Self {
        Self {
            clause_type: ClauseType::And,
            atoms: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn or() -> Self {
        Self {
            clause_type: ClauseType::Or,
            atoms: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn clause_type(mut self, ct: ClauseType) -> Self {
        self.clause_type = ct;
        self
    }

    #[must_use]
    pub fn atoms(&self) -> &BTreeSet<Atom> {
        &self.atoms
    }

    pub fn atoms_mut(&mut self) -> &mut BTreeSet<Atom> {
        &mut self.atoms
    }

    #[must_use]
    pub fn insert(mut self, stats: StatSet, mut atom: Atom) -> Self {
        atom.stats = stats;
        self.atoms.insert(atom);
        self
    }

    #[must_use]
    pub fn atom(mut self, atom: Atom) -> Self {
        self.atoms.insert(atom);
        self
    }

    pub fn add_atom(&mut self, atom: Atom) {
        self.atoms.insert(atom);
    }

    #[must_use]
    pub fn satisfied_by(&self, stats: &StatMap) -> bool {
        match self.clause_type {
            ClauseType::And => self.atoms.iter().all(|atom| atom.satisfied_by(stats)),
            ClauseType::Or => self.atoms.iter().any(|atom| atom.satisfied_by(stats)),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.atoms().iter().any(|a| !a.is_empty())
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let joiner = match self.clause_type {
            ClauseType::And => ", ",
            ClauseType::Or => " OR ",
        };

        let atom_strs: Vec<String> = self
            .atoms
            .iter()
            .filter(|a| !a.is_empty())
            .map(|atom| format!("{atom}"))
            .collect();

        write!(f, "{}", atom_strs.join(joiner))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrereqGroup {
    pub alternatives: BTreeSet<String>,
}

impl PrereqGroup {
    #[must_use]
    pub fn single(name: &str) -> Self {
        Self {
            alternatives: BTreeSet::from([name.to_string()]),
        }
    }

    #[must_use]
    pub fn any<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            alternatives: names.into_iter().map(Into::into).collect(),
        }
    }

    #[must_use]
    pub fn is_single(&self) -> bool {
        self.alternatives.len() == 1
    }

    pub fn alternatives(&self) -> impl Iterator<Item = &String> {
        self.alternatives.iter()
    }

    pub fn parse(input: &str) -> error::Result<Self> {
        crate::parse::req::parse_prereq_group(input)
    }
}

impl fmt::Display for PrereqGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.alternatives
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(" | ")
        )
    }
}

impl FromStr for PrereqGroup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::req::parse_prereq_group(s).map_err(|e| format!("Failed to parse prereq: {e}"))
    }
}

impl<'de> Deserialize<'de> for PrereqGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl Serialize for PrereqGroup {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Requirement {
    // optional name for the req for referencing elsewhere
    pub name: Option<String>,
    // DIRECT prerequisites (does not include transitive)
    pub prereqs: BTreeSet<PrereqGroup>,

    pub clauses: BTreeSet<Clause>,
}

impl Requirement {
    pub fn parse(input: &str) -> error::Result<Self> {
        crate::parse::req::parse_req(input)
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            prereqs: BTreeSet::new(),
            clauses: BTreeSet::new(),
        }
    }

    pub fn add_clause(&mut self, clause: Clause) -> &mut Self {
        self.clauses.insert(clause);
        self
    }

    pub fn add_prereq(&mut self, prereq: &str) -> &mut Self {
        self.prereqs.insert(PrereqGroup::single(prereq));
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    #[must_use]
    pub fn name_or_default(&self) -> String {
        match &self.name {
            Some(n) => n.clone(),
            None => self.to_string(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Clause> {
        self.clauses.iter()
    }

    pub fn and_iter(&self) -> impl Iterator<Item = &Clause> {
        self.clauses
            .iter()
            .filter(|c| c.clause_type == ClauseType::And)
    }

    pub fn or_iter(&self) -> impl Iterator<Item = &Clause> {
        self.clauses
            .iter()
            .filter(|c| c.clause_type == ClauseType::Or)
    }

    pub fn atoms(&self) -> impl Iterator<Item = &Atom> {
        self.clauses.iter().flat_map(|clause| clause.atoms.iter())
    }

    pub fn add_to_all(&mut self, val: i64) -> &mut Self {
        self.add_to_atoms(val, |_| true)
    }

    /// Adds `val` to every atom that does not gate on [`Stat::Total`], leaving power level
    /// gates untouched.
    pub fn add_to_stat_atoms(&mut self, val: i64) -> &mut Self {
        self.add_to_atoms(val, |atom| !atom.stats.contains(&Stat::Total))
    }

    fn add_to_atoms(&mut self, val: i64, predicate: impl Fn(&Atom) -> bool) -> &mut Self {
        let mut new_clauses: BTreeSet<Clause> = BTreeSet::new();
        // construct new atoms
        for clause in self.clauses.iter().cloned() {
            new_clauses.insert(Clause {
                clause_type: clause.clause_type,
                atoms: clause
                    .atoms
                    .iter()
                    .map(|atom| {
                        if !predicate(atom) {
                            return atom.clone();
                        }

                        let mut new_atom = atom.clone();
                        new_atom.value += val;
                        new_atom.value = new_atom.value.clamp(0, 100);
                        new_atom
                    })
                    .collect(),
            });
        }
        self.clauses = new_clauses;
        self
    }

    pub fn strict_atoms(&self) -> impl Iterator<Item = &Atom> {
        self.clauses.iter().flat_map(|clause| {
            clause
                .atoms
                .iter()
                .filter(|atom| atom.reducability == Reducability::Strict)
        })
    }

    #[must_use]
    /// Grab all the stats present in a requirement
    pub fn used_stats(&self) -> HashSet<Stat> {
        self.atoms().fold(HashSet::new(), |mut acc, atom| {
            for stat in &atom.stats {
                if stat == &Stat::Total {
                    continue;
                }

                acc.insert(*stat);
            }
            acc
        })
    }

    #[must_use]
    pub fn satisfied_by(&self, stats: &StatMap) -> bool {
        self.clauses.iter().all(|clause| clause.satisfied_by(stats))
    }

    #[must_use]
    /// The requirement requires nothing and is therefore trivially satisfied (wow!)
    pub fn is_empty(&self) -> bool {
        !self.clauses.iter().any(|c| !c.is_empty())
    }
}

impl Default for Requirement {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Clause> for Requirement {
    fn from(clause: Clause) -> Self {
        Self {
            name: None,
            prereqs: BTreeSet::new(),
            clauses: BTreeSet::from_iter([clause]),
        }
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.prereqs.is_empty() {
            write!(
                f,
                "{} => ",
                self.prereqs
                    .iter()
                    .enumerate()
                    .fold(String::new(), |mut acc, (i, group)| {
                        if i > 0 {
                            acc.push_str(", ");
                        }
                        acc.push_str(&group.to_string());
                        acc
                    })
            )?;
        }
        if let Some(name) = &self.name {
            write!(f, "{name} := ")?;
        }
        if self.is_empty() {
            write!(f, "()")
        } else {
            let clause_strs: Vec<String> = self
                .clauses
                .iter()
                .filter(|clause| !clause.is_empty())
                .map(ToString::to_string)
                .collect();

            write!(f, "{}", clause_strs.join(", "))
        }
    }
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::req::parse_req(s).map_err(|e| format!("Failed to parse requirement: {e}"))
    }
}

impl<'de> Deserialize<'de> for Requirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl Serialize for Requirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Timing {
    Free,
    Post,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn khan_lowers_stat_reqs_but_not_power_gates() {
        // crypt blade is equippable at 72 SDW / 37 HVY as a Khan
        let mut req: Requirement = "crypt_blade := 40r HVY, 75r SDW".parse().unwrap();
        req.add_to_stat_atoms(-3);
        assert_eq!(req.to_string(), "crypt_blade := 37r HVY, 72r SDW");

        // TTL models a power level, which Versatile does not lower
        let mut req: Requirement = "abyss_wanderers_boots := 165r TTL".parse().unwrap();
        req.add_to_stat_atoms(-3);
        assert_eq!(req.to_string(), "abyss_wanderers_boots := 165r TTL");

        // a mixed req keeps its power gate and lowers only the stat
        let mut req: Requirement = "11th_legion_plate := 90r TTL, 10r FTD".parse().unwrap();
        req.add_to_stat_atoms(-3);
        assert_eq!(req.to_string(), "11th_legion_plate := 7r FTD, 90r TTL");

        // an OR clause lowers each of its atoms
        let mut req: Requirement = "kindred_edict := 50r MED, 30r STR OR 30r FTD".parse().unwrap();
        req.add_to_stat_atoms(-3);
        assert_eq!(req.to_string(), "kindred_edict := 47r MED, 27r STR OR 27r FTD");
    }

    #[test]
    fn khan_clamps_at_zero() {
        let mut req: Requirement = "thing := 2r STR".parse().unwrap();
        req.add_to_stat_atoms(-3);
        assert_eq!(req.to_string(), "thing := 0r STR");
    }
}
