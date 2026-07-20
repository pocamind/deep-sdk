//! Implementations of Deepwoken's formulas.
//!
//! Each is a free function taking exactly what it depends on, so the inputs to a piece
//! of math are visible in its signature. See `docs/formulas.md` in the dwt repo for the
//! derivations and which of them the wiki states outright.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::constants::{
    DAMAGE_CAPS_OUT_OF_COMBAT, DAMAGE_CAPS_PVE, DAMAGE_CAPS_PVP, INNATE_BLEED_RATE,
    KHAN_REQ_REDUCTION, MAX_SINGLE_RESIST, PROFICIENCY_PER_POINT, REQUIREMENT_PENALTY, RING_FACTOR,
    SCALING_DIVISOR, SCALING_FACTOR, SILENTHEART, SILENTHEART_REQ_REDUCTION, TRAIT_CAP,
};
use crate::model::aggregate::{BuildParams, DamageKind, StarMod, StatOrigin, StatSource};
use crate::model::data::DeepData;
use crate::model::req::{Atom, ClauseType, Requirement};
use crate::model::stat;
use crate::util::statmap::StatMap;

/// Resistance from any one source is clamped to 99% before it is applied
fn clamp_resist(percent: f64) -> f64 {
    percent.clamp(0.0, MAX_SINGLE_RESIST) / 100.0
}

/// Damage reduction against a damage group or a specific type, as a fraction from 0 to 1.
///
/// Equipment resistances sum into a single factor, while every other source is its own factor,
/// then the factors multiply (as per the wiki).
///
/// # Arguments
///
/// * `percents` - the aggregated percentage contributions, keyed by stat name
/// * `kind` - a [`DamageGroup`] for a hit with no subtype, or a [`DamageType`] for one
///   that also picks up that type's own resistance
///
/// # Formula
///
/// ```text
/// 1 - (1 - outfit) * (1 - outfitSubtype) * (1 - Σequipment) * (1 - talent) * ...
/// ```
///
/// [`DamageGroup`]: crate::model::aggregate::DamageGroup
/// [`DamageType`]: crate::model::aggregate::DamageType
#[must_use]
pub fn damage_reduction(
    percents: &HashMap<String, Vec<StatSource>>,
    kind: impl Into<DamageKind>,
) -> f64 {
    let (group, subtype) = kind.into().keys();
    let mut equipment = 0.0;
    let mut product = 1.0;

    for key in [Some(group), subtype].into_iter().flatten() {
        let Some(entries) = percents.get(key) else {
            continue;
        };
        for entry in entries {
            if entry.origin == StatOrigin::Equipment {
                equipment += entry.value;
            } else {
                product *= 1.0 - clamp_resist(entry.value);
            }
        }
    }

    product *= 1.0 - clamp_resist(equipment);
    1.0 - product
}

/// How much raw damage you can absorb before dropping.
///
/// # Arguments
///
/// * `health` - maximum health
/// * `reduction` - damage reduction as a fraction from 0 to 1
///
/// # Formula
///
/// ```text
/// health / max(1 - reduction, 0.01)
/// ```
///
/// The denominator is floored so a build at or above 99% reduction reports a finite
/// number rather than dividing by zero.
#[must_use]
pub fn effective_health(health: f64, reduction: f64) -> f64 {
    health / (1.0 - reduction).max(0.01)
}

/* ================= DAMAGE FORMULAS ================= */


/// Which damage modifier caps apply.
///
/// Entering any combat drops the soft cap, but only player combat has the hard cap.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatState {
    OutOfCombat,
    Pve,
    #[default]
    Pvp,
}

impl CombatState {
    /// `(soft, hard)` as fractions
    #[must_use]
    pub fn damage_caps(self) -> (f64, f64) {
        match self {
            CombatState::OutOfCombat => DAMAGE_CAPS_OUT_OF_COMBAT,
            CombatState::Pve => DAMAGE_CAPS_PVE,
            CombatState::Pvp => DAMAGE_CAPS_PVP,
        }
    }
}

