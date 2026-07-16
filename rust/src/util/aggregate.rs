use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::model::data::DeepData;
use crate::util::pips;
use crate::util::statmap::StatMap;

const TRAIT_CAP: i64 = 6;
const POWER_CAP: i64 = 20;

/// Starting stat values (see wiki)
/// TODO! use a static/lazy initiliazed map or smth,
/// but tbh at this scale a linear search is probably faster 😹😹😹😹😹😹😹😹
const STARTING_FLAT: &[(&str, f64)] = &[
    ("Health", 220.0),
    ("Posture", 20.0),
    ("Ether", 120.0),
    ("Tempo", 120.0),
    ("Sanity", 80.0),
    ("Carry Load", 100.0),
];

const PERCENT_STATS: &[&str] = &["Physical Armor", "Elemental Armor", "Knockback Resistance"];

// Map the left stat into the right stat
const ALIASES: &[(&str, &str)] = &[
    ("Physical Armor", "Physical Resistance"),
    ("Elemental Armor", "Elemental Resistance"),
];

#[must_use]
pub fn is_percent_stat(stat: &str) -> bool {
    PERCENT_STATS.contains(&stat)
        || stat.ends_with(" Armor")
        || stat.ends_with(" Resistance")
        || stat == "Pen"
        || stat == "Melee Pen"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatSource {
    pub value: f64,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EquipmentSelection {
    pub name: String,
    pub pips: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildSnapshot {
    pub stats: StatMap,
    pub race: String,
    pub talents: Vec<String>,
    pub boons: Vec<String>,
    pub traits: HashMap<String, i64>,
    pub equipment: Vec<EquipmentSelection>,
    pub outfit: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub flat: HashMap<String, Vec<StatSource>>,
    pub percents: HashMap<String, Vec<StatSource>>,
    pub derived: HashMap<String, f64>,
}

impl AggregatedStats {
    #[must_use]
    pub fn flat_totals(&self) -> HashMap<String, f64> {
        totals(&self.flat)
    }

    #[must_use]
    pub fn percent_totals(&self) -> HashMap<String, f64> {
        totals(&self.percents)
    }
}

fn totals(map: &HashMap<String, Vec<StatSource>>) -> HashMap<String, f64> {
    map.iter()
        .map(|(stat, entries)| (stat.clone(), entries.iter().map(|e| e.value).sum()))
        .collect()
}

#[must_use]
#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
pub fn aggregate(data: &DeepData, build: &BuildSnapshot) -> AggregatedStats {
    fn add(map: &mut HashMap<String, Vec<StatSource>>, stat: &str, value: f64, source: &str) {
        if value == 0.0 {
            return;
        }
        map.entry(stat.to_string()).or_default().push(StatSource {
            value,
            source: source.to_string(),
        });
    }

    let mut flat: HashMap<String, Vec<StatSource>> = HashMap::new();
    let mut percents: HashMap<String, Vec<StatSource>> = HashMap::new();

    for (stat, value) in STARTING_FLAT {
        add(&mut flat, stat, *value, "Starting");
    }

    let stats = &build.stats;
    let level = stats.level().min(POWER_CAP);
    let str_ = stats.get(&Stat::Strength);
    let ftd = stats.get(&Stat::Fortitude);
    let agl = stats.get(&Stat::Agility);
    let int = stats.get(&Stat::Intelligence);
    let wll = stats.get(&Stat::Willpower);
    let cha = stats.get(&Stat::Charisma);
    let vitality = build.traits.get("Vitality").copied().unwrap_or(0).min(TRAIT_CAP);
    let erudition = build.traits.get("Erudition").copied().unwrap_or(0).min(TRAIT_CAP);

    let sanity_mult = if build.race == "Ganymede" { 1.2 } else { 1.0 };
    let stealth_mult = if build.race == "Felinor" { 1.2 } else { 1.0 };

    add(&mut flat, "Health", 4.0 * level as f64, &format!("Power {level}"));
    let ftd_health = if ftd <= 50 {
        ftd as f64 * 0.5
    } else {
        25.0 + (ftd - 50) as f64 * 0.25
    };
    add(&mut flat, "Health", ftd_health, &format!("{ftd} Fortitude"));
    add(&mut flat, "Health", 10.0 * vitality as f64, &format!("{vitality} Vitality"));

    add(&mut flat, "Ether", 2.0 * int as f64, &format!("{int} Intelligence"));
    add(&mut flat, "Ether", 1.5 * cha as f64, &format!("{cha} Charisma"));

    add(&mut flat, "Sanity", 3.0 * wll as f64 * sanity_mult, &format!("{wll} Willpower"));

    add(&mut flat, "Tempo", 0.5 * wll as f64, &format!("{wll} Willpower"));
    add(&mut flat, "Tempo", 5.0 * erudition as f64, &format!("{erudition} Erudition"));

    add(&mut flat, "Stealth", 0.2 * agl as f64 * stealth_mult, &format!("{agl} Agility"));
    if build.race == "Felinor" {
        add(&mut flat, "Stealth", 20.0, "Felinor");
    }

    add(&mut percents, "Pen", 0.1 * str_ as f64, &format!("{str_} Strength"));

    add(&mut flat, "Carry Load", (str_ as f64 * 0.5).min(50.0), &format!("{str_} Strength"));
    add(&mut flat, "Carry Load", (ftd as f64 * 0.5).min(50.0), &format!("{ftd} Fortitude"));
    if build.boons.iter().any(|b| b == "Packmule") {
        add(&mut flat, "Carry Load", 50.0, "Packmule");
    }

    for name in &build.talents {
        let Some(talent) = data.get_talent(name) else {
            continue;
        };
        for (stat, value) in &talent.stats {
            let target = if is_percent_stat(stat) { &mut percents } else { &mut flat };
            add(target, stat, *value, &format!("Talent: {name}"));
        }
    }

    for selection in &build.equipment {
        let Some(equip) = data.get_equipment(&selection.name) else {
            continue;
        };
        let source = format!("Equipment: {}", selection.name);

        for (stat, innate) in &equip.innates {
            let target = if innate.percentage { &mut percents } else { &mut flat };
            add(target, stat, innate.value, &source);
        }

        for (rarity, chosen) in &selection.pips {
            for pip in chosen {
                for (stat, value) in pips::pip_stats(pip, equip.equipment_type, rarity) {
                    let target = if is_percent_stat(stat) { &mut percents } else { &mut flat };
                    add(target, stat, value, &source);
                }
            }
        }
    }

    if let Some(outfit_name) = &build.outfit
        && let Some(outfit) = data.get_outfit(outfit_name)
    {
        let source = format!("Outfit: {outfit_name}");
        for (stat, value) in &outfit.resistances {
            add(&mut percents, stat, *value, &source);
        }
        for (stat, value) in &outfit.extra_percents {
            add(&mut percents, stat, *value as f64, &source);
        }
    }

    for map in [&mut flat, &mut percents] {
        for (from, to) in ALIASES {
            if let Some(entries) = map.remove(*from) {
                map.entry((*to).to_string()).or_default().extend(entries);
            }
        }

        for entries in map.values_mut() {
            let mut by_source: Vec<(String, f64)> = Vec::new();
            for entry in entries.drain(..) {
                match by_source.iter_mut().find(|(s, _)| *s == entry.source) {
                    Some((_, v)) => *v += entry.value,
                    None => by_source.push((entry.source, entry.value)),
                }
            }
            *entries = by_source
                .into_iter()
                .map(|(source, value)| StatSource { value, source })
                .collect();
            entries.sort_by(|a, b| b.value.total_cmp(&a.value));
        }
    }

    let mut result = AggregatedStats {
        flat,
        percents,
        derived: HashMap::new(),
    };
    result.derived = get_derived(&result.flat_totals(), &result.percent_totals());

    result
}

/// Derived stats are stats that are derived from other base ones, such as EHP.
fn get_derived(flat: &HashMap<String, f64>, percents: &HashMap<String, f64>) -> HashMap<String, f64> {
    let mut derived = HashMap::new();

    let health = flat.get("Health").copied().unwrap_or(0.0);

    for (name, resistance) in [
        ("Physical EHP", "Physical Resistance"),
        ("Elemental EHP", "Elemental Resistance"),
    ] {
        let res = percents.get(resistance).copied().unwrap_or(0.0).clamp(0.0, 99.0);
        derived.insert(name.to_string(), health / (1.0 - res / 100.0));
    }

    derived
}

#[must_use]
/// Returns an unstructured diff (prob make structured later) of two pocamind/data versions
pub fn changed_items(old: &DeepData, new: &DeepData) -> Vec<String> {
    let mut changed = Vec::new();

    for talent in new.talents() {
        match old.get_talent(&talent.name) {
            Some(prev) if prev.stats == talent.stats => {}
            _ => changed.push(talent.name.clone()),
        }
    }
    for talent in old.talents() {
        if new.get_talent(&talent.name).is_none() {
            changed.push(talent.name.clone());
        }
    }

    for equip in new.equipment() {
        match old.get_equipment(&equip.name) {
            Some(prev)
                if prev.innates == equip.innates
                    && prev.pips == equip.pips
                    && prev.talents == equip.talents => {}
            _ => changed.push(equip.name.clone()),
        }
    }
    for equip in old.equipment() {
        if new.get_equipment(&equip.name).is_none() {
            changed.push(equip.name.clone());
        }
    }

    for outfit in new.outfits() {
        match old.get_outfit(&outfit.name) {
            Some(prev)
                if prev.resistances == outfit.resistances
                    && prev.extra_percents == outfit.extra_percents
                    && prev.talent == outfit.talent => {}
            _ => changed.push(outfit.name.clone()),
        }
    }
    for outfit in old.outfits() {
        if new.get_outfit(&outfit.name).is_none() {
            changed.push(outfit.name.clone());
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_stat_routing() {
        assert!(is_percent_stat("Physical Armor"));
        assert!(is_percent_stat("Shadow Resistance"));
        assert!(is_percent_stat("Pen"));
        assert!(!is_percent_stat("Health"));
        assert!(!is_percent_stat("Carry Load"));
    }

    #[test]
    fn empty_build_gets_starting_stats() {
        let data = DeepData::default();
        let result = aggregate(&data, &BuildSnapshot::default());

        let flat = result.flat_totals();
        assert_eq!(flat.get("Health"), Some(&220.0));
        assert_eq!(flat.get("Ether"), Some(&120.0));
        assert_eq!(result.derived.get("Physical EHP"), Some(&220.0));
    }

    #[test]
    fn fortitude_health_curve_halves_past_50() {
        let data = DeepData::default();
        let mut build = BuildSnapshot::default();
        build.stats.insert(Stat::Fortitude, 80);

        let result = aggregate(&data, &build);
        let fortitude_health = result.flat["Health"]
            .iter()
            .find(|e| e.source == "80 Fortitude")
            .unwrap();
        assert_eq!(fortitude_health.value, 25.0 + 30.0 * 0.25);
    }

    #[test]
    fn aliases_merge_into_canonical_name() {
        let data = DeepData::default();
        let result = aggregate(&data, &BuildSnapshot::default());
        assert!(!result.percents.contains_key("Physical Armor"));
    }

    #[test]
    fn sources_coalesce_and_sort_descending() {
        let data = DeepData::default();
        let mut build = BuildSnapshot::default();
        build.stats.insert(Stat::Willpower, 40);
        build.traits.insert("Erudition".to_string(), 2);

        let result = aggregate(&data, &build);
        let tempo = &result.flat["Tempo"];
        assert_eq!(tempo[0].source, "Starting");
        assert!(tempo.windows(2).all(|w| w[0].value >= w[1].value));
    }

    #[test]
    fn identical_data_has_no_changed_items() {
        let data = DeepData::default();
        assert!(changed_items(&data, &data).is_empty());
    }
}
