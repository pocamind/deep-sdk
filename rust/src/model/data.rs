// Types that wrap the structures found in pocamind/data

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::error::{DeepError, Result};
use crate::model::req::Requirement;
use crate::util::name_to_identifier;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Aspect {
    pub name: String,
    pub innate: HashMap<Stat, i64>,
    pub is_pathfinder: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outfit {
    pub name: String,
    pub category: String,
    pub durability: i64,
    pub resistances: HashMap<String, f64>,
    pub extra_percents: HashMap<String, i64>,
    pub talent: Option<String>,
    pub reqs: Requirement,
    pub mats: HashMap<String, i64>,
    pub notes: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Talent {
    pub name: String,
    pub desc: String,
    pub rarity: String,
    pub category: String,
    pub reqs: Requirement,
    pub exclusive: Vec<String>,
    pub innates: HashMap<String, i64>,
    pub not_counted_towards_total: bool,
    pub vaulted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,
    #[serde(rename = "type")]
    pub weapon_type: String,
    pub damage_type: String,
    pub reqs: Requirement,
    pub damage: f64,
    pub pen: f64,
    pub chip: f64,
    pub weight: f64,
    pub range: f64,
    pub speed: f64,
    pub endlag: f64,
    /// Can contain stats as keys, can also contain
    /// pseudo-stats like 'Mind'
    pub scaling: HashMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mantra {
    pub name: String,
    pub desc: String,
    pub stars: i64,
    pub category: String,
    #[serde(rename = "type")]
    pub mantra_type: String,
    pub attributes: Vec<String>,
    pub reqs: Requirement,
    pub vaulted: bool,
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

    /// Retrieve an iterator of talents
    pub fn outfits(&self) -> impl Iterator<Item = &Outfit> {
        self.outfits.values()
    }

    /// Retrieve an iterator of talents
    pub fn aspects(&self) -> impl Iterator<Item = &Aspect> {
        self.aspects.values()
    }
}
