//! Types describing a build's inputs and its aggregated output.
//!
//! The aggregation itself lives in `util::aggregate`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::formulas::CombatState;
use crate::util::statmap::StatMap;

/// Where a contribution came from.
///
/// We need this for resistances, since equipment resistances sum together into a single factor,
/// but every other source is its own multiplicative factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatOrigin {
    Base,
    Talent,
    Mantra,
    Equipment,
    Outfit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatSource {
    pub value: f64,
    pub source: String,
    #[serde(default = "base_origin")]
    pub origin: StatOrigin,
    /// Most likely a pre-formatted string for the UI, e.g. `+15%`, `×10%`, or a custom label
    #[serde(default)]
    pub display_value: String,
}

fn base_origin() -> StatOrigin {
    StatOrigin::Base
}

/// A damage type someone can be resistant to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Blunt,
    Slash,
    Flame,
    Ice,
    Thunder,
    Wind,
    Shadow,
    Metal,
    Blood,
}

/// The broad category a damage type belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageGroup {
    Physical,
    Elemental,
}

/// What an incoming hit is being resisted as: a whole group, or one specific type.
///
/// A group on its own is what the game reports for "Physical" or "Elemental". A specific
/// type additionally picks up that type's own resistance, so it is never lower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageKind {
    Group(DamageGroup),
    Type(DamageType),
}

impl From<DamageGroup> for DamageKind {
    fn from(group: DamageGroup) -> Self {
        DamageKind::Group(group)
    }
}

impl From<DamageType> for DamageKind {
    fn from(damage_type: DamageType) -> Self {
        DamageKind::Type(damage_type)
    }
}

impl DamageKind {
    /// The broad key, then the type-specific key when there is one.
    #[must_use]
    pub fn keys(self) -> (&'static str, Option<&'static str>) {
        match self {
            DamageKind::Group(group) => (group.key(), None),
            DamageKind::Type(t) => (t.group_key(), Some(t.subtype_key())),
        }
    }
}

impl DamageGroup {
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            DamageGroup::Physical => "Physical Resistance",
            DamageGroup::Elemental => "Elemental Resistance",
        }
    }

    #[must_use]
    pub fn types(self) -> &'static [DamageType] {
        match self {
            DamageGroup::Physical => DamageType::PHYSICAL,
            DamageGroup::Elemental => DamageType::ELEMENTAL,
        }
    }
}

