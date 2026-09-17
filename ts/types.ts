export { ATTUNEMENT_STATS, CORE_STATS, EQUIPMENT_SLOTS, ITEM_RARITIES, TALENT_RARITIES, WEAPON_STATS, WEAPON_TYPES } from './generated.js';
export type { EquipmentSlot, ItemRarity, MantraType, RangeType, Stat, TalentRarity, WeaponType } from './generated.js';

import type { EquipmentSlot, ItemRarity, MantraType, RangeType, Stat, TalentRarity, WeaponType } from './generated.js';

export interface StatValue {
    value: StatFormula;
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
    voi_only: boolean;
    implicit?: boolean;
    exclusive?: string[];
    immediate_grants?: string[];
    stats?: Record<string, StatFormula>;
    multiplicative_percents?: Record<string, StatFormula>;
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
    voi_only: boolean;
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
    can_offhand?: boolean;
    only_offhand?: boolean;
    talents?: string[];
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
    voi_only: boolean;
    damage?: MantraDamageVariant[];
    scaling?: Record<string, number>;
    stats?: Record<string, StatFormula>;
    multiplicative_percents?: Record<string, StatFormula>;
    modifiers?: string[];
    sparks?: string[];
    related_talents?: string[];
    shared_cooldowns?: string[];
    miscellaneous?: string;
    icon?: string;
    ether_cost?: number;
    flat_level_5_cost?: number;
}

export interface Outfit {
    name: string;
    variants: string[];
    category: string;
    durability: number;
    resistances: Record<string, number>;
    extra_percents: Record<string, number>;
    talents: string[];
    reqs: string;
    mats: Record<string, number>;
    notes: number;
    voi: boolean;
    voi_only: boolean;
    desc: string;
}

export interface Equipment {
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
    voi: boolean;
    voi_only: boolean;
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
    talent?: string[];
    exclude_cosmetics?: string[];
}

/** A stat contribution: a constant, or an expression over stat short-names (see docs/stat_expressions.md). */
export type StatFormula = number | string;

export type Variable =
    | {
        kind: "toggle";
        id: string;
        label: string;
        default: boolean;
    }
    | {
        kind: "slider";
        id: string;
        label: string;
        min: number;
        max: number;
        step: number;
        default: number;
    };

export interface Enchant {
    name: string;
    category: string;
    info: string;
    in_game_desc?: string;
    obtainable_in?: string;
    stats?: Record<string, StatFormula>;
    multiplicative_percents?: Record<string, StatFormula>;
}

export interface Pip {
    name: string;
    amounts: Partial<Record<EquipmentSlot, Record<string, Record<string, number>>>>;
}

export interface Preset {
    name: string;
    desc: string;
    opts: string;
    obtain_if_available: string[];
}

export interface Item {
    name: string;
    kind: string;
    asset?: string;
    icon?: string;
    desc?: string;
    effect?: string;
    brewing?: Record<string, number> | null;
    voi?: boolean;
    voi_only?: boolean;
}

export interface PotionEffect {
    name: string;
    order: number;
    timed: boolean;
    unit: string;
    formula: string;
    positive_suffixes: string[];
    negative_suffixes: string[];
    positive?: string;
    negative?: string;
}

export interface Origin {
    name: string;
    desc: string;
    outfit: string;
    spawns: string[];
    talents: string[];
    faction?: string;
}

export interface Resonance {
    name: string;
    desc: string;
    rarity: string;
}

export interface Objective {
    name: string;
    desc: string;
    account_wide_unlock: boolean;
    reqs: string;
    prereqs: string[];
}
