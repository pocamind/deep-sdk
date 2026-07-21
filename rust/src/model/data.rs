// Types that wrap the structures found in pocamind/data

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::error::{DeepError, Result};
use crate::model::enums::{EquipmentSlot, ItemRarity, MantraType, RangeType, TalentRarity, WeaponType};
use crate::model::formula::{StatContributions, StatFormula};
use crate::model::req::{PrereqGroup, Requirement};
use crate::util::graph::PrereqGraph;
use crate::util::name_to_identifier;

fn build_requirement(
    namespace: &str,
    key: &str,
    reqs: &Requirement,
    prereqs: &[PrereqGroup],
) -> Requirement {
    let mut req = Requirement::new();
    req.name = Some(format!("{namespace}:{key}"));
    req.clauses = reqs.clauses.clone();
    req.prereqs = prereqs.iter().cloned().collect();
    req
}

fn reqless_requirement(qualified_id: &str) -> Requirement {
    let mut req = Requirement::new();
    req.name = Some(qualified_id.to_string());
    req
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AspectVariantInfo {
    name: String,
    unlock: Option<String>,
    /// All colors are in hexadecimal format #RRGGBB
    colors: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Aspect {
    pub name: String,
    pub desc: String,
    pub innate: HashMap<Stat, i64>,
    pub is_pathfinder: bool,
    pub variants: HashMap<String, AspectVariantInfo>,
    #[serde(default)]
    pub talent: Vec<String>,
    #[serde(default)]
    pub exclude_cosmetics: Vec<String>,
}

impl Aspect {
    pub const NAMESPACE: &'static str = "aspect";
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatValue {
    pub value: StatFormula,
    pub percentage: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outfit {
    pub name: String,
    #[serde(default)]
    pub pants_id: Option<String>,
    #[serde(default)]
    pub shirt_id: Option<String>,
    pub category: String,
    pub durability: i64,
    pub resistances: HashMap<String, f64>,
    pub extra_percents: HashMap<String, i64>,
    pub talent: Option<String>,
    #[serde(default)]
    pub variants: Vec<String>,
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
    pub mats: HashMap<String, i64>,
    pub notes: i64,
    #[serde(default)]
    pub voi: bool,
    #[serde(default)]
    pub voi_only: bool,
    pub desc: String,
}

impl Outfit {
    pub const NAMESPACE: &'static str = "outfit";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub equippable: bool,
    #[serde(rename = "type")]
    pub equipment_type: EquipmentSlot,
    pub rarity: ItemRarity,
    pub set: Option<String>,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub talents: Vec<String>,
    #[serde(default)]
    pub innates: HashMap<String, StatValue>,
    #[serde(default)]
    pub pips: HashMap<String, i64>,
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
    pub voi: bool,
    #[serde(default)]
    pub voi_only: bool,
    pub desc: String,
}

impl Equipment {
    pub const NAMESPACE: &'static str = "equipment";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Talent {
    pub name: String,
    pub desc: String,
    pub rarity: TalentRarity,
    pub category: String,
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
    pub count_towards_talent_total: bool,
    pub vaulted: bool,
    pub voi: bool,
    #[serde(default)]
    pub voi_only: bool,
    /// Whether this talent is implicitly granted as a byproduct of meeting its
    /// stat requirements (e.g. attunement milestones like Adept/Master), rather
    /// than chosen. Absent in the data unless true.
    #[serde(default)]
    pub implicit: bool,
    #[serde(default)]
    pub exclusive: Vec<String>,
    #[serde(flatten)]
    pub contributions: StatContributions,
    #[serde(default)]
    pub additional_info: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub roll2able: Option<bool>,
}

impl Talent {
    pub const NAMESPACE: &'static str = "talent";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    #[serde(rename = "type")]
    pub weapon_type: WeaponType,
    pub rarity: ItemRarity,
    pub damage: Option<f64>,
    pub posture_damage: Option<f64>,
    pub range: Option<f64>,
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
    pub enchantable: bool,
    pub equip_motifs: bool,
    pub voi: bool,
    #[serde(default)]
    pub voi_only: bool,
    pub desc: String,
    #[serde(default)]
    pub damage_types: Vec<String>,
    #[serde(default)]
    pub range_type: Option<RangeType>,
    #[serde(default)]
    pub attack_duration: Option<f64>,
    #[serde(default)]
    pub endlag: Option<f64>,
    #[serde(default)]
    pub swing_speed: Option<f64>,
    #[serde(default)]
    pub scaling: HashMap<String, f64>,
    #[serde(default)]
    pub bleed_damage: Option<f64>,
    #[serde(default)]
    pub chip_damage: Option<f64>,
    #[serde(default)]
    pub penetration: Option<f64>,
    #[serde(default)]
    pub posture_max: Option<f64>,
    #[serde(default)]
    pub posture_restoration: Option<f64>,
    #[serde(default)]
    pub talents: Vec<String>,
}

impl Weapon {
    pub const NAMESPACE: &'static str = "weapon";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MantraDamageLevel {
    pub level: String,
    pub damage: f64,
    pub posture_damage: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MantraDamageVariant {
    pub variant: Option<String>,
    pub levels: Vec<MantraDamageLevel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mantra {
    pub name: String,
    pub desc: String,
    pub stars: i64,
    pub category: String,
    #[serde(rename = "type")]
    pub mantra_type: MantraType,
    pub attributes: Vec<String>,
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
    pub vaulted: bool,
    pub voi: bool,
    #[serde(default)]
    pub voi_only: bool,
    #[serde(default)]
    pub damage: Vec<MantraDamageVariant>,
    #[serde(default)]
    pub scaling: HashMap<String, f64>,
    #[serde(flatten)]
    pub contributions: StatContributions,
    #[serde(default)]
    pub modifiers: Vec<String>,
    #[serde(default)]
    pub sparks: Vec<String>,
    #[serde(default)]
    pub related_talents: Vec<String>,
    #[serde(default)]
    pub shared_cooldowns: Vec<String>,
    #[serde(default)]
    pub miscellaneous: Option<String>,
}

impl Mantra {
    pub const NAMESPACE: &'static str = "mantra";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enchant {
    pub name: String,
    pub category: String,
    pub info: String,
    #[serde(default)]
    pub in_game_desc: Option<String>,
    #[serde(default)]
    pub obtainable_in: Option<String>,
    #[serde(flatten)]
    pub contributions: StatContributions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub desc: String,
    /// A reqfile segment, i.e. the `Free:` and `Post:` blocks, applied as an
    /// optional reqfile when this preset is selected.
    pub opts: String,
}

impl Enchant {
    pub const NAMESPACE: &'static str = "enchant";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Origin {
    pub name: String,
    pub desc: String,
    pub outfit: String,
    #[serde(default)]
    pub spawns: Vec<String>,
    #[serde(default)]
    pub talents: Vec<String>,
    #[serde(default)]
    pub faction: Option<String>,
}

impl Origin {
    pub const NAMESPACE: &'static str = "origin";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resonance {
    pub name: String,
    pub desc: String,
    pub rarity: String,
}

impl Resonance {
    pub const NAMESPACE: &'static str = "resonance";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Objective {
    pub name: String,
    pub desc: String,
    #[serde(default, rename = "accountWideUnlock")]
    pub account_wide_unlock: bool,
    #[serde(default)]
    pub reqs: Requirement,
    #[serde(default)]
    pub prereqs: Vec<PrereqGroup>,
}

impl Objective {
    pub const NAMESPACE: &'static str = "objective";

    #[must_use]
    pub fn requirement(&self, key: &str) -> Requirement {
        build_requirement(Self::NAMESPACE, key, &self.reqs, &self.prereqs)
    }
}

/// A struct mirroring the structure of the 'all.json'
/// bundle found on [pocamind/data releases](https://github.com/pocamind/data/releases).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeepData {
    aspects: HashMap<String, Aspect>,
    talents: HashMap<String, Talent>,
    mantras: HashMap<String, Mantra>,
    weapons: HashMap<String, Weapon>,
    outfits: HashMap<String, Outfit>,
    equipment: HashMap<String, Equipment>,
    enchants: HashMap<String, Enchant>,
    origins: HashMap<String, Origin>,
    resonances: HashMap<String, Resonance>,
    objectives: HashMap<String, Objective>,
    presets: HashMap<String, Preset>,

    /// The raw json payload used to construct the object, which may be more up-to-date.
    /// The shape is guarenteed to have at least the fields that `DeepData` has.
    #[serde(skip, default)]
    raw: String,
}

impl DeepData {
    pub fn from_json(json: &str) -> Result<DeepData> {
        let mut ret: DeepData = serde_json::from_str(json).map_err(DeepError::from)?;

        ret.raw = json.to_string();
        ret.validate_formulas()?;

        Ok(ret)
    }

    fn validate_formulas(&self) -> Result<()> {
        let named = |item: &str, stat: &str, e: DeepError| {
            DeepError::Formula(format!("{item} / {stat}: {e}"))
        };

        let named_sources = self
            .talents
            .values()
            .map(|t| (&t.name, &t.contributions))
            .chain(self.mantras.values().map(|m| (&m.name, &m.contributions)));

        for (item, contributions) in named_sources {
            for map in contributions.all() {
                for (stat, formula) in map {
                    formula.validate().map_err(|e| named(item, stat, e))?;
                }
            }
        }

        for equip in self.equipment.values() {
            for (stat, innate) in &equip.innates {
                innate
                    .value
                    .validate()
                    .map_err(|e| named(&equip.name, stat, e))?;
            }
        }

        Ok(())
    }

    /// Retrieve Deepwoken data that was bundled with this release. This may be severely out of date and should not be relied on for up-to-date info, prefer DeepData::latest_release + from_release instead.
    #[cfg(feature = "static")]
    pub fn bundled() -> DeepData {
        DeepData::from_json(include_str!("../../assets/all.json"))
            .expect("bundled all.json failed to parse")
    }

    /// Retrieve the raw JSON used to construct the data schema. 
    /// 
    /// We expose this functionality because the data schema may be
    /// frequently updated, though it is a guarentee that the data must be
    /// parsable into the current DeepData structure and it's strongly-typed definitions.
    pub fn raw(&self) -> &String {
        &self.raw
    }

    /// Retrieve a talent by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_talent(&self, name: &str) -> Option<&Talent> {
        self.talents.get(&name_to_identifier(name))
    }

    /// Retrieve a mantra by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_mantra(&self, name: &str) -> Option<&Mantra> {
        self.mantras.get(&name_to_identifier(name))
    }

    /// Retrieve a weapon by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_weapon(&self, name: &str) -> Option<&Weapon> {
        self.weapons.get(&name_to_identifier(name))
    }

    /// Retrieve an outfit by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_outfit(&self, name: &str) -> Option<&Outfit> {
        self.outfits.get(&name_to_identifier(name))
    }

    /// Retrieve an equipment piece by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_equipment(&self, name: &str) -> Option<&Equipment> {
        self.equipment.get(&name_to_identifier(name))
    }

    /// Retrieve an aspect by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_aspect(&self, name: &str) -> Option<&Aspect> {
        self.aspects.get(&name_to_identifier(name))
    }

    /// Retrieve an enchant by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_enchant(&self, name: &str) -> Option<&Enchant> {
        self.enchants.get(&name_to_identifier(name))
    }

    /// Retrieve a preset by it's name.
    ///
    /// The passed in name can be it's in-game name, or the
    /// internal map key
    #[must_use]
    pub fn get_preset(&self, name: &str) -> Option<&Preset> {
        self.presets.get(&name_to_identifier(name))
    }

    #[must_use]
    pub fn get_origin(&self, name: &str) -> Option<&Origin> {
        self.origins.get(&name_to_identifier(name))
    }

    #[must_use]
    pub fn get_resonance(&self, name: &str) -> Option<&Resonance> {
        self.resonances.get(&name_to_identifier(name))
    }

    #[must_use]
    pub fn get_objective(&self, name: &str) -> Option<&Objective> {
        self.objectives.get(&name_to_identifier(name))
    }

    #[must_use]
    pub fn requirement(&self, qualified_id: &str) -> Option<Requirement> {
        let (namespace, key) = qualified_id.split_once(':')?;

        match namespace {
            Talent::NAMESPACE => self.talents.get(key).map(|t| t.requirement(key)),
            Mantra::NAMESPACE => self.mantras.get(key).map(|m| m.requirement(key)),
            Weapon::NAMESPACE => self.weapons.get(key).map(|w| w.requirement(key)),
            Outfit::NAMESPACE => self.outfits.get(key).map(|o| o.requirement(key)),
            Equipment::NAMESPACE => self.equipment.get(key).map(|e| e.requirement(key)),
            Objective::NAMESPACE => self.objectives.get(key).map(|o| o.requirement(key)),
            Aspect::NAMESPACE => self.aspects.get(key).map(|_| reqless_requirement(qualified_id)),
            Origin::NAMESPACE => self.origins.get(key).map(|_| reqless_requirement(qualified_id)),
            Resonance::NAMESPACE => self
                .resonances
                .get(key)
                .map(|_| reqless_requirement(qualified_id)),
            Enchant::NAMESPACE => self
                .enchants
                .get(key)
                .map(|_| reqless_requirement(qualified_id)),
            _ => None,
        }
    }

    #[must_use]
    pub fn implicit_requirements(&self) -> HashMap<String, Requirement> {
        self.talents
            .iter()
            .filter(|(_, talent)| talent.implicit)
            .map(|(key, talent)| (format!("{}:{key}", Talent::NAMESPACE), talent.requirement(key)))
            .collect()
    }

    #[must_use]
    pub fn prereq_graph(&self) -> PrereqGraph {
        let mut graph = PrereqGraph::new();

        for (key, talent) in &self.talents {
            graph.insert(talent.requirement(key));
        }
        for (key, mantra) in &self.mantras {
            graph.insert(mantra.requirement(key));
        }
        for (key, weapon) in &self.weapons {
            graph.insert(weapon.requirement(key));
        }
        for (key, outfit) in &self.outfits {
            graph.insert(outfit.requirement(key));
        }
        for (key, equipment) in &self.equipment {
            graph.insert(equipment.requirement(key));
        }
        for (key, objective) in &self.objectives {
            graph.insert(objective.requirement(key));
        }

        for key in self.aspects.keys() {
            graph.insert_node(format!("{}:{key}", Aspect::NAMESPACE));
        }
        for key in self.origins.keys() {
            graph.insert_node(format!("{}:{key}", Origin::NAMESPACE));
        }
        for key in self.resonances.keys() {
            graph.insert_node(format!("{}:{key}", Resonance::NAMESPACE));
        }
        for key in self.enchants.keys() {
            graph.insert_node(format!("{}:{key}", Enchant::NAMESPACE));
        }

        graph
    }

    /// Retrieve an iterator of talents
    pub fn talents(&self) -> impl Iterator<Item = &Talent> {
        self.talents.values()
    }

    /// Retrieve an iterator of talents
    pub fn mantras(&self) -> impl Iterator<Item = &Mantra> {
        self.mantras.values()
    }

    /// Retrieve an iterator of talents
    pub fn weapons(&self) -> impl Iterator<Item = &Weapon> {
        self.weapons.values()
    }

    /// Retrieve an iterator of outfits
    pub fn outfits(&self) -> impl Iterator<Item = &Outfit> {
        self.outfits.values()
    }

    /// Retrieve an iterator of equipment
    pub fn equipment(&self) -> impl Iterator<Item = &Equipment> {
        self.equipment.values()
    }

    /// Retrieve an iterator of aspects
    pub fn aspects(&self) -> impl Iterator<Item = &Aspect> {
        self.aspects.values()
    }

    /// Retrieve an iterator of enchants
    pub fn enchants(&self) -> impl Iterator<Item = &Enchant> {
        self.enchants.values()
    }

    /// Retrieve an iterator of presets
    pub fn presets(&self) -> impl Iterator<Item = &Preset> {
        self.presets.values()
    }

    pub fn origins(&self) -> impl Iterator<Item = &Origin> {
        self.origins.values()
    }

    pub fn resonances(&self) -> impl Iterator<Item = &Resonance> {
        self.resonances.values()
    }

    pub fn objectives(&self) -> impl Iterator<Item = &Objective> {
        self.objectives.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::req::PrereqGroup;

    const NEW_FORMAT: &str = r#"{
        "talents": {
            "a_world_without_song": {
                "name": "A World Without Song",
                "desc": "",
                "rarity": "Advanced",
                "category": "Silencer",
                "reqs": "75s WND",
                "prereqs": ["talent:silencers_blade"],
                "count_towards_talent_total": true,
                "vaulted": false,
                "voi": false
            }
        },
        "objectives": {
            "justicar": {
                "name": "Justicar",
                "desc": "",
                "accountWideUnlock": true
            }
        }
    }"#;

    #[test]
    fn new_format_requirement() {
        let data = DeepData::from_json(NEW_FORMAT).unwrap();
        let talent = data.get_talent("a_world_without_song").unwrap();

        let req = talent.requirement("a_world_without_song");
        assert_eq!(req.name, Some("talent:a_world_without_song".to_string()));
        assert_eq!(
            req.prereqs,
            std::collections::BTreeSet::from([PrereqGroup::single("talent:silencers_blade")])
        );
        assert_eq!(req.clauses.len(), 1);
    }

    #[test]
    fn objectives_table_loads() {
        let data = DeepData::from_json(NEW_FORMAT).unwrap();
        let objective = data.get_objective("justicar").unwrap();

        assert_eq!(objective.name, "Justicar");
        assert!(objective.account_wide_unlock);

        let req = data.requirement("objective:justicar").unwrap();
        assert_eq!(req.name, Some("objective:justicar".to_string()));
        assert!(req.is_empty());
    }
}
