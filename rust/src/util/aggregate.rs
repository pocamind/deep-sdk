use std::collections::{HashMap, HashSet};

use crate::Stat;
use crate::constants::{
    CARRY_LOAD_PER_FORTITUDE, CARRY_LOAD_PER_STAT_CAP, CARRY_LOAD_PER_STRENGTH,
    ECHO_CARRY_LOAD, ETHER_PER_CHARISMA, ETHER_PER_ERUDITION, ETHER_PER_INTELLIGENCE, FELINOR_STEALTH_FLAT,
    FELINOR_STEALTH_MULT, FORTITUDE_HEALTH_KNEE, GANYMEDE_SANITY_MULT, HEALTH_PER_FORTITUDE,
    HEALTH_PER_FORTITUDE_PAST_KNEE, HEALTH_PER_LEVEL, HEALTH_PER_VITALITY,
    PACKMULE_CARRY_LOAD, PEN_CAP, PEN_CAP_BREAKERS, PEN_CAP_LIFTED, PEN_PER_STRENGTH,
    PEN_PER_TRAIT_POINT, SANITY_PER_WILLPOWER, STARTING_FLAT,
    STEALTH_PER_AGILITY, TEMPO_GAIN_PER_ERUDITION, TEMPO_PER_ERUDITION, TEMPO_PER_WILLPOWER,
    TRAIT_CAP,
};

use crate::formulas::{self, CombatState};
use crate::model::aggregate::{
    BuildParams, BuildTotalStats, DamageGroup, DamageKind, Scenario, StarMod, StatOrigin,
    StatSource,
};
use crate::model::enums::EquipmentSlot;
use crate::model::data::DeepData;
use crate::model::formula::StatFormula;
use crate::util::pips;
use crate::util::statmap::StatMap;


const PERCENT_STATS: &[&str] = &[
    "Pen",
    "Melee Pen",
    "Mantra Pen",
    "Damage",
    "Damage vs. Monsters",
    "Bleed",
];

/// Map the left stat into the right stat as a hacky workaround.
const ALIASES: &[(&str, &str)] = &[
    ("Physical Armor", "Physical Resistance"),
    ("Elemental Armor", "Elemental Resistance"),
];

#[must_use]
pub fn is_percent_stat(stat: &str) -> bool {
    PERCENT_STATS.contains(&stat) || stat.ends_with(" Armor") || stat.ends_with(" Resistance")
}

/// Every talent the build actually (should) have, deduplicated.
/// Adds racial innates, outfit, equipment talents, etc. The ones granted
/// by granted_talents
#[must_use]
pub fn all_talents(data: &DeepData, build: &BuildParams) -> Vec<String> {
    let mut seen = HashSet::new();
    build
        .talents
        .iter()
        .cloned()
        .chain(granted_talents(data, build))
        .filter(|name| seen.insert(name.clone()))
        .collect()
}

/// Talents the build must have, granted by equipment, the outfit, the
/// aspect, or implicitly by meeting stat requirements
#[must_use]
pub fn granted_talents(data: &DeepData, build: &BuildParams) -> Vec<String> {
    let mut seen: HashSet<String> = build.talents.iter().cloned().collect();
    let mut granted = Vec::new();

    let mut push = |name: &str| {
        if seen.insert(name.to_string()) {
            granted.push(name.to_string());
        }
    };

    for selection in &build.equipment {
        if let Some(equip) = data.get_equipment(&selection.name) {
            for name in &equip.talents {
                push(name);
            }
        }
    }

    if let Some(outfit_name) = &build.outfit
        && let Some(outfit) = data.get_outfit(outfit_name)
        && let Some(name) = &outfit.talent
    {
        push(name);
    }

    if let Some(aspect) = data.get_aspect(&build.race) {
        for name in &aspect.talent {
            push(name);
        }
    }

    for talent in build.stats.implicit_talents(data) {
        push(&talent.name);
    }

    granted
}

