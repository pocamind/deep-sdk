use crate::model::enums::EquipmentSlot;

pub const PIP_RARITIES: &[&str] = &["Common", "Uncommon", "Rare", "Legendary"];

type PipRow = (&'static str, EquipmentSlot, &'static str, [Option<f64>; 4]);

// TODO! ok maybe lazy map for this one
// do not underestimate modern cpu..
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

#[must_use]
pub fn pip_stats(pip: &str, slot: EquipmentSlot, rarity: &str) -> Vec<(&'static str, f64)> {
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

    #[test]
    fn pip_stats_lookup() {
        assert_eq!(
            pip_stats("Health", EquipmentSlot::Rings, "Rare"),
            vec![("Health", 3.0)]
        );
        assert_eq!(pip_stats("Health", EquipmentSlot::Head, "Common"), vec![]);
        assert_eq!(pip_stats("Posture", EquipmentSlot::Head, "Legendary"), vec![]);
        assert_eq!(pip_stats("Health", EquipmentSlot::Rings, "Mythic"), vec![]);
    }

    #[test]
    fn multi_stat_pips_grant_riders() {
        let anchor = pip_stats("Anchor", EquipmentSlot::Legs, "Legendary");
        assert_eq!(anchor.len(), 3);
        assert!(anchor.contains(&("Knockback Resistance", 10.0)));

        let sanity = pip_stats("Sanity", EquipmentSlot::Rings, "Uncommon");
        assert_eq!(sanity.len(), 2);
    }
}