/// Weapon damage after stat scaling, prior to any damage modifiers.
///
/// # Arguments
///
/// * `base` - the weapon's damage with any `DMG%` quality stars already applied
/// * `scaling` - each stat's invested value paired with the weapon's coefficient for it
/// * `rings` - investments backing equipped scaling rings, in any order
/// * `proficiency` - the Proficiency trait, 0 to 6
///
/// # Formula
///
/// ```text
/// base * (1 + (0.75 * Σ(stat * coeff) / 1000 + Σrings) * (1 + 0.065 * proficiency))
///
/// where the rank-k ring contributes  1.2 * investment / (2^(k-1) * 1000)
/// ```
///
/// Rings are ranked by investment descending and sit inside the Proficiency multiplier,
/// so a weapon with no scaling stat still benefits from Proficiency through them.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "at most 4 rings")]
pub fn scaled_damage(base: f64, scaling: &[(f64, f64)], rings: &[f64], proficiency: i64) -> f64 {
    let stat_term: f64 = scaling.iter().map(|(value, coeff)| value * coeff).sum();
    let stat_fraction = SCALING_FACTOR * stat_term / SCALING_DIVISOR;

    let mut ranked = rings.to_vec();
    ranked.sort_by(|a, b| b.total_cmp(a));
    let ring_fraction: f64 = ranked
        .iter()
        .enumerate()
        .map(|(i, investment)| RING_FACTOR * investment / (2f64.powi(i as i32) * SCALING_DIVISOR))
        .sum();

    let proficiency_mult = 1.0 + proficiency as f64 * PROFICIENCY_PER_POINT;
    base * (1.0 + (stat_fraction + ring_fraction) * proficiency_mult)
}

/// The damage modifier actually applied, after the soft and hard caps.
///
/// # Arguments
///
/// * `raw` - the sum of every percentage bonus, as a fraction
/// * `state` - which cap pair applies, see [`CombatState`]
///
/// # Formula
///
/// ```text
/// min(hard,  raw <= soft ? raw : soft + (raw - soft) / 2)
/// ```
///
/// Past the soft cap each further point counts as half a point, and the hard cap
/// truncates. Bonuses are additive with each other, not multiplicative.
#[must_use]
pub fn damage_modifier(raw: f64, state: CombatState) -> f64 {
    let (soft, hard) = state.damage_caps();
    let softened = if raw > soft { soft + (raw - soft) / 2.0 } else { raw };
    softened.min(hard)
}

/// Bleed damage, a flat share of *scaled* damage.
///
/// # Arguments
///
/// * `scaled` - damage after stat scaling, before percentage modifiers
/// * `bleed_rate` - 0.15 for a weapon that deals Bleed, 0.075 from Speed Demon
///
/// # Formula
///
/// ```text
/// scaled * bleed_rate
/// ```
///
/// Keyed off scaled damage rather than modified damage, and added on top of the modified
/// damage rather than multiplied by it.
#[must_use]
pub fn bleed_damage(scaled: f64, bleed_rate: f64) -> f64 {
    scaled * bleed_rate
}

/// Damage multiplier from failing a weapon's stat requirements.
///
/// # Arguments
///
/// * `worst_ratio` - the smallest `have / needed` across the required stats, counting
///   only stats that fall short. `needed` is the requirement after any Khan or
///   Silentheart reduction
///
/// # Formula
///
/// ```text
/// worst_ratio >= 1  ->  1
/// else              ->  1 - 0.25 * (1 - worst_ratio)
/// ```
///
/// Bottoms out at 0.75, and is multiplicative against every other damage multiplier.
#[must_use]
pub fn requirement_debuff(worst_ratio: f64) -> f64 {
    if worst_ratio >= 1.0 {
        1.0
    } else {
        1.0 - REQUIREMENT_PENALTY * (1.0 - worst_ratio.max(0.0))
    }
}

/// Damage surviving the target's resistance.
///
/// # Arguments
///
/// * `damage` - incoming damage before resistance
/// * `reduction` - the target's damage reduction, a fraction from 0 to 1
/// * `penetration` - the attacker's penetration, a fraction from 0 to 1
///
/// # Formula
///
/// ```text
/// damage * (1 - reduction * (1 - penetration))
/// ```
///
/// Penetration erodes the resistance rather than adding damage, so it is worth more the
/// more resistance the target has.
#[must_use]
pub fn damage_after_resistance(damage: f64, reduction: f64, penetration: f64) -> f64 {
    damage * (1.0 - reduction.clamp(0.0, 1.0) * (1.0 - penetration.clamp(0.0, 1.0)))
}