#[must_use]
#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
/// Mega function to calculate a build's total stats, including derived stats such as EHP, DPS, etc.
pub fn aggregate(data: &DeepData, build: &BuildParams, scenario: Scenario) -> BuildTotalStats {
    let Scenario { mode, combat_state, .. } = scenario;

    fn add(
        map: &mut HashMap<String, Vec<StatSource>>,
        stat: &str,
        value: f64,
        source: &str,
        origin: StatOrigin,
    ) {
        if value == 0.0 {
            return;
        }
        map.entry(stat.to_string()).or_default().push(StatSource {
            value,
            source: source.to_string(),
            origin,
        });
    }

    fn eval(formula: &StatFormula, stats: &StatMap, state: CombatState, source: &str, stat: &str) -> f64 {
        match formula.eval(stats, state) {
            Ok(value) => value,
            Err(e) => {
                log::warn!("{source} / {stat}: {e}");
                0.0
            }
        }
    }

    let mut flat: HashMap<String, Vec<StatSource>> = HashMap::new();
    let mut percents: HashMap<String, Vec<StatSource>> = HashMap::new();
    let mut multiplicative_percents: HashMap<String, Vec<StatSource>> = HashMap::new();

    // Starting values
    for (stat, value) in STARTING_FLAT {
        add(&mut flat, stat, *value, "Starting", StatOrigin::Base);
    }

    // Attribute investment and trait scaling
    let stats = &build.stats;
    let level = stats.level(None);
    let str_ = stats.get(&Stat::Strength);
    let ftd = stats.get(&Stat::Fortitude);
    let agl = stats.get(&Stat::Agility);
    let int = stats.get(&Stat::Intelligence);
    let wll = stats.get(&Stat::Willpower);
    let cha = stats.get(&Stat::Charisma);
    let vitality = build.traits.get("Vitality").copied().unwrap_or(0).min(TRAIT_CAP);
    let erudition = build.traits.get("Erudition").copied().unwrap_or(0).min(TRAIT_CAP);
    let proficiency = build.traits.get("Proficiency").copied().unwrap_or(0).min(TRAIT_CAP);
    let songchant = build.traits.get("Songchant").copied().unwrap_or(0).min(TRAIT_CAP);

    add(&mut flat, "Health", HEALTH_PER_LEVEL * level as f64, &format!("Power {level}"), StatOrigin::Base);
    let ftd_health = if ftd <= FORTITUDE_HEALTH_KNEE {
        ftd as f64 * HEALTH_PER_FORTITUDE
    } else {
        FORTITUDE_HEALTH_KNEE as f64 * HEALTH_PER_FORTITUDE
            + (ftd - FORTITUDE_HEALTH_KNEE) as f64 * HEALTH_PER_FORTITUDE_PAST_KNEE
    };
    add(&mut flat, "Health", ftd_health, &format!("{ftd} Fortitude"), StatOrigin::Base);
    add(&mut flat, "Health", HEALTH_PER_VITALITY * vitality as f64, &format!("{vitality} Vitality"), StatOrigin::Base);

    add(&mut flat, "Ether", ETHER_PER_INTELLIGENCE * int as f64, &format!("{int} Intelligence"), StatOrigin::Base);
    add(&mut flat, "Ether", ETHER_PER_CHARISMA * cha as f64, &format!("{cha} Charisma"), StatOrigin::Base);
    add(&mut flat, "Ether", ETHER_PER_ERUDITION * erudition as f64, &format!("{erudition} Erudition"), StatOrigin::Base);

    add(&mut flat, "Sanity", SANITY_PER_WILLPOWER * wll as f64, &format!("{wll} Willpower"), StatOrigin::Base);

    add(&mut flat, "Tempo", TEMPO_PER_WILLPOWER * wll as f64, &format!("{wll} Willpower"), StatOrigin::Base);
    add(&mut flat, "Tempo", TEMPO_PER_ERUDITION * erudition as f64, &format!("{erudition} Erudition"), StatOrigin::Base);
    add(&mut percents, "Tempo Gain", TEMPO_GAIN_PER_ERUDITION * erudition as f64, &format!("{erudition} Erudition"), StatOrigin::Base);

    add(&mut flat, "Stealth", STEALTH_PER_AGILITY * agl as f64, &format!("{agl} Agility"), StatOrigin::Base);

    add(&mut percents, "Pen", PEN_PER_STRENGTH * str_ as f64, &format!("{str_} Strength"), StatOrigin::Base);
    add(&mut percents, "Melee Pen", PEN_PER_TRAIT_POINT * proficiency as f64, &format!("{proficiency} Proficiency"), StatOrigin::Base);
    add(&mut percents, "Mantra Pen", PEN_PER_TRAIT_POINT * songchant as f64, &format!("{songchant} Songchant"), StatOrigin::Base);

    // weapon pen contribution
    if let Some((selection, weapon)) = build
        .weapon
        .as_ref()
        .and_then(|selection| Some((selection, data.get_weapon(&selection.name)?)))
    {
        let star = if selection.star_mod() == Some(StarMod::Penetration) {
            selection.star_bonus()
        } else {
            0.0
        };
        let pen = (weapon.penetration.unwrap_or(0.0) + star) * 100.0;
        if pen != 0.0 {
            add(&mut percents, "Melee Pen", pen, &selection.name, StatOrigin::Equipment);
        }
    }

    add(&mut flat, "Carry Load", ECHO_CARRY_LOAD, "Echo upgrades", StatOrigin::Base);
    add(&mut flat, "Carry Load", (str_ as f64 * CARRY_LOAD_PER_STRENGTH).min(CARRY_LOAD_PER_STAT_CAP), &format!("{str_} Strength"), StatOrigin::Base);
    add(&mut flat, "Carry Load", (ftd as f64 * CARRY_LOAD_PER_FORTITUDE).min(CARRY_LOAD_PER_STAT_CAP), &format!("{ftd} Fortitude"), StatOrigin::Base);
    if build.boons.iter().any(|b| b == "Packmule") {
        add(&mut flat, "Carry Load", PACKMULE_CARRY_LOAD, "Packmule", StatOrigin::Base);
    }

    // Collect all other sources of stats
    let enchants: Vec<&str> = build
        .weapon
        .iter()
        .filter_map(|w| w.enchant.as_deref())
        .chain(build.equipment.iter().filter_map(|e| e.enchant.as_deref()))
        .collect();

    let talents = all_talents(data, build);

    let sources = talents
        .iter()
        .filter_map(|name| {
            let talent = data.get_talent(name)?;
            Some((format!("Talent: {name}"), StatOrigin::Talent, &talent.contributions))
        })
        .chain(build.mantras.iter().filter_map(|selection| {
            let mantra = data.get_mantra(&selection.name)?;
            let source = format!("Mantra: {}", selection.name);
            Some((source, StatOrigin::Mantra, &mantra.contributions))
        }))
        .chain(enchants.into_iter().filter_map(|name| {
            let enchant = data.get_enchant(name)?;
            Some((format!("Enchant: {name}"), StatOrigin::Equipment, &enchant.contributions))
        }));

    let optimistic = mode.is_optimistic();

    for (source, origin, contributions) in sources {
        for map in contributions.additive(optimistic) {
            for (stat, formula) in map {
                let target = if is_percent_stat(stat) { &mut percents } else { &mut flat };
                add(target, stat, eval(formula, stats, combat_state, &source, stat), &source, origin);
            }
        }
        for map in contributions.multiplicative(optimistic) {
            for (stat, formula) in map {
                add(
                    &mut multiplicative_percents,
                    stat,
                    eval(formula, stats, combat_state, &source, stat),
                    &source,
                    origin,
                );
            }
        }
    }

    for selection in &build.equipment {
        let Some(equip) = data.get_equipment(&selection.name) else {
            continue;
        };
        let source = format!("Equipment: {}", selection.name);

        for (stat, innate) in &equip.innates {
            let target = if innate.percentage { &mut percents } else { &mut flat };
            let value = eval(&innate.value, stats, combat_state, &source, stat);
            add(target, stat, value, &source, StatOrigin::Equipment);
        }

        for (rarity, chosen) in &selection.pips {
            for pip in chosen {
                for (stat, value) in pips::pip_stats(pip, equip.equipment_type, rarity) {
                    let target = if is_percent_stat(stat) { &mut percents } else { &mut flat };
                    add(target, stat, value, &source, StatOrigin::Equipment);
                }
            }
        }

        if matches!(
            equip.equipment_type,
            EquipmentSlot::Head | EquipmentSlot::Arms | EquipmentSlot::Legs
        ) {
            let stars = f64::from(selection.stars);
            add(&mut flat, "Health", stars, &source, StatOrigin::Equipment);
        }

    }

    if let Some(outfit_name) = &build.outfit
        && let Some(outfit) = data.get_outfit(outfit_name)
    {
        let source = format!("Outfit: {outfit_name}");
        for (stat, value) in &outfit.resistances {
            add(&mut percents, stat, *value, &source, StatOrigin::Outfit);
        }
        for (stat, value) in &outfit.extra_percents {
            add(&mut percents, stat, *value as f64, &source, StatOrigin::Outfit);
        }
    }

    // Racial contributions
    // Ganymede skips base Sanity, and Felinor's bonus is flat and is added after its own multiplier.
    let sum_of = |map: &HashMap<String, Vec<StatSource>>, stat: &str, skip_base: bool| -> f64 {
        map.get(stat).map_or(0.0, |entries| {
            entries
                .iter()
                .filter(|e| !(skip_base && e.source == "Starting"))
                .map(|e| e.value)
                .sum()
        })
    };

    // Racial scaling
    if build.race == "Ganymede" {
        let scaled = sum_of(&flat, "Sanity", true) * (GANYMEDE_SANITY_MULT - 1.0);
        add(&mut flat, "Sanity", scaled, "Ganymede", StatOrigin::Base);
    }

    if build.race == "Felinor" {
        let scaled = sum_of(&flat, "Stealth", false) * (FELINOR_STEALTH_MULT - 1.0);
        add(&mut flat, "Stealth", scaled, "Felinor", StatOrigin::Base);
        add(&mut flat, "Stealth", FELINOR_STEALTH_FLAT, "Felinor", StatOrigin::Base);
    }

    // Resolve aliases
    for map in [&mut flat, &mut percents, &mut multiplicative_percents] {
        for (from, to) in ALIASES {
            if let Some(entries) = map.remove(*from) {
                map.entry((*to).to_string()).or_default().extend(entries);
            }
        }
    }

    // Turn the generic 'Pen' stat into Melee and Mantra pen
    for map in [&mut percents, &mut multiplicative_percents] {
        if let Some(backbone) = map.remove("Pen") {
            for channel in ["Melee Pen", "Mantra Pen"] {
                for entry in &backbone {
                    add(map, channel, entry.value, &entry.source, entry.origin);
                }
            }
        }
    }

    // Resolve multiplicative stat entries (cheap shot and some enchants afaik) 
    // into the main count
    for (stat, entries) in std::mem::take(&mut multiplicative_percents) {
        let base = if is_percent_stat(&stat) {
            sum_of(&percents, &stat, false)
        } else {
            sum_of(&flat, &stat, false)
        };
        let target = if is_percent_stat(&stat) { &mut percents } else { &mut flat };
        for entry in entries {
            add(target, &stat, base * entry.value / 100.0, &entry.source, entry.origin);
        }
    }

    // Clamp pen if not limit broken
    let uncapped = talents.iter().any(|t| PEN_CAP_BREAKERS.contains(&t.as_str()));
    let pen_cap = 100.0 * if uncapped { PEN_CAP_LIFTED } else { PEN_CAP };
    for channel in ["Melee Pen", "Mantra Pen"] {
        let total = sum_of(&percents, channel, false);
        if total > pen_cap {
            add(&mut percents, channel, pen_cap - total, "PEN cap", StatOrigin::Base);
        }
    }

    // resolve source list
    for map in [&mut flat, &mut percents] {
        for entries in map.values_mut() {
            let mut by_source: Vec<(String, f64, StatOrigin)> = Vec::new();
            for entry in entries.drain(..) {
                match by_source.iter_mut().find(|(s, _, _)| *s == entry.source) {
                    Some((_, v, _)) => *v += entry.value,
                    None => by_source.push((entry.source, entry.value, entry.origin)),
                }
            }
            *entries = by_source
                .into_iter()
                .map(|(source, value, origin)| StatSource { value, source, origin })
                .collect();
            entries.sort_by(|a, b| b.value.total_cmp(&a.value));
        }
    }

    let mut result = BuildTotalStats {
        flat,
        percents,
        derived: HashMap::new(),
    };
    result.derived = get_derived(data, build, &result, &talents, scenario);

    result
}

