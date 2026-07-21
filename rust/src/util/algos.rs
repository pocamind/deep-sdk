/* algo implementations */

use crate::{
    Stat,
    data::{
        Aspect, DeepData, Enchant, Equipment, Mantra, Objective, Origin, Outfit, Resonance, Talent,
        Weapon,
    },
    enums::TalentRarity,
    error::{DeepError, Result},
    model::reqfile::Reqfile,
    model::stat::StatRange,
    req::{Atom, Clause, ClauseType, PrereqGroup, Reducability, Requirement},
    util::statmap::StatMap,
};

use crate::constants::KHAN_REQ_REDUCTION;
use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    ops::RangeInclusive,
};

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

const EXCLUSIVE_NAMESPACES: [&str; 3] = [Origin::NAMESPACE, Aspect::NAMESPACE, Outfit::NAMESPACE];

fn namespace_of(id: &str) -> &str {
    id.split_once(':').map_or(id, |(ns, _)| ns)
}

fn empty_named(name: &str) -> Requirement {
    let mut req = Requirement::new();
    req.name = Some(name.to_string());
    req
}

fn strictify(req: &Requirement) -> Requirement {
    let mut clauses: BTreeSet<Clause> = BTreeSet::new();

    for clause in &req.clauses {
        clauses.insert(Clause {
            clause_type: clause.clause_type.clone(),
            atoms: clause
                .atoms
                .iter()
                .cloned()
                .map(|a| a.reducability(Reducability::Strict))
                .collect(),
        });
    }

    Requirement {
        name: req.name.clone(),
        prereqs: req.prereqs.clone(),
        clauses,
    }
}

enum Emit {
    Skip,
    General(Requirement),
    Post(Requirement),
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

    /// Qualified ids (`ns:name`) of everything the build must obtain.
    pub reqs: Vec<String>,
    /// Qualified ids (`ns:name`) of reqs that are given as facts (origin, race).
    pub given: Vec<String>,
    /// Qualified ids (`ns:name`) from `reqs` that should be forced into a post-shrine stage.
    pub post: Vec<String>,
    /// Qualified talent ids granted by worn equipment/outfit/aspect. A granted id's own req is
    /// vacuous, but it does NOT satisfy prereqs (as per the game's granted talent semantics)
    pub granted: Vec<String>,

    pub required_mantra_levels: Option<StatMap>,
    pub race: Option<String>,

    pub final_ranges: HashMap<Stat, RangeInclusive<u32>>,

    /// Use optional reqfiles
    pub use_presets: Vec<Reqfile>,
}

impl BuildConfig {
    fn build_req(&self, data: &DeepData, id: &str) -> Result<Emit> {
        let (namespace, key) = id
            .split_once(':')
            .ok_or(DeepError::ReqfileBuild(format!("Unqualified id: {id}")))?;

        let emit = match namespace {
            Talent::NAMESPACE => {
                let talent = data.get_talent(key).ok_or(DeepError::ReqfileBuild(format!(
                    "Talent {id} not found in database"
                )))?;

                // exclude implicit reqs in the reqfile formulation
                if talent.implicit {
                    return Ok(Emit::Skip);
                }

                let req = talent.requirement(key);

                // oath root cards ("Oath: X") can only be acquired post-shrine, EXCEPT Oathless,
                // which is the one oath obtainable pre-shrine
                if talent.rarity == TalentRarity::Oath && talent.category != "Oathless" {
                    Emit::Post(req)
                } else {
                    Emit::General(req)
                }
            }
            Mantra::NAMESPACE => {
                let mantra = data.get_mantra(key).ok_or(DeepError::ReqfileBuild(format!(
                    "Mantra {id} not found in database"
                )))?;

                Emit::General(mantra.requirement(key))
            }
            Weapon::NAMESPACE => {
                let weapon = data.get_weapon(key).ok_or(DeepError::ReqfileBuild(format!(
                    "Weapon {id} not found in database"
                )))?;

                let mut req = if self.disable_som_weapons {
                    strictify(&weapon.requirement(key))
                } else {
                    weapon.requirement(key)
                };

                if self.is_khan(data)? {
                    req.add_to_stat_atoms(-KHAN_REQ_REDUCTION);
                }

                if self.allow_weapons_preshrine {
                    Emit::General(req)
                } else {
                    Emit::Post(req)
                }
            }
            Outfit::NAMESPACE => {
                let outfit = data.get_outfit(key).ok_or(DeepError::ReqfileBuild(format!(
                    "Outfit {id} not found in database"
                )))?;

                Emit::General(outfit.requirement(key))
            }
            Equipment::NAMESPACE => {
                let equipment = data
                    .get_equipment(key)
                    .ok_or(DeepError::ReqfileBuild(format!(
                        "Equipment {id} not found in database"
                    )))?;

                let mut req = equipment.requirement(key);

                if self.is_khan(data)? {
                    req.add_to_stat_atoms(-KHAN_REQ_REDUCTION);
                }

                Emit::General(req)
            }
            Objective::NAMESPACE => {
                let objective = data
                    .get_objective(key)
                    .ok_or(DeepError::ReqfileBuild(format!(
                        "Objective {id} not found in database"
                    )))?;

                Emit::General(objective.requirement(key))
            }
            Aspect::NAMESPACE | Origin::NAMESPACE | Resonance::NAMESPACE | Enchant::NAMESPACE => {
                Emit::General(data.requirement(id).ok_or(DeepError::ReqfileBuild(format!(
                    "{id} not found in database"
                )))?)
            }
            other => {
                return Err(DeepError::ReqfileBuild(format!(
                    "Unknown namespace '{other}' in id {id}"
                )));
            }
        };

        let emit = match emit {
            Emit::General(req) if self.post.iter().any(|p| p == id) => Emit::Post(req),
            other => other,
        };

        Ok(emit)
    }

