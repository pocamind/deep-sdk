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
    let mut stats: Vec<(&str, f64)> = data
        .get_pip(pip)
        .and_then(|buff| buff.amounts.get(&slot))
        .map(|stats| {
            stats
                .iter()
                .filter_map(|(stat, amounts)| amounts.get(rarity).map(|amount| (stat.as_str(), *amount)))
                .collect()
        })
        .unwrap_or_default();

    stats.sort_by(|(a, _), (b, _)| a.cmp(b));
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity is the interesting row: two stats, and one equipment type it skips. Anchor carries
    /// three stats at one rarity, which is what the rider assertions read.
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
            },
            "anchor": {
                "name": "Anchor",
                "amounts": {
                    "Legs": {
                        "Health": { "Legendary": 3 },
                        "Posture": { "Legendary": 0.5 },
                        "Knockback Resistance": { "Legendary": 10 }
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
        let data = fixture();

        assert_eq!(
            pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Rare"),
            vec![("Ether", 6.0), ("Sanity", 6.0)]
        );
        assert_eq!(pip_stats(&data, "Sanity", EquipmentSlot::Earrings, "Legendary"), vec![]);
        assert_eq!(pip_stats(&data, "Sanity", EquipmentSlot::Head, "Rare"), vec![]);
        assert_eq!(pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Mythic"), vec![]);
    }

    #[test]
    fn multi_stat_pips_grant_riders() {
        let data = fixture();

        let anchor = pip_stats(&data, "Anchor", EquipmentSlot::Legs, "Legendary");
        assert_eq!(
            anchor,
            vec![("Health", 3.0), ("Knockback Resistance", 10.0), ("Posture", 0.5)]
        );

        let sanity = pip_stats(&data, "Sanity", EquipmentSlot::Rings, "Uncommon");
        assert_eq!(sanity.len(), 2);
    }

    /// The table is the whole truth. A buff it does not carry does not roll.
    #[test]
    fn a_buff_outside_the_table_does_not_roll() {
        let data = fixture();

        assert_eq!(pip_stats(&data, "Health", EquipmentSlot::Rings, "Rare"), vec![]);
        assert_eq!(pip_stats(&DeepData::default(), "Sanity", EquipmentSlot::Rings, "Rare"), vec![]);
    }
}