/// Derived stats are stats that are derived from other base ones, such as EHP, DPS, etc.
fn get_derived(
    data: &DeepData,
    build: &BuildParams,
    stats: &BuildTotalStats,
    talents: &[String],
    scenario: Scenario,
) -> HashMap<String, f64> {
    let mut derived = HashMap::new();

    let percent = stats.percent_totals();

    // YOUR pen resist
    let pen_resist = percent.get("Pen Resistance").copied().unwrap_or(0.0) / 100.0;
    // The pen the enemy has
    let faced_pen = scenario.enemy_pen / 100.0 * (1.0 - pen_resist);
    // YOUR melee pen
    let melee_pen = percent.get("Melee Pen").copied().unwrap_or(0.0) / 100.0;
    // The resistance your enemy has
    let enemy_resistance = scenario.enemy_resistance / 100.0;

    if let Some((m1, dps)) =
        formulas::weapon_damage(data, build, talents, &percent, scenario.combat_state)
    {
        derived.insert("M1 Damage".to_string(), m1);
        if let Some(dps) = dps {
            derived.insert(
                "DPS".to_string(),
                formulas::damage_after_resistance(dps, enemy_resistance, melee_pen),
            );
        }
    }

    let kinds = [DamageGroup::Physical, DamageGroup::Elemental]
        .into_iter()
        .flat_map(|group| {
            std::iter::once(DamageKind::from(group))
                .chain(group.types().iter().copied().map(DamageKind::from))
        });

    let health = stats.flat_totals().get("Health").copied().unwrap_or(0.0);

    for kind in kinds {
        let key = kind.keys().1.unwrap_or_else(|| kind.keys().0);
        let reduction = formulas::damage_reduction(&stats.percents, kind);
        derived.insert(key.to_string(), reduction * 100.0);
        derived.insert(
            format!("{} EHP", key.replace(" Resistance", "")),
            formulas::effective_health(health, reduction * (1.0 - faced_pen)),
        );
    }

    derived
}

