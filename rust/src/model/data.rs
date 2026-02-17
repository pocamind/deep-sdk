// Types that wrap the structures found in pocamind/data

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Stat;
use crate::model::req::Requirement;
use crate::error::{DeepError, Result};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepData {
    pub aspects: HashMap<String, Aspect>,
    pub talents: HashMap<String, Talent>,
    pub mantras: HashMap<String, Mantra>,
    pub weapons: HashMap<String, Weapon>,
    pub outfits: HashMap<String, Outfit>,
}

impl DeepData {
    pub fn from_json(json: &str) -> Result<DeepData> {
        serde_json::from_str(json)
            .map_err(DeepError::from)
    }

    pub fn get_talent(_name: &str) -> &Talent {
        todo!()
    }    
}