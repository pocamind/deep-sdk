use crate::model::data::DeepData;
use crate::model::enums::EquipmentSlot;

pub const PIP_RARITIES: &[&str] = &["Common", "Uncommon", "Rare", "Legendary"];

/// What a single pip of `pip` grants in a slot of the given equipment type and rarity, as
/// stat name and amount. Empty when the buff does not roll there at all.
///
/// Stats come back sorted by name so the same lookup always reads the same way.
#[must_use]
pub fn pip_stats<'a>(
    data: &'a DeepData,
    pip: &str,
    slot: EquipmentSlot,
    rarity: &str,
) -> Vec<(&'a str, f64)> {
    let mut stats: Vec<(&str, f64)> = if data.pips().next().is_none() {
        static_pip_stats(pip, slot, rarity)
    } else {
        data.get_pip(pip)
            .and_then(|buff| buff.amounts.get(&slot))
            .map(|stats| {
                stats
                    .iter()
                    .filter_map(|(stat, amounts)| {
                        amounts.get(rarity).map(|amount| (stat.as_str(), *amount))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    stats.sort_by(|(a, _), (b, _)| a.cmp(b));
    stats
}

type PipRow = (&'static str, EquipmentSlot, &'static str, [Option<f64>; 4]);

// Serves bundles that predate the `pips` table, which is where these amounts now live.
// Delete this table, `static_pip_stats`, and the branch in `pip_stats` that reaches for it,
// once a data release carries the table.
const PIP_BUFFS: &[PipRow] = &[
    ("Health", EquipmentSlot::Head, "Health", [None, Some(4.0), Some(4.0), Some(5.0)]),
    ("Health", EquipmentSlot::Arms, "Health", [Some(3.0), Some(4.0), Some(4.0), Some(5.0)]),
    ("Health", EquipmentSlot::Legs, "Health", [None, Some(4.0), Some(4.0), Some(5.0)]),
    ("Health", EquipmentSlot::Torso, "Health", [Some(3.0), None, Some(4.0), Some(5.0)]),
    ("Health", EquipmentSlot::Rings, "Health", [None, Some(2.0), Some(3.0), Some(4.0)]),
    ("Ether", EquipmentSlot::Head, "Ether", [None, Some(8.0), Some(10.0), Some(12.0)]),
    ("Ether", EquipmentSlot::Arms, "Ether", [None, Some(8.0), Some(10.0), Some(12.0)]),
    ("Ether", EquipmentSlot::Legs, "Ether", [None, Some(8.0), Some(10.0), Some(12.0)]),
    ("Ether", EquipmentSlot::Torso, "Ether", [None, Some(8.0), Some(10.0), Some(12.0)]),
    ("Ether", EquipmentSlot::Face, "Ether", [None, Some(4.0), Some(6.0), Some(8.0)]),
    ("Ether", EquipmentSlot::Earrings, "Ether", [None, Some(4.0), None, Some(8.0)]),
    ("Ether", EquipmentSlot::Rings, "Ether", [Some(4.0), Some(6.0), Some(8.0), Some(10.0)]),
    ("Sanity", EquipmentSlot::Face, "Sanity", [None, None, Some(4.0), Some(6.0)]),
    ("Sanity", EquipmentSlot::Face, "Ether", [None, None, Some(6.0), Some(8.0)]),
    ("Sanity", EquipmentSlot::Earrings, "Sanity", [None, None, Some(6.0), None]),
    ("Sanity", EquipmentSlot::Earrings, "Ether", [None, None, Some(6.0), None]),
    ("Sanity", EquipmentSlot::Rings, "Sanity", [None, Some(4.0), Some(6.0), Some(8.0)]),
    ("Sanity", EquipmentSlot::Rings, "Ether", [None, Some(4.0), Some(6.0), Some(8.0)]),
    ("Posture", EquipmentSlot::Rings, "Posture", [None, None, Some(1.0), Some(2.0)]),
    ("Physical Armor", EquipmentSlot::Head, "Physical Armor", [None, None, Some(2.0), Some(4.0)]),
    ("Physical Armor", EquipmentSlot::Arms, "Physical Armor", [None, None, Some(2.0), Some(4.0)]),
    ("Elemental Armor", EquipmentSlot::Head, "Elemental Armor", [None, None, Some(3.0), Some(4.0)]),
    ("Elemental Armor", EquipmentSlot::Arms, "Elemental Armor", [None, None, Some(3.0), Some(4.0)]),
    ("Anchor", EquipmentSlot::Legs, "Health", [None, None, None, Some(3.0)]),
    ("Anchor", EquipmentSlot::Legs, "Posture", [None, None, None, Some(0.5)]),
    ("Anchor", EquipmentSlot::Legs, "Knockback Resistance", [None, None, None, Some(10.0)]),
];

fn static_pip_stats(pip: &str, slot: EquipmentSlot, rarity: &str) -> Vec<(&'static str, f64)> {
    let Some(index) = PIP_RARITIES.iter().position(|r| *r == rarity) else {
        return Vec::new();
    };

    PIP_BUFFS
        .iter()
        .filter(|(name, pip_slot, ..)| *name == pip && *pip_slot == slot)
        .filter_map(|(_, _, stat, amounts)| amounts[index].map(|value| (*stat, value)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity is the interesting row: two stats, and one equipment type it skips.
    const FIXTURE: &str = r#"{
        "pips": {
            "sanity": {
                "name": "Sanity",
                "amounts": {
                    "Rings": {
                        "Sanity": { "Uncommon": 4, "Rare": 6, "Legendary": 8 },
                        "Ether": { "Uncommon": 4, "Rare": 6, "Legendary": 8 }
                    },
                    "Earrings": {
                        "Sanity": { "Rare": 6 },
                        "Ether": { "Rare": 6 }
                    }
                }
            }
        }
    }"#;

    fn fixture() -> DeepData {
        DeepData::from_json(FIXTURE).unwrap()
    }

    #[test]
    fn pip_stats_lookup() {
        let data = DeepData::default();

        assert_eq!(
            pip_stats(&data, "Health", EquipmentSlot::Rings, "Rare"),
            vec![("Health", 3.0)]
        );
        assert_eq!(pip_stats(&data, "Health", EquipmentSlot::Head, "Common"), vec![]);
        assert_eq!(pip_stats(&data, "Posture", EquipmentSlot::Head, "Legendary"), vec![]);
        assert_eq!(pip_stats(&data, "Health", EquipmentSlot::Rings, "Mythic"), vec![]);
    }

    #[test]
    fn multi_stat_pips_grant_riders() {
        let data = DeepData::default();

        let anchor = pip_stats(&data, "Anchor", EquipmentSlot::Legs, "Legendary");
        assert_eq!(anchor.len(), 3);
        assert!(anchor.contains(&("Knockback Resistance", 10.0)));

        let sanity = pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Uncommon");
        assert_eq!(sanity.len(), 2);
    }

    #[test]
    fn loaded_table_answers_the_lookup() {
        let data = fixture();

        assert_eq!(
            pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Uncommon"),
            vec![("Ether", 4.0), ("Sanity", 4.0)]
        );
        assert_eq!(
            pip_stats(&data, "Sanity", EquipmentSlot::Earrings, "Legendary"),
            vec![]
        );
        assert_eq!(pip_stats(&data, "Sanity", EquipmentSlot::Head, "Rare"), vec![]);
    }

    /// A loaded table is the whole truth. A buff missing from it does not roll, rather than
    /// falling through to the built-in copy.
    #[test]
    fn loaded_table_shuts_off_the_fallback() {
        let data = fixture();

        assert_eq!(pip_stats(&data, "Health", EquipmentSlot::Rings, "Rare"), vec![]);
    }

    #[test]
    fn fallback_and_loaded_table_agree_on_order() {
        let data = fixture();
        let empty = DeepData::default();

        let loaded = pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Rare");
        let fallback = pip_stats(&empty, "Sanity", EquipmentSlot::Rings, "Rare");

        assert_eq!(loaded, fallback);
    }
}
