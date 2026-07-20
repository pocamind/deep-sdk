/* algo implementations */

use crate::{
    Stat,
    data::DeepData,
    error::{DeepError, Result},
    model::reqfile::Reqfile,
    model::stat::StatRange,
    req::{Atom, Clause, ClauseType, Reducability, Requirement},
    util::statmap::StatMap,
};

use crate::constants::KHAN_REQ_REDUCTION;
use std::{collections::{BTreeSet, HashMap, HashSet}, ops::RangeInclusive};


#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "values are not big enough for this to matter"
)]
pub fn shrine_order_dwb(pre: &StatMap, racial: &StatMap) -> StatMap {
    use crate::constants::SHRINE_ORDER_MAX_LOSS as SHRINE_DIFF_CAP;
    use crate::constants::STAT_CAP;

    let points_start = pre.cost();

    let mut work: HashMap<Stat, f64> = pre
        .iter()
        .map(|(stat, value)| (*stat, *value as f64))
        .collect();

    let mut total = 0.0_f64;
    let mut divide_by: i64 = 0;
    let mut affected_stats: Vec<Stat> = Vec::new();

    for (stat, value) in pre.iter() {
        if *value <= 0 {
            continue;
        }

        let racial_val = racial.get(stat);

        if racial_val > 0 && *value - racial_val <= 0 {
            continue;
        }

        total += (*value - racial_val.max(0)) as f64;
        affected_stats.push(*stat);
        divide_by += 1;
    }

    if divide_by == 0 {
        return pre.clone();
    }

    let average = total / divide_by as f64;
    for stat in &affected_stats {
        work.insert(*stat, average);
    }

    let mut bottlenecked_divide_by = divide_by;
    let mut bottlenecked: HashSet<Stat> = HashSet::new();
    let mut prev = work.clone();

    loop {
        let mut bottlenecked_points = 0.0_f64;
        let mut bottlenecked_stats = false;

        for stat in &affected_stats {
            if stat.is_attunement() {
                continue;
            }

            let prev_val = *prev.get(stat).unwrap_or(&0.0);
            let shrine_val = pre.get(stat) as f64;
            let current = *work.get(stat).unwrap_or(&0.0);

            if shrine_val - current > SHRINE_DIFF_CAP {
                let new_val = shrine_val - SHRINE_DIFF_CAP;
                work.insert(*stat, new_val);
                bottlenecked_points += new_val - prev_val;

                if bottlenecked.insert(*stat) {
                    bottlenecked_divide_by -= 1;
                }
            }
        }

        if bottlenecked_divide_by <= 0 {
            break;
        }

        let spread = bottlenecked_points / bottlenecked_divide_by as f64;

        // Second pass: redistribute
        for stat in &affected_stats {
            if bottlenecked.contains(stat) {
                continue;
            }

            let current = *work.get(stat).unwrap_or(&0.0);
            let next = current - spread;
            work.insert(*stat, next);

            if !stat.is_attunement() {
                let shrine_val = pre.get(stat) as f64;
                if shrine_val - next > SHRINE_DIFF_CAP {
                    bottlenecked_stats = true;
                }
            }
        }

        prev.clone_from(&work);

        if !bottlenecked_stats {
            break;
        }
    }

    let mut result = pre.clone();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "value is floored before converting to i64"
    )]
    for (stat, value) in work {
        result.insert(stat, value.floor() as i64);
    }

    let mut spare_points = points_start - result.cost();

    while bottlenecked_divide_by > 0 && spare_points >= bottlenecked_divide_by {
        let mut changed = false;

        for stat in &affected_stats {
            if bottlenecked.contains(stat) {
                continue;
            }

            if result.get(stat) >= STAT_CAP {
                continue;
            }

            *result.entry(*stat).or_insert(0) += 1;
            spare_points -= 1;
            changed = true;
        }

        if !changed {
            break;
        }
    }

    result
}

/// The configuration for a build that affect requirement generation.
pub struct BuildConfig {
    /// Controls whether the requirement generation will output weapon requirements as
    /// strict or reducible.
    ///
    #[allow(clippy::doc_markdown, reason = "false positive on SoM")]
    /// Default: false (allow SoM on weapon requirements)
    pub disable_som_weapons: bool,

    /// Puts weapon requirements in the Free: block instead of constraining it to Post.
    pub allow_weapons_preshrine: bool,

    pub talents: Vec<String>,
    /// A list of talent names that should be forced into a post-shrine stage. 
    /// A talent here should be present in the `talents` field too, this is just a marker for talents that should be obtained postshrine
    /// TODO! should generalize after a unified id/req framework
    pub post_talents: Vec<String>,