    fn push_emit(ret: &mut Reqfile, emitted: &mut HashSet<String>, id: &str, emit: Emit) {
        match emit {
            Emit::Skip => {}
            Emit::General(req) => {
                ret.general.push(req);
                emitted.insert(id.to_string());
            }
            Emit::Post(req) => {
                ret.post.push(req);
                emitted.insert(id.to_string());
            }
        }
    }

    fn rewrite_edges(reqs: &mut [Requirement], known: &HashSet<String>) -> Result<()> {
        for req in reqs.iter_mut() {
            let mut groups: BTreeSet<PrereqGroup> = BTreeSet::new();

            for group in &req.prereqs {
                let present: Vec<&String> = group
                    .alternatives()
                    .filter(|alt| known.contains(*alt))
                    .collect();

                match present.len() {
                    0 => {
                        return Err(DeepError::ReqfileBuild(format!(
                            "Requirement '{}' has an unsatisfiable prereq group '{group}'",
                            req.name_or_default()
                        )));
                    }
                    1 => {
                        groups.insert(PrereqGroup::single(present[0]));
                    }
                    _ => {
                        log::warn!(
                            "requirement '{}': prereq group '{group}' has multiple present alternatives. using '{}'",
                            req.name_or_default(),
                            present[0]
                        );
                        groups.insert(PrereqGroup::single(present[0]));
                    }
                }
            }

            req.prereqs = groups;
        }

        Ok(())
    }

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

        ret.resolve_implicit(data);

        let given: HashSet<String> = self.given.iter().cloned().collect();

        let mut exclusive: HashMap<String, String> = HashMap::new();
        for id in self.reqs.iter().chain(self.given.iter()) {
            let ns = namespace_of(id);
            if EXCLUSIVE_NAMESPACES.contains(&ns) {
                if let Some(existing) = exclusive.get(ns) {
                    if existing != id {
                        return Err(DeepError::ReqfileBuild(format!(
                            "Conflicting {ns} entries: '{existing}' and '{id}' (only one {ns} allowed)"
                        )));
                    }
                } else {
                    exclusive.insert(ns.to_string(), id.clone());
                }
            }
        }

        let mut emitted: HashSet<String> = HashSet::new();

        let graph = data.prereq_graph();

        let granted: HashSet<String> = self.granted.iter().cloned().collect();
        let earned: HashSet<String> = self
            .reqs
            .iter()
            .flat_map(|id| graph.all_prereqs(id))
            .collect();
        let vacuous = |id: &String| granted.contains(id) && !earned.contains(id);

        for id in &self.reqs {
            let emit = match self.build_req(data, id)? {
                Emit::Skip => Emit::Skip,
                _ if given.contains(id) || vacuous(id) => Emit::General(empty_named(id)),
                emit => emit,
            };
            Self::push_emit(&mut ret, &mut emitted, id, emit);
        }

        let mut queue: VecDeque<String> = emitted.iter().cloned().collect();
        let mut walked: HashSet<String> = HashSet::new();

