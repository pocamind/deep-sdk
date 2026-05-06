// Types that wrap the structures found in pocamind/data

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::error::{DeepError, Result};
use crate::model::enums::{EquipmentSlot, ItemRarity, MantraType, RangeType, TalentRarity, WeaponType};
use crate::model::req::Requirement;
use crate::util::name_to_identifier;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatValue {
    pub value: f64,
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
    pub reqs: Requirement,
    pub mats: HashMap<String, i64>,
    pub notes: i64,
    #[serde(default)]
    pub voi: bool,
    pub desc: String,
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
    pub voi: bool,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Talent {
    pub name: String,
    pub desc: String,
    pub rarity: TalentRarity,
    pub category: String,
    pub reqs: Requirement,
    pub count_towards_talent_total: bool,
    pub vaulted: bool,
    pub voi: bool,
    #[serde(default)]
    pub exclusive: Vec<String>,
    #[serde(default)]
    pub stats: HashMap<String, f64>,
    #[serde(default)]
    pub additional_info: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub roll2able: Option<bool>,
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
    pub enchantable: bool,
    pub equip_motifs: bool,
    pub voi: bool,
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
    pub vaulted: bool,
    pub voi: bool,
    #[serde(default)]
    pub damage: Vec<MantraDamageVariant>,
    #[serde(default)]
    pub scaling: HashMap<String, f64>,
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

    /// The raw json payload used to construct the object, which may be more up-to-date.
    /// The shape is guarenteed to have at least the fields that `DeepData` has.
    #[serde(skip, default)]
    raw: String,
}

impl DeepData {
    pub fn from_json(json: &str) -> Result<DeepData> {
        let mut ret: DeepData = serde_json::from_str(json).map_err(DeepError::from)?;

        ret.raw = json.to_string();

        Ok(ret)
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
}
