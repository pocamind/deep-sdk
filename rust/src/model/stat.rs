use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de};

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stat {
    Strength = 0,
    Fortitude = 1,
    Agility = 2,
    Intelligence = 3,
    Willpower = 4,
    Charisma = 5,
    HeavyWeapon = 6,
    MediumWeapon = 7,
    LightWeapon = 8,
    Frostdraw = 9,
    Flamecharm = 10,
    Thundercall = 11,
    Galebreathe = 12,
    Shadowcast = 13,
    Ironsing = 14,
    Bloodrend = 15,
    /// A stat representing the total cost of all stats, aka the 'Cost'
    /// Can and should be used to model power levels
    Total = 16
}

impl Stat {
    pub fn from_u32_unchecked(value: u32) -> Self {
        // LOL
        unsafe { std::mem::transmute(value) }
    }
    
    pub fn short_name(&self) -> &'static str {
        match self {
            Stat::Strength => "STR",
            Stat::Fortitude => "FTD",
            Stat::Agility => "AGL",
            Stat::Intelligence => "INT",
            Stat::Willpower => "WLL",
            Stat::Charisma => "CHA",
            Stat::HeavyWeapon => "HVY",
            Stat::MediumWeapon => "MED",
            Stat::LightWeapon => "LHT",
            Stat::Frostdraw => "ICE",
            Stat::Flamecharm => "FLM",
            Stat::Thundercall => "LTN",
            Stat::Galebreathe => "WND",
            Stat::Shadowcast => "SDW",
            Stat::Ironsing => "MTL",
            Stat::Bloodrend => "BLD",
            Stat::Total => "TTL",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Stat::Strength => "Strength",
            Stat::Fortitude => "Fortitude",
            Stat::Agility => "Agility",
            Stat::Intelligence => "Intelligence",
            Stat::Willpower => "Willpower",
            Stat::Charisma => "Charisma",
            Stat::HeavyWeapon => "Heavy",
            Stat::MediumWeapon => "Medium",
            Stat::LightWeapon => "Light",
            Stat::Frostdraw => "Frostdraw",
            Stat::Flamecharm => "Flamecharm",
            Stat::Thundercall => "Thundercall",
            Stat::Galebreathe => "Galebreathe",
            Stat::Shadowcast => "Shadowcast",
            Stat::Ironsing => "Ironsing",
            Stat::Bloodrend => "Bloodrend",
            Stat::Total => "Total",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.to_uppercase();
        match name.as_str() {
            "STRENGTH" => Some(Stat::Strength),
            "FORTITUDE" => Some(Stat::Fortitude),
            "AGILITY" => Some(Stat::Agility),
            "INTELLIGENCE" => Some(Stat::Intelligence),
            "WILLPOWER" => Some(Stat::Willpower),
            "CHARISMA" => Some(Stat::Charisma),
            "HEAVY WEP." | "HEAVY" => Some(Stat::HeavyWeapon),
            "MEDIUM WEP." | "MEDIUM" => Some(Stat::MediumWeapon),
            "LIGHT WEP." | "LIGHT" => Some(Stat::LightWeapon),
            "FROSTDRAW" => Some(Stat::Frostdraw),
            "FLAMECHARM" => Some(Stat::Flamecharm),
            "THUNDERCALL" => Some(Stat::Thundercall),
            "GALEBREATHE" => Some(Stat::Galebreathe),
            "SHADOWCAST" => Some(Stat::Shadowcast),
            "IRONSING" => Some(Stat::Ironsing),
            "BLOODREND" => Some(Stat::Bloodrend),
            "TOTAL" => Some(Stat::Total),
            _ => None,
        }
    }

    pub fn from_short_name(short: &str) -> Option<Self> {
        let short = short.to_uppercase();

        match short.as_str() {
            "STR" => Some(Stat::Strength),
            "FTD" => Some(Stat::Fortitude),
            "AGL" | "AGI" => Some(Stat::Agility),
            "INT" => Some(Stat::Intelligence),
            // bruh
            "WLL" | "WIL" => Some(Stat::Willpower),
            "CHA" => Some(Stat::Charisma),
            "HVY" => Some(Stat::HeavyWeapon),
            "MED" => Some(Stat::MediumWeapon),
            "LHT" => Some(Stat::LightWeapon),
            "ICE" => Some(Stat::Frostdraw),
            "FLM" | "FIR" => Some(Stat::Flamecharm),
            "LTN" => Some(Stat::Thundercall),
            "WND" => Some(Stat::Galebreathe),
            "SDW" => Some(Stat::Shadowcast),
            "MTL" => Some(Stat::Ironsing),
            "BLD" => Some(Stat::Bloodrend),
            "TTL" | "TOT" => Some(Stat::Total),
            _ => None,
        }
    }

    pub const fn is_attunement(&self) -> bool {
        match self {
            Stat::Frostdraw | Stat::Flamecharm | Stat::Thundercall
            | Stat::Galebreathe | Stat::Shadowcast | Stat::Ironsing
            | Stat::Bloodrend => true,
            _ => false,
        }
    }

    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    pub const fn as_i64(self) -> i64 {
        self as i64
    }
}

impl From<Stat> for u32 {
    fn from(stat: Stat) -> u32 {
        stat as u32
    }
}

impl From<Stat> for i64 {
    fn from(stat: Stat) -> i64 {
        stat as i64
    }
}

impl TryFrom<u32> for Stat {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Stat::Strength),
            1 => Ok(Stat::Fortitude),
            2 => Ok(Stat::Agility),
            3 => Ok(Stat::Intelligence),
            4 => Ok(Stat::Willpower),
            5 => Ok(Stat::Charisma),
            6 => Ok(Stat::HeavyWeapon),
            7 => Ok(Stat::MediumWeapon),
            8 => Ok(Stat::LightWeapon),
            9 => Ok(Stat::Frostdraw),
            10 => Ok(Stat::Flamecharm),
            11 => Ok(Stat::Thundercall),
            12 => Ok(Stat::Galebreathe),
            13 => Ok(Stat::Shadowcast),
            14 => Ok(Stat::Ironsing),
            15 => Ok(Stat::Bloodrend),
            _ => Err("Invalid stat id"),
        }
    }
}

impl TryFrom<i64> for Stat {
    type Error = &'static str;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err("Stat id cannot be negative");
        }
        (value as u32).try_into()
    }
}

impl FromStr for Stat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_short_name(s)
            .or_else(|| Self::from_name(s))
            .ok_or("Invalid stat name or abbreviation")
    }
}

impl fmt::Debug for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<Stat> for String {
    fn from(stat: Stat) -> String {
        stat.name().to_string()
    }
}

impl<'de> Deserialize<'de> for Stat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // we can do this since it implements from_str
        s.parse().map_err(de::Error::custom)
    }
}

impl Serialize for Stat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}