        while let Some(id) = queue.pop_front() {
            if !walked.insert(id.clone()) {
                continue;
            }

            if given.contains(&id) || vacuous(&id) {
                continue;
            }

            let Some(groups) = graph.prereqs(&id) else {
                continue;
            };

            for group in groups.clone() {
                if group.is_single() {
                    let alt = group.alternatives().next().unwrap().clone();

                    if emitted.contains(&alt) {
                        queue.push_back(alt);
                        continue;
                    }

                    if ret.implicit.contains_key(&alt) {
                        continue;
                    }

                    if given.contains(&alt) {
                        ret.general.push(empty_named(&alt));
                        emitted.insert(alt.clone());
                        queue.push_back(alt);
                        continue;
                    }

                    let ns = namespace_of(&alt);
                    if EXCLUSIVE_NAMESPACES.contains(&ns) {
                        if exclusive.contains_key(ns) {
                            continue;
                        }
                        exclusive.insert(ns.to_string(), alt.clone());
                    }

                    let emit = self.build_req(data, &alt)?;
                    Self::push_emit(&mut ret, &mut emitted, &alt, emit);
                    queue.push_back(alt);
                } else {
                    for alt in group.alternatives() {
                        if given.contains(alt) && !emitted.contains(alt) {
                            ret.general.push(empty_named(alt));
                            emitted.insert(alt.clone());
                            queue.push_back(alt.clone());
                        }
                    }
                }
            }
        }

        let mut known = emitted;
        known.extend(ret.implicit.keys().cloned());

        Self::rewrite_edges(&mut ret.general, &known)?;
        Self::rewrite_edges(&mut ret.post, &known)?;

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

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE_PATH: &str = "/home/niooi/projects/deep/data/.dist/all.json";

    fn load_data() -> DeepData {
        let json = std::fs::read_to_string(BUNDLE_PATH).expect("read all.json bundle");
        DeepData::from_json(&json).expect("parse bundle")
    }

    fn config(reqs: &[&str], given: &[&str], race: Option<&str>) -> BuildConfig {
        BuildConfig {
            disable_som_weapons: false,
            allow_weapons_preshrine: false,
            reqs: reqs.iter().map(ToString::to_string).collect(),
            given: given.iter().map(ToString::to_string).collect(),
            post: vec![],
            granted: vec![],
            required_mantra_levels: None,
            race: race.map(ToString::to_string),
            final_ranges: HashMap::new(),
            use_presets: vec![],
        }
    }

    fn known_names(rf: &Reqfile) -> HashSet<String> {
        rf.req_iter()
            .map(Requirement::name_or_default)
            .chain(rf.implicit.keys().cloned())
            .collect()
    }

    fn single_atom_value(req: &Requirement) -> i64 {
        req.clauses
            .iter()
            .next()
            .unwrap()
            .atoms
            .iter()
            .next()
            .unwrap()
            .value
    }

    #[test]
    fn closure_resolves_origin_prereq() {
        let data = load_data();
        let rf = config(
            &["talent:voidwalker_contract"],
            &["origin:voidwalker"],
            None,
        )
        .to_reqfile(&data)
        .unwrap();

        let origin = rf
            .req_iter()
            .find(|r| r.name.as_deref() == Some("origin:voidwalker"))
            .expect("origin emitted");
        assert!(origin.is_empty());
        assert!(origin.prereqs.is_empty());

        let names = known_names(&rf);
        for req in rf.req_iter() {
            for group in &req.prereqs {
                assert!(
                    group.alternatives().all(|a| names.contains(a)),
                    "dangling prereq group '{group}' on '{}'",
                    req.name_or_default()
                );
            }
        }
    }