/// Seconds per swing, or `None` when the weapon publishes no usable timing.
///
/// # Arguments
///
/// * `attack_duration` - the weapon's attack duration in seconds, if it has one
/// * `swing_speed` - a relative multiplier such as `1.1`, used only as a fallback
/// * `endlag` - recovery time in seconds, added to the swing speed fallback
///
/// # Formula
///
/// ```text
/// attack_duration > 0  ->  attack_duration
/// swing_speed > 0      ->  1 / swing_speed + endlag
/// else                 ->  None
/// ```
///
/// 230 of 266 weapons publish an attack duration, 22 fall back to swing speed, and 14
/// have neither. Those last return `None` rather than 0, since "no rate" is not "no
/// damage".
#[must_use]
pub fn attack_cycle(
    attack_duration: Option<f64>,
    swing_speed: Option<f64>,
    endlag: f64,
) -> Option<f64> {
    match (attack_duration, swing_speed) {
        (Some(duration), _) if duration > 0.0 => Some(duration),
        (_, Some(speed)) if speed > 0.0 => Some(1.0 / speed + endlag),
        _ => None,
    }
}

/// Damage per second, or `None` when the weapon has no usable attack cycle.
///
/// # Arguments
///
/// * `damage` - damage per swing, after resistance
/// * `cycle` - seconds per swing, from [`attack_cycle`]
///
/// # Formula
///
/// ```text
/// damage / cycle
/// ```
///
/// Note that Deepwoken has no official DPS measurement. This is just a comparison metric, 
/// and it ignores any sensible PVP structure, only measuring M1 damage.
#[must_use]
pub fn dps(damage: f64, cycle: Option<f64>) -> Option<f64> {
    cycle.filter(|c| *c > 0.0).map(|c| damage / c)
}

/// How far a weapon requirement is lowered for this build.
/// 
/// Khan and SH reductions are included (in fact thats the only thing that happens here)
fn requirement_reduction(build: &BuildParams, talents: &[String], atom_stats: &BTreeSet<Stat>) -> i64 {
    let mut reduction = 0;

    if build.race == "Khan" && !atom_stats.contains(&Stat::Total) {
        reduction += KHAN_REQ_REDUCTION;
    }

    let weapon_only = atom_stats.iter().all(|stat| stat::WEAPON.contains(stat));
    if weapon_only && talents.iter().any(|t| t == SILENTHEART) {
        reduction += SILENTHEART_REQ_REDUCTION;
    }

    reduction
}

/// The worst `have / needed` across a weapon's requirement clauses, 1.0 when every clause
/// is met.
///
/// Clauses are ANDed, so the result is the worst clause. Within a clause, an `OR` is met by
/// whichever branch you are closest on.
/// 
/// TODO! I HATE WHAT I NAME THINGS
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
pub fn worst_requirement_ratio(build: &BuildParams, talents: &[String], reqs: &Requirement) -> f64 {
    let ratio = |atom: &Atom| -> f64 {
        let needed = atom.value - requirement_reduction(build, talents, &atom.stats);
        if needed <= 0 {
            return 1.0;
        }
        let have: i64 = atom
            .stats
            .iter()
            .map(|stat| {
                if *stat == Stat::Total {
                    build.stats.cost()
                } else {
                    build.stats.get(stat)
                }
            })
            .sum();
        (have as f64 / needed as f64).min(1.0)
    };

    let mut worst = 1.0_f64;
    for clause in reqs.iter() {
        if clause.is_empty() {
            continue;
        }
        let clause_ratio = match clause.clause_type {
            ClauseType::Or => clause.atoms().iter().map(ratio).fold(0.0_f64, f64::max),
            ClauseType::And => clause.atoms().iter().map(ratio).fold(1.0_f64, f64::min),
        };
        worst = worst.min(clause_ratio);
    }

    worst
}

