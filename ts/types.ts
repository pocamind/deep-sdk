export type { Stat, ItemRarity, TalentRarity, WeaponType, EquipmentSlot, RangeType, MantraType } from './generated.js';
export { CORE_STATS, WEAPON_STATS, ATTUNEMENT_STATS, ITEM_RARITIES, TALENT_RARITIES, WEAPON_TYPES, EQUIPMENT_SLOTS } from './generated.js';

import type { Stat, ItemRarity, TalentRarity, WeaponType, EquipmentSlot, RangeType, MantraType } from './generated.js';

export interface StatValue {
    value: number;
    percentage: boolean;
}

export interface Talent {
    name: string;
    desc: string;
    rarity: TalentRarity;
    category: string;
    reqs: string;
    count_towards_talent_total: boolean;
    vaulted: boolean;
    voi: boolean;
    additional_info?: string;
    icon?: string;
    roll2able?: boolean;
}

export interface Weapon {
    name: string;
    type: WeaponType;
    rarity: ItemRarity;
    damage: number | null;
    posture_damage: number | null;
    range: number | null;
    reqs: string;
    enchantable: boolean;
    equip_motifs: boolean;
    voi: boolean;
    desc: string;
    damage_types?: string[];
    range_type?: RangeType;
    attack_duration?: number;
    endlag?: number | null;
    swing_speed?: number;
    scaling?: Record<string, number>;
    bleed_damage?: number;
    chip_damage?: number;
    penetration?: number;
    posture_max?: number;
    posture_restoration?: number;
}

export interface MantraDamageLevel {
    level: string;
    damage: number;
    posture_damage: number | null;
}

export interface MantraDamageVariant {
    variant: string | null;
    levels: MantraDamageLevel[];
}

export interface Mantra {
    name: string;
    desc: string;
    stars: number;
    category: string;
    type: MantraType;
    attributes: string[];
    reqs: string;
    vaulted: boolean;
    voi: boolean;
    damage?: MantraDamageVariant[];
    scaling?: Record<string, number>;
    modifiers?: string[];
    related_talents?: string[];
    shared_cooldowns?: string[];
}

export interface Outfit {
    name: string;
    equippable: boolean;
    type: EquipmentSlot;
    rarity: ItemRarity;
    set: string | null;
    variants: string[];
    talents: string[];
    innates: Record<string, StatValue>;
    pips: Record<string, number>;
    reqs: string;
    media: string | null;
    voi: boolean;
    desc: string;
}

export interface AspectVariantInfo {
	name: string;
    unlock: string | null;
    colors: Record<string, string>;
}

export interface Aspect {
    name: string;
    desc: string;
    innate: Partial<Record<Stat, number>>;
    is_pathfinder: boolean;
    variants: Record<string, AspectVariantInfo>;
}
