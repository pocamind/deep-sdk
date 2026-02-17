use core::fmt;
use std::{collections::{BTreeSet, HashSet}, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{Stat, error, util::statmap::StatMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom {
    pub reducability: Reducability,
    pub value: i64,
    /// Stats to sum up to meet value (mostly will be a singular stat)
    pub stats: StatSet,
}

impl Atom {
    pub fn new(r: Reducability) -> Self {
        Self {
            reducability: r,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    pub fn strict() -> Self {
        Self {
            reducability: Reducability::Strict,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    pub fn reducible() -> Self {
        Self {
            reducability: Reducability::Reducible,
            value: 0,
            stats: BTreeSet::new(),
        }
    }

    pub fn value(mut self, v: i64) -> Self {
        self.value = v;
        self
    }

    pub fn reducability(mut self, r: Reducability) -> Self {
        self.reducability = r;
        self
    }

    /// Adds a stat to the stat summation requirement.
    pub fn stat(mut self, stat: Stat) -> Self {
        self.stats.insert(stat);
        self
    }

    pub fn add_stat(&mut self, stat: Stat) {
        self.stats.insert(stat);
    }

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

    // is it trivially satisfied
    pub fn is_empty(&self) -> bool {
        self.stats.is_empty() && self.value == 0
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.stats.len() == 1 {
            write!(
                f, "{}{} {}", 
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ClauseType {
    And,
    Or,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Clause {
    pub clause_type: ClauseType,
    pub atoms: BTreeSet<Atom>,
}

impl Clause {
    pub fn new(clause_type: ClauseType) -> Self {
        Self {
            clause_type,
            atoms: BTreeSet::new(),
        }
    }

    pub fn and() -> Self {
        Self {
            clause_type: ClauseType::And,
            atoms: BTreeSet::new(),
        }
    }

    pub fn or() -> Self {
        Self {
            clause_type: ClauseType::Or,
            atoms: BTreeSet::new(),
        }
    }

    pub fn clause_type(mut self, ct: ClauseType) -> Self {
        self.clause_type = ct;
        self
    }

    pub fn atoms(&self) -> &BTreeSet<Atom> {
        &self.atoms
    }

    pub fn atoms_mut(&mut self) -> &mut BTreeSet<Atom> {
        &mut self.atoms
    }

    pub fn insert(mut self, stats: StatSet, mut atom: Atom) -> Self {
        atom.stats = stats;
        self.atoms.insert(atom);
        self
    }

    pub fn atom(mut self, atom: Atom) -> Self {
        self.atoms.insert(atom);
        self
    }

    pub fn add_atom(&mut self, atom: Atom) {
        self.atoms.insert(atom);
    }

    pub fn satisfied_by(&self, stats: &StatMap) -> bool {
        match self.clause_type {
            ClauseType::And => self.atoms.iter().all(|atom| atom.satisfied_by(stats)),
            ClauseType::Or => self.atoms.iter().any(|atom| atom.satisfied_by(stats)),
        }
    }

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
            .map(|atom| format!("{}", atom))
            .collect();

        write!(f, "{}", atom_strs.join(joiner))
    }
}

#[derive(Clone, Debug, Hash)]
pub struct Requirement {
    // optional name for the req for referencing elsewhere
    pub name: Option<String>,
    // DIRECT prerequisites (does not include transitive)
    pub prereqs: Vec<String>,

    pub clauses: Vec<Clause>,
}

impl PartialEq for Requirement {
    fn eq(&self, other: &Self) -> bool {
        if self.clauses.len() != other.clauses.len() {
            return false;
        }

        // a \subseteq b and b \subseteq a iff a = b ahh comparison
        self.clauses.iter().all(|c| other.clauses.contains(c))
            && other.clauses.iter().all(|c| self.clauses.contains(c))
            && self.name_or_default() == other.name_or_default() 
            // also check if string names are the same (yes names will matter now)
    }
}

impl Eq for Requirement {}

impl Requirement {
    pub fn parse(input: &str) -> error::Result<Self> {
        crate::parse::req::parse_req(input)
    }

    pub fn new() -> Self {
        Self {
            name: None,
            prereqs: Vec::new(),
            clauses: Vec::new(),
        }
    }

    pub fn add_clause(&mut self, clause: Clause) -> &mut Self {
        self.clauses.push(clause);
        self
    }

    pub fn add_prereq(&mut self, prereq: &str) -> &mut Self {
        self.prereqs.push(prereq.to_string());
        self
    }

    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

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
        // construct new atoms
        for clause in &mut self.clauses {
            clause.atoms = clause
                .atoms
                .iter()
                .map(|atom| {
                    let mut new_atom = atom.clone();
                    new_atom.value += val;
                    new_atom.value = new_atom.value.clamp(0, 100);
                    new_atom
                })
                .collect();
        }
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

    /// Grab all the stats present in a requirement
    pub fn used_stats(&self) -> HashSet<Stat> {
        self.atoms().fold(HashSet::new(), |mut acc, atom| {
            for stat in &atom.stats {
                if stat == &Stat::Total {
                    continue;
                }

                acc.insert(stat.clone());
            }
            acc
        })
    }

    pub fn satisfied_by(&self, stats: &StatMap) -> bool {
        self.clauses.iter().all(|clause| clause.satisfied_by(stats))
    }

    /// The requirement requires nothing and is therefore trivially satisfied (wow!)
    pub fn is_empty(&self) -> bool {
        !self.clauses.iter().any(|c| !c.is_empty())
    }
}

impl From<Clause> for Requirement {
    fn from(clause: Clause) -> Self {
        Self {
            name: None,
            prereqs: Vec::new(),
            clauses: vec![clause],
        }
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.prereqs.is_empty() {
            write!(f, "{} => ", self.prereqs.join(", "))?;
        }
        if let Some(name) = &self.name {
            write!(f, "{} := ", name)?;
        }
        if self.is_empty() {
            write!(f, "()")
        } else {
            let clause_strs: Vec<String> = self
                .clauses
                .iter()
                .filter(|clause| !clause.is_empty())
                .map(|clause| clause.to_string())
                .collect();

            write!(f, "{}", clause_strs.join(", "))
        }
    }
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::req::parse_req(s).map_err(|e| format!("Failed to parse requirement: {}", e))
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