/// The stat value a weapon scaling term needs.
/// Either a specific attribute, or the max over a category 
/// (Mind, Body, Weapon, Attunement).
#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
fn scaling_value(name: &str, stats: &StatMap) -> Option<f64> {
    if let Ok(stat) = name.parse::<Stat>() {
        return Some(stats.get(&stat) as f64);
    }
    let members = stat::category(name)?;
    members.iter().map(|s| stats.get(s)).max().map(|v| v as f64)
}

/// Damage of one basic attack, and the resulting DPS.
/// Does not take into account enemy resistances.
#[must_use]
#[allow(clippy::cast_precision_loss, reason = "stat values are small")]
pub fn weapon_damage(
    data: &DeepData,
    build: &BuildParams,
    talents: &[String],
    percent: &HashMap<String, f64>,
    combat_state: CombatState,
) -> Option<(f64, Option<f64>)> {
    let selection = build.weapon.as_ref()?;
    let weapon = data.get_weapon(&selection.name)?;

    let mut base = weapon.damage?;
    if selection.star_mod() == Some(StarMod::Damage) {
        base *= 1.0 + selection.star_bonus();
    }

    let scaling: Vec<(f64, f64)> = weapon
        .scaling
        .iter()
        .filter_map(|(name, coeff)| Some((scaling_value(name, &build.stats)?, *coeff)))
        .collect();

    let proficiency = build.traits.get("Proficiency").copied().unwrap_or(0).min(TRAIT_CAP);
    let scaled = scaled_damage(base, &scaling, &[], proficiency);

    let raw_modifier = percent.get("Damage").copied().unwrap_or(0.0) / 100.0;
    let modifier = damage_modifier(raw_modifier, combat_state);

    let innate_bleed = weapon
        .damage_types
        .iter()
        .any(|t| t == "Bleed")
        .then_some(INNATE_BLEED_RATE)
        .unwrap_or(0.0);
    let bleed_rate = (percent.get("Bleed").copied().unwrap_or(0.0) / 100.0).max(innate_bleed);
    let bleed = bleed_damage(scaled, bleed_rate);

    let debuff = requirement_debuff(worst_requirement_ratio(build, talents, &weapon.reqs));
    let m1 = (scaled * (1.0 + modifier) + bleed) * debuff;

    let cycle = attack_cycle(
        weapon.attack_duration,
        weapon.swing_speed,
        weapon.endlag.unwrap_or(0.0),
    );

    Some((m1, dps(m1, cycle)))
}