impl DamageType {
    pub const PHYSICAL: &'static [DamageType] = &[DamageType::Blunt, DamageType::Slash];
    pub const ELEMENTAL: &'static [DamageType] = &[
        DamageType::Flame,
        DamageType::Ice,
        DamageType::Thunder,
        DamageType::Wind,
        DamageType::Shadow,
        DamageType::Metal,
        DamageType::Blood,
    ];

    pub const ALL: &'static [DamageType] = &[
        DamageType::Blunt,
        DamageType::Slash,
        DamageType::Flame,
        DamageType::Ice,
        DamageType::Thunder,
        DamageType::Wind,
        DamageType::Shadow,
        DamageType::Metal,
        DamageType::Blood,
    ];

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            DamageType::Blunt => "Blunt",
            DamageType::Slash => "Slash",
            DamageType::Flame => "Flame",
            DamageType::Ice => "Ice",
            DamageType::Thunder => "Thunder",
            DamageType::Wind => "Wind",
            DamageType::Shadow => "Shadow",
            DamageType::Metal => "Metal",
            DamageType::Blood => "Blood",
        }
    }

    #[must_use]
    pub fn group(self) -> DamageGroup {
        if Self::PHYSICAL.contains(&self) {
            DamageGroup::Physical
        } else {
            DamageGroup::Elemental
        }
    }

    /// The broad resistance key covering this damage type
    #[must_use]
    pub fn group_key(self) -> &'static str {
        self.group().key()
    }

    /// The resistance key specific to this damage type
    #[must_use]
    pub fn subtype_key(self) -> &'static str {
        match self {
            DamageType::Blunt => "Blunt Resistance",
            DamageType::Slash => "Slash Resistance",
            DamageType::Flame => "Flame Resistance",
            DamageType::Ice => "Ice Resistance",
            DamageType::Thunder => "Thunder Resistance",
            DamageType::Wind => "Wind Resistance",
            DamageType::Shadow => "Shadow Resistance",
            DamageType::Metal => "Metal Resistance",
            DamageType::Blood => "Blood Resistance",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EquipmentSelection {
    pub name: String,
    /// Map of pip rarity -> chosen pips.
    pub pips: HashMap<String, Vec<String>>,
    /// Quality stars from 0 to 3. Head, arms and legs gain 1 Health per star
    pub stars: u8,
    pub enchant: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponSelection {
    pub name: String,
    /// 0 to 3
    pub stars: u8,
    /// `DMG%`, `PEN%` or `WGT%`, or empty when the weapon has no buff
    #[serde(rename = "starBuff")]
    pub star_buff: String,
    pub enchant: Option<String>,
}

impl WeaponSelection {
    #[must_use]
    pub fn star_mod(&self) -> Option<StarMod> {
        match self.star_buff.as_str() {
            "DMG%" => Some(StarMod::Damage),
            "PEN%" => Some(StarMod::Penetration),
            "WGT%" => Some(StarMod::Weight),
            _ => None,
        }
    }

    /// Quality star bonus as a fraction, 0 when there is no star buff
    #[must_use]
    pub fn star_bonus(&self) -> f64 {
        self.star_mod().map_or(0.0, |m| m.bonus(self.stars))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarMod {
    #[serde(rename = "DMG%")]
    Damage,
    #[serde(rename = "PEN%")]
    Penetration,
    #[serde(rename = "WGT%")]
    Weight,
}

impl StarMod {
    /// - Damage multiplies base damage
    /// - Pen adds to Melee pen
    /// - Weight raises posture damage and max posture
    #[must_use]
    pub fn bonus(self, stars: u8) -> f64 {
        let table: [f64; 3] = match self {
            StarMod::Damage => [0.02, 0.04, 0.06],
            StarMod::Penetration => [0.05, 0.10, 0.15],
            StarMod::Weight => [0.04, 0.08, 0.12],
        };
        match stars {
            1..=3 => table[stars as usize - 1],
            _ => 0.0,
        }
    }
}

/// `Base` is what the build always has. 
/// `Optimistic` additionally counts every conditional talent at its maximum,
/// (e.g. when a buff is active)\, which acts like a ceiling
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateMode {
    #[default]
    Base,
    Optimistic,
}

impl AggregateMode {
    #[must_use]
    pub fn is_optimistic(self) -> bool {
        self == AggregateMode::Optimistic
    }
}

/// The conditions a build is evaluated under: how thorough the count is, which combat state
/// fills the `PVP` / `PVE` expression variables, and the assumed enemy the EHP and DPS
/// readouts face.
///
/// Everything defaults to a plain PvP build with no assumed enemy. `enemy_pen` and
/// `enemy_resistance` are percents, and at 0 they leave EHP and DPS at their unopposed
/// values, so a caller opts in by setting them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Scenario {
    pub mode: AggregateMode,
    pub combat_state: CombatState,
    /// The attacker's penetration the EHP readout faces, as a percent.
    pub enemy_pen: f64,
    /// The target's damage reduction the DPS readout faces, as a percent.
    pub enemy_resistance: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
/// All the information needed to derive total stats of the build
pub struct BuildParams {
    pub stats: StatMap,
    pub race: String,
    pub talents: Vec<String>,
    pub boons: Vec<String>,
    pub traits: HashMap<String, i64>,
    pub equipment: Vec<EquipmentSelection>,
    pub outfit: Option<String>,
    pub weapon: Option<WeaponSelection>,
    pub mantras: Vec<MantraSelection>,
}

/// A mantra the build holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MantraSelection {
    pub name: String,
    /// 1 to 5.
    pub level: i64,
    pub gem: Option<String>,
    pub sparks: Vec<String>,
    pub modifiers: HashMap<String, i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildTotalStats {
    pub flat: HashMap<String, Vec<StatSource>>,
    pub percents: HashMap<String, Vec<StatSource>>,
    pub derived: HashMap<String, f64>,
}

impl BuildTotalStats {
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