    pub mantras: Vec<String>,
    pub weapons: Vec<String>,
    pub equipment: Vec<String>,
    pub outfit: Option<String>,
    pub required_mantra_levels: Option<StatMap>,
    pub race: Option<String>,

    pub final_ranges: HashMap<Stat, RangeInclusive<u32>>,

    /// Use optional reqfiles (don't expose the optional req api yet)
    pub use_presets: Vec<Reqfile>,
}

impl BuildConfig {
    /// Whether the build's race lowers equipment and weapon requirements (Khan's Versatile).
    fn is_khan(&self, data: &DeepData) -> Result<bool> {
        let Some(race) = &self.race else {
            return Ok(false);
        };

        let race = data
            .get_aspect(race)
            .ok_or(DeepError::ReqfileBuild(format!("Race not found: {race}")))?;

        Ok(race.name == "Khan")
    }

    /// Generates a reqfile from the given data.
    pub fn to_reqfile(&self, data: &DeepData) -> Result<Reqfile> {
        let mut ret = Reqfile {
            general: vec![],
            post: vec![],
            final_ranges: self
                .final_ranges
                .iter()
                .map(|(stat, range)| StatRange {
                    stat: *stat,
                    range: range.clone(),
                })
                .collect(),
            optional: vec![],
            implicit: HashMap::new(),
        };

        let post_talents: HashSet<String> = self.post_talents.clone().into_iter().collect();

        for name in &self.talents {
            let talent = data
                .get_talent(name)
                .ok_or(DeepError::ReqfileBuild(format!(
                    "Talent {name} not found in database"
                )))?;

            // exclude implicit reqs in the reqfile formulation
            if talent.implicit {
                continue;
            }

            if post_talents.contains(name) {
                ret.post.push(talent.reqs.clone());
            } else {
                ret.general.push(talent.reqs.clone());
            }
        }

        for name in &self.mantras {
            let mantra = data
                .get_mantra(name)
                .ok_or(DeepError::ReqfileBuild(format!(
                    "Mantra {name} not found in database"
                )))?;

            ret.general.push(mantra.reqs.clone());
        }

        for name in &self.weapons {
            let weapon = data
                .get_weapon(name)
                .ok_or(DeepError::ReqfileBuild(format!(
                    "Weapon {name} not found in database"
                )))?;

            let mut req = if self.disable_som_weapons {
                let mut new_req_clauses: BTreeSet<Clause> = BTreeSet::new();

                for clause in &weapon.reqs.clauses {
                    new_req_clauses.insert(Clause {
                        clause_type: clause.clause_type.clone(),
                        atoms: clause
                            .atoms
                            .clone()
                            .into_iter()
                            .map(|a| a.reducability(Reducability::Strict))
                            .collect(),
                    });
                }

                Requirement {
                    name: weapon.reqs.name.clone(),
                    prereqs: weapon.reqs.prereqs.clone(),
                    clauses: new_req_clauses,
                }
            } else {
                weapon.reqs.clone()
            };

            if self.is_khan(data)? {
                req.add_to_stat_atoms(-KHAN_REQ_REDUCTION);
            }

            {
                if self.allow_weapons_preshrine {
                    &mut ret.general
                } else {
                    &mut ret.post
                }
            }
            .push(req);
        }

        if let Some(name) = &self.outfit {
            ret.general.push(
                data.get_outfit(name)
                    .ok_or(DeepError::ReqfileBuild(format!(
                        "Outfit {name} not found in database"
                    )))?
                    .reqs
                    .clone(),
            );
        }

        for name in &self.equipment {
            let equipment = data
                .get_equipment(name)
                .ok_or(DeepError::ReqfileBuild(format!(
                    "Equipment {name} not found in database"
                )))?;

            let mut req = equipment.reqs.clone();

            if self.is_khan(data)? {
                req.add_to_stat_atoms(-KHAN_REQ_REDUCTION);
            }

            ret.general.push(req);
        }

        if let Some(mantra_levels) = &self.required_mantra_levels {
            let mut clause = Clause::new(ClauseType::And);
            for (stat, lvl) in &mantra_levels.0 {
                let lvl = (*lvl).max(1);

                if lvl == 1 {
                    clause.add_atom(Atom::reducible().stat(*stat).value(1));
                } else {
                    clause.add_atom(Atom::reducible().stat(*stat).value((lvl - 1) * 20));
                }
            }

            let mut req = Requirement::from(clause);

            req.name = Some("mantra_levels".into());

            ret.post.push(req);
        }

        // append on the presets if applicable
        for preset in self.use_presets.clone() {
            ret += preset;
        }

        ret.resolve_implicit(data);

        Ok(ret)
    }
}