#[must_use]
/// Returns an unstructured diff (prob make structured later) of two pocamind/data versions
pub fn changed_items(old: &DeepData, new: &DeepData) -> Vec<String> {
    let mut changed = Vec::new();

    for talent in new.talents() {
        match old.get_talent(&talent.name) {
            Some(prev)
                if prev.contributions == talent.contributions => {}
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
        let result = aggregate(&data, &BuildParams::default(), Scenario::default());

        let flat = result.flat_totals();
        assert_eq!(flat.get("Health"), Some(&220.0));
        assert_eq!(flat.get("Ether"), Some(&120.0));
        assert_eq!(result.derived.get("Physical EHP"), Some(&220.0));
    }

    #[test]
    fn fortitude_health_curve_halves_past_50() {
        let data = DeepData::default();
        let mut build = BuildParams::default();
        build.stats.insert(Stat::Fortitude, 80);

        let result = aggregate(&data, &build, Scenario::default());
        let fortitude_health = result.flat["Health"]
            .iter()
            .find(|e| e.source == "80 Fortitude")
            .unwrap();
        assert_eq!(fortitude_health.value, 25.0 + 30.0 * 0.25);
    }

    #[test]
    fn aliases_merge_into_canonical_name() {
        let data = DeepData::default();
        let result = aggregate(&data, &BuildParams::default(), Scenario::default());
        assert!(!result.percents.contains_key("Physical Armor"));
    }

    #[test]
    fn sources_coalesce_and_sort_descending() {
        let data = DeepData::default();
        let mut build = BuildParams::default();
        build.stats.insert(Stat::Willpower, 40);
        build.traits.insert("Erudition".to_string(), 2);

        let result = aggregate(&data, &build, Scenario::default());
        let tempo = &result.flat["Tempo"];
        assert_eq!(tempo[0].source, "Starting");
        assert!(tempo.windows(2).all(|w| w[0].value >= w[1].value));
    }

    #[test]
    fn identical_data_has_no_changed_items() {
        let data = DeepData::default();
        assert!(changed_items(&data, &data).is_empty());
    }

    use crate::model::aggregate::{AggregateMode, EquipmentSelection, WeaponSelection};

    /// Enchant effects now come from the data. Sear grants a flat +5% melee pen always, and
    /// its conditional Cauterize (+40 universal pen) folds into both channels optimistically.
    #[test]
    fn enchant_contributions_come_from_data() {
        let data = DeepData::from_json(
            r#"{"enchants":{"sear":{"name":"Sear","category":"Weapon","info":"",
                "stats":{"Melee Pen":5},"conditional_stats":{"Pen":40}}}}"#,
        )
        .unwrap();
        let mut build = BuildParams::default();
        build.weapon = Some(WeaponSelection { enchant: Some("Sear".to_string()), ..Default::default() });

        let base = aggregate(&data, &build, Scenario::default()).percent_totals();
        assert!((base.get("Melee Pen").copied().unwrap_or(0.0) - 5.0).abs() < 1e-9);
        assert!(base.get("Mantra Pen").copied().unwrap_or(0.0).abs() < 1e-9);

        let opt = aggregate(&data, &build, Scenario { mode: AggregateMode::Optimistic, ..Default::default() })
            .percent_totals();
        assert!((opt["Melee Pen"] - 45.0).abs() < 1e-9);
        assert!((opt["Mantra Pen"] - 40.0).abs() < 1e-9);
    }

    /// Heroism's damage buff is a combat-state expression, so PVP and PVE differ.
    #[test]
    fn heroism_damage_branches_on_combat_state() {
        let data = DeepData::from_json(
            r#"{"enchants":{"heroism":{"name":"Heroism","category":"Weapon","info":"",
                "conditional_stats":{"Damage":"if(PVP, 20, 5)"}}}}"#,
        )
        .unwrap();
        let mut build = BuildParams::default();
        build.weapon = Some(WeaponSelection { enchant: Some("Heroism".to_string()), ..Default::default() });

        let dmg = |state| {
            aggregate(&data, &build, Scenario { mode: AggregateMode::Optimistic, combat_state: state, ..Default::default() })
                .percent_totals()
                .get("Damage")
                .copied()
                .unwrap_or(0.0)
        };
        assert!((dmg(CombatState::Pvp) - 20.0).abs() < 1e-9);
        assert!((dmg(CombatState::Pve) - 5.0).abs() < 1e-9);
    }

    /// enemy_pen erodes EHP, but only after the build's Pen Resistance blunts it. Both
    /// default to no effect.
    #[test]
    fn enemy_pen_erodes_ehp_after_pen_resistance() {
        let data = DeepData::from_json(
            r#"{"enchants":{"plate":{"name":"Plate","category":"Equipment","info":"",
                "stats":{"Physical Resistance":50,"Pen Resistance":40}}}}"#,
        )
        .unwrap();
        let mut build = BuildParams::default();
        build.equipment = vec![EquipmentSelection { enchant: Some("Plate".to_string()), ..Default::default() }];

        // 220 base health, 50% reduction, no enemy pen: 220 / 0.5 = 440.
        let unopposed = aggregate(&data, &build, Scenario::default());
        assert!((unopposed.derived["Physical EHP"] - 440.0).abs() < 1e-6);

        // 50% enemy pen against 40% pen resist gives faced pen 0.3, effective reduction 0.35.
        let vs_pen = aggregate(&data, &build, Scenario { enemy_pen: 50.0, ..Default::default() });
        assert!((vs_pen.derived["Physical EHP"] - 220.0 / 0.65).abs() < 1e-6);
    }
}