// claude wrote all these tests against wiki info i sure do hope nothing is wrong
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::aggregate::{DamageGroup, DamageType};

    fn percents(entries: &[(&str, f64, StatOrigin)]) -> HashMap<String, Vec<StatSource>> {
        let mut map: HashMap<String, Vec<StatSource>> = HashMap::new();
        for (key, value, origin) in entries {
            map.entry((*key).to_string()).or_default().push(StatSource {
                value: *value,
                source: String::new(),
                origin: *origin,
            });
        }
        map
    }

    /// Wiki `Character Stats` example 2, verbatim:
    /// `1 - ((1 - 30%) x (1 - 10%) x (1 - (10 + 16 + 3)%) x (1 - 3%) x (1 - 15%)) = 0.6312`
    #[test]
    fn matches_wiki_worked_example() {
        let p = percents(&[
            ("Physical Resistance", 30.0, StatOrigin::Outfit),
            ("Blunt Resistance", 10.0, StatOrigin::Outfit),
            ("Physical Resistance", 10.0, StatOrigin::Equipment),
            ("Physical Resistance", 16.0, StatOrigin::Equipment),
            ("Physical Resistance", 3.0, StatOrigin::Equipment),
            ("Blunt Resistance", 3.0, StatOrigin::Talent),
            ("Blunt Resistance", 15.0, StatOrigin::Talent),
        ]);

        let dr = damage_reduction(&p, DamageType::Blunt);
        assert!((dr - 0.6312).abs() < 1e-4, "got {dr}");
    }

    #[test]
    fn equipment_sums_before_multiplying() {
        let equip = percents(&[
            ("Physical Resistance", 10.0, StatOrigin::Equipment),
            ("Physical Resistance", 10.0, StatOrigin::Equipment),
        ]);
        assert!((damage_reduction(&equip, DamageType::Blunt) - 0.20).abs() < 1e-9);

        let talents = percents(&[
            ("Physical Resistance", 10.0, StatOrigin::Talent),
            ("Physical Resistance", 10.0, StatOrigin::Talent),
        ]);
        assert!((damage_reduction(&talents, DamageType::Blunt) - 0.19).abs() < 1e-9);
    }

    /// Wiki `Legion Centurion`: +30% Elemental with +5% Wind gives "30% Elemental" and
    /// "33.5% Galebreathe". The group value is the plain 30%, not a mean over types.
    #[test]
    fn group_is_not_the_mean_over_types() {
        let p = percents(&[
            ("Elemental Resistance", 30.0, StatOrigin::Outfit),
            ("Wind Resistance", 5.0, StatOrigin::Outfit),
        ]);

        assert!((damage_reduction(&p, DamageGroup::Elemental) - 0.30).abs() < 1e-9);
        assert!((damage_reduction(&p, DamageType::Wind) - 0.335).abs() < 1e-9);
        assert!((damage_reduction(&p, DamageType::Flame) - 0.30).abs() < 1e-9);
    }

    #[test]
    fn effective_health_floors_the_denominator() {
        assert!((effective_health(220.0, 0.0) - 220.0).abs() < 1e-9);
        assert!((effective_health(220.0, 0.5) - 440.0).abs() < 1e-9);
        assert!((effective_health(220.0, 1.0) - 22000.0).abs() < 1e-9);
    }

    /// Rimebreakers: 16.5 base, Light Weapon 65 at 5.00, Frostdraw 50 at 3.50 and
    /// Galebreathe 78 at 3.50, no Proficiency. `0.75 * 773 / 1000 = 0.57975`.
    #[test]
    fn scaled_damage_matches_rimebreakers() {
        let scaling = [(65.0, 5.0), (50.0, 3.5), (78.0, 3.5)];
        let scaled = scaled_damage(16.5, &scaling, &[], 0);
        assert!((scaled - 26.065_875).abs() < 1e-6, "got {scaled}");
    }

    /// The wiki ranks rings by investment, so rank 1 divides by 1000, rank 2 by 2000.
    /// Passing them out of order must not change the answer.
    #[test]
    fn scaling_rings_rank_by_investment() {
        let ordered = scaled_damage(100.0, &[], &[100.0, 50.0], 0);
        let shuffled = scaled_damage(100.0, &[], &[50.0, 100.0], 0);
        assert!((ordered - shuffled).abs() < 1e-9);

        let expected = 100.0 * (1.0 + (1.2 * 100.0 / 1000.0 + 1.2 * 50.0 / 2000.0));
        assert!((ordered - expected).abs() < 1e-9);
    }

    /// Proficiency multiplies the scaling term only, so it does nothing without scaling.
    #[test]
    fn proficiency_multiplies_only_the_scaling_term() {
        assert!((scaled_damage(20.0, &[], &[], 6) - 20.0).abs() < 1e-9);

        let with = scaled_damage(20.0, &[(100.0, 4.0)], &[], 6);
        let expected = 20.0 * (1.0 + 0.75 * 400.0 / 1000.0 * (1.0 + 6.0 * 0.065));
        assert!((with - expected).abs() < 1e-9);
    }

    #[test]
    fn damage_modifier_halves_past_the_soft_cap() {
        assert!((damage_modifier(0.20, CombatState::Pvp) - 0.20).abs() < 1e-9);
        assert!((damage_modifier(0.44, CombatState::Pvp) - 0.345).abs() < 1e-9);
        assert!((damage_modifier(2.00, CombatState::Pvp) - 0.50).abs() < 1e-9);
        assert!((damage_modifier(0.44, CombatState::OutOfCombat) - 0.44).abs() < 1e-9);
        assert!((damage_modifier(2.00, CombatState::Pve) - 0.75).abs() < 1e-9);
    }

    #[test]
    fn requirement_debuff_bottoms_out_at_three_quarters() {
        assert!((requirement_debuff(1.0) - 1.0).abs() < 1e-9);
        assert!((requirement_debuff(0.0) - 0.75).abs() < 1e-9);
        assert!((requirement_debuff(0.85) - 0.9625).abs() < 1e-9);
    }

    /// A category req like Mind is an `OR` of its stats and resolves to the max, so meeting
    /// one branch satisfies the clause and the shortfall is measured against the best branch.
    #[test]
    fn or_clause_takes_the_best_branch() {
        let reqs = Requirement::parse("50r INT OR 50r WLL OR 50r CHA").unwrap();

        let mut met = BuildParams::default();
        met.stats.insert(Stat::Intelligence, 50);
        assert!((worst_requirement_ratio(&met, &[], &reqs) - 1.0).abs() < 1e-9);

        let mut partial = BuildParams::default();
        partial.stats.insert(Stat::Intelligence, 40);
        assert!((worst_requirement_ratio(&partial, &[], &reqs) - 0.8).abs() < 1e-9);
    }

    /// Silentheart drops Heavy/Medium/Light Weapon requirements by 25 but never core
    /// attributes, and the reduced value is what the debuff ratio is measured against.
    #[test]
    fn silentheart_lowers_weapon_stat_reqs_only() {
        let reqs = Requirement::parse("55r HVY, 20r STR").unwrap();
        let oath = [SILENTHEART.to_string()];

        let mut met = BuildParams::default();
        met.stats.insert(Stat::HeavyWeapon, 30);
        met.stats.insert(Stat::Strength, 20);
        assert!((worst_requirement_ratio(&met, &oath, &reqs) - 1.0).abs() < 1e-9);

        let mut short_str = BuildParams::default();
        short_str.stats.insert(Stat::HeavyWeapon, 30);
        short_str.stats.insert(Stat::Strength, 10);
        assert!((worst_requirement_ratio(&short_str, &oath, &reqs) - 0.5).abs() < 1e-9);
    }

    /// A Total requirement measures against the build's cost, not a stored stat, mirroring
    /// Atom::satisfied_by. Without that mapping the have-sum reads 0 and pins the debuff.
    #[test]
    fn total_requirement_reads_build_cost() {
        use crate::model::req::Clause;
        let mut reqs = Requirement::new();
        reqs.add_clause(Clause::and().atom(Atom::strict().value(150).stat(Stat::Total)));

        let mut build = BuildParams::default();
        build.stats.insert(Stat::Strength, 150);

        assert!((worst_requirement_ratio(&build, &[], &reqs) - 1.0).abs() < 1e-9);
    }

    /// A category scaling term (Mind, Body, ...) resolves to the max over its stats, and a
    /// specific stat resolves directly. Unknown names are dropped.
    #[test]
    fn scaling_resolves_categories_as_max() {
        let mut stats = StatMap::new();
        stats.insert(Stat::Intelligence, 50);
        stats.insert(Stat::Willpower, 30);
        stats.insert(Stat::Charisma, 10);
        stats.insert(Stat::HeavyWeapon, 40);

        assert_eq!(scaling_value("Mind", &stats), Some(50.0));
        assert_eq!(scaling_value("Heavy Weapon", &stats), Some(40.0));
        assert_eq!(scaling_value("Body", &stats), Some(0.0));
        assert_eq!(scaling_value("Nonsense", &stats), None);
    }

    /// Wiki `Character Stats` example 4: a 30% attack with 35% PEN against 25.87% armor.
    #[test]
    fn penetration_erodes_resistance() {
        let taken = damage_after_resistance(100.0, 0.30, 0.0);
        assert!((taken - 70.0).abs() < 1e-9);

        let pierced = damage_after_resistance(100.0, 0.30, 0.35);
        assert!((pierced - 80.5).abs() < 1e-9);

        let full = damage_after_resistance(100.0, 0.30, 1.0);
        assert!((full - 100.0).abs() < 1e-9);
    }

    /// Rimebreakers publishes `attack duration 0.5s` alongside `swing speed 1.1x`. The
    /// duration wins.
    #[test]
    fn attack_duration_beats_swing_speed() {
        assert_eq!(attack_cycle(Some(0.5), Some(1.1), 0.0), Some(0.5));
        assert_eq!(attack_cycle(None, Some(1.25), 0.2), Some(1.0 / 1.25 + 0.2));
        assert_eq!(attack_cycle(None, None, 0.0), None);
        assert_eq!(dps(27.0, None), None);
        assert!((dps(27.0, Some(0.5)).unwrap() - 54.0).abs() < 1e-9);
    }
}