    #[test]
    fn closure_missing_alternative_errors() {
        let data = load_data();
        let err = config(&["talent:voidwalker_contract"], &["origin:castaway"], None)
            .to_reqfile(&data)
            .unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("origin:voidwalker"), "unexpected error: {msg}");
    }

    #[test]
    fn two_origins_exclusive_error() {
        let data = load_data();
        let err = config(
            &["talent:voidwalker_contract"],
            &["origin:voidwalker", "origin:castaway"],
            None,
        )
        .to_reqfile(&data)
        .unwrap_err();

        let msg = err.to_string();
        assert!(
            msg.contains("Conflicting origin"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn oath_timing_split() {
        let data = load_data();
        let rf = config(
            &["talent:oath_arcwarder", "talent:oath_oathless"],
            &[],
            None,
        )
        .to_reqfile(&data)
        .unwrap();

        assert!(
            rf.post
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:oath_arcwarder"))
        );
        assert!(
            rf.general
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:oath_oathless"))
        );
        assert!(
            !rf.post
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:oath_oathless"))
        );
    }

    #[test]
    fn granted_vacuous_unless_depended_on() {
        let data = load_data();

        let mut cfg = config(&["talent:a_world_without_song"], &[], None);
        cfg.granted = vec!["talent:a_world_without_song".to_string()];
        let rf = cfg.to_reqfile(&data).unwrap();
        let req = rf
            .general
            .iter()
            .find(|r| r.name.as_deref() == Some("talent:a_world_without_song"))
            .expect("granted req emitted");
        assert!(req.is_empty());
        assert!(
            !rf.req_iter()
                .any(|r| r.name.as_deref() == Some("talent:silencers_blade"))
        );

        let mut cfg = config(
            &["talent:silencers_blade", "talent:a_world_without_song"],
            &[],
            None,
        );
        cfg.granted = vec!["talent:silencers_blade".to_string()];
        let rf = cfg.to_reqfile(&data).unwrap();
        let blade = rf
            .req_iter()
            .find(|r| r.name.as_deref() == Some("talent:silencers_blade"))
            .expect("depended-on granted req emitted");
        assert!(!blade.is_empty());
    }

    #[test]
    fn multi_present_alternatives_pin_first() {
        let data = load_data();
        let rf = config(
            &[
                "talent:meteor_impact",
                "mantra:rising_flame",
                "mantra:rising_frost",
            ],
            &[],
            None,
        )
        .to_reqfile(&data)
        .unwrap();

        let meteor = rf
            .req_iter()
            .find(|r| r.name.as_deref() == Some("talent:meteor_impact"))
            .expect("meteor emitted");
        assert_eq!(meteor.prereqs.len(), 1);
        let group = meteor.prereqs.iter().next().unwrap();
        assert!(group.is_single());
        assert_eq!(group.alternatives().next().unwrap(), "mantra:rising_flame");
    }

    #[test]
    fn given_overrides_req_as_empty() {
        let data = load_data();
        let rf = config(
            &["talent:a_world_without_song"],
            &["talent:a_world_without_song"],
            None,
        )
        .to_reqfile(&data)
        .unwrap();

        let req = rf
            .general
            .iter()
            .find(|r| r.name.as_deref() == Some("talent:a_world_without_song"))
            .expect("given req emitted");
        assert!(req.is_empty());
        assert!(
            !rf.req_iter()
                .any(|r| r.name.as_deref() == Some("talent:silencers_blade"))
        );
    }

    #[test]
    fn post_hint_forces_post() {
        let data = load_data();
        let mut cfg = config(&["talent:a_world_without_song"], &[], None);
        cfg.post = vec!["talent:a_world_without_song".to_string()];
        let rf = cfg.to_reqfile(&data).unwrap();

        assert!(
            rf.post
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:a_world_without_song"))
        );
        assert!(
            !rf.general
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:a_world_without_song"))
        );
        assert!(
            rf.general
                .iter()
                .any(|r| r.name.as_deref() == Some("talent:silencers_blade"))
        );
    }

    #[test]
    fn khan_deducts_weapon_and_equipment() {
        let data = load_data();
        let rf = config(
            &[
                "weapon:acherons_warspear",
                "equipment:black_cape",
                "equipment:rogue_assassins_hood",
            ],
            &[],
            Some("khan"),
        )
        .to_reqfile(&data)
        .unwrap();

        let weapon = rf
            .post
            .iter()
            .find(|r| r.name.as_deref() == Some("weapon:acherons_warspear"))
            .expect("weapon emitted post");
        assert_eq!(single_atom_value(weapon), 37);

        let hood = rf
            .general
            .iter()
            .find(|r| r.name.as_deref() == Some("equipment:rogue_assassins_hood"))
            .expect("equipment emitted general");
        assert_eq!(single_atom_value(hood), 7);

        let cape = rf
            .general
            .iter()
            .find(|r| r.name.as_deref() == Some("equipment:black_cape"))
            .expect("equipment emitted general");
        assert_eq!(single_atom_value(cape), 90);
    }

    #[test]
    fn or_group_resolves_present_alternative() {
        let data = load_data();
        let rf = config(&["outfit:stranded"], &["origin:castaway"], None)
            .to_reqfile(&data)
            .unwrap();

        let outfit = rf
            .req_iter()
            .find(|r| r.name.as_deref() == Some("outfit:stranded"))
            .expect("outfit emitted");
        assert_eq!(outfit.prereqs.len(), 1);

        let group = outfit.prereqs.iter().next().unwrap();
        assert!(group.is_single());
        assert_eq!(group.alternatives().next().unwrap(), "origin:castaway");

        assert!(
            rf.req_iter()
                .any(|r| r.name.as_deref() == Some("origin:castaway") && r.is_empty())
        );
    }
}
