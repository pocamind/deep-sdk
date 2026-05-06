use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $str:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $( $(#[$vmeta])* $variant ),+
        }

        impl $name {
            pub const ALL: &[Self] = &[ $( Self::$variant ),+ ];

            #[must_use]
            pub fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $str ),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.name())
            }
        }

        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(self.name())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = String::deserialize(d)?;
                Self::ALL.iter().find(|v| v.name() == s).copied()
                    .ok_or_else(|| serde::de::Error::custom(
                        format!("unknown {} variant: {s}", stringify!($name))
                    ))
            }
        }
    };
}

string_enum! {
    pub enum ItemRarity {
        Common => "Common",
        Uncommon => "Uncommon",
        Rare => "Rare",
        Legendary => "Legendary",
        Mythical => "Mythical",
        Unique => "Unique",
        Exclusive => "Exclusive",
        Relic => "Relic",
        Unknown => "Unknown",
        Named => "Named",
        Hallowtide => "Hallowtide",
        Spec => "Spec",
    }
}

string_enum! {
    pub enum TalentRarity {
        Common => "Common",
        Rare => "Rare",
        Advanced => "Advanced",
        Faction => "Faction",
        Innate => "Innate",
        Memento => "Memento",
        Murmur => "Murmur",
        Oath => "Oath",
        Origin => "Origin",
        Quest => "Quest",
        Spec => "Spec",
        Equipment => "Equipment",
        Outfit => "Outfit",
        Weapon => "Weapon",
    }
}

string_enum! {
    pub enum WeaponType {
        Bow => "Bow",
        Club => "Club",
        Dagger => "Dagger",
        Fist => "Fist",
        Greataxe => "Greataxe",
        Greatcannon => "Greatcannon",
        Greathammer => "Greathammer",
        Greatsword => "Greatsword",
        ParryingDagger => "Parrying Dagger",
        Pistol => "Pistol",
        Rapier => "Rapier",
        Rifle => "Rifle",
        Shield => "Shield",
        Spear => "Spear",
        SpearRifle => "Spear / Rifle",
        Staff => "Staff",
        Sword => "Sword",
        SwordGreatsword => "Sword / Greatsword",
        Twinblade => "Twinblade",
        Exclusive => "Exclusive",
    }
}

string_enum! {
    pub enum EquipmentSlot {
        Arms => "Arms",
        Earrings => "Earrings",
        Face => "Face",
        Head => "Head",
        Legs => "Legs",
        Rings => "Rings",
        Torso => "Torso",
    }
}

string_enum! {
    pub enum RangeType {
        Sweep => "Sweep",
        Lunge => "Lunge",
    }
}

string_enum! {
    pub enum MantraType {
        Normal => "Normal",
        Oath => "Oath",
        Origin => "Origin",
        Event => "Event",
        Monster => "Monster",
    }
}
