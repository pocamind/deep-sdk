export { ATTUNEMENT_STATS, CORE_STATS, DAMAGE_TYPES, EQUIPMENT_SLOTS, ITEM_RARITIES, TALENT_RARITIES, WEAPON_STATS, WEAPON_TYPES } from './generated.js';
export type { DamageType, EquipmentSlot, ItemRarity, MantraType, RangeType, Stat, TalentRarity, WeaponType } from './generated.js';

import type { EquipmentSlot, ItemRarity, MantraType, RangeType, Stat, TalentRarity, WeaponType } from './generated.js';

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
    voi_only: boolean;
    implicit?: boolean;
    exclusive?: string[];
    stats?: Record<string, number>;
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
    modifiers?: string[];
    sparks?: string[];
    related_talents?: string[];
    shared_cooldowns?: string[];
    miscellaneous?: string;
}

export interface Outfit {
    name: string;
    pants_id: string | null;
    shirt_id: string | null;
    category: string;
    durability: number;
    resistances: Record<string, number>;
    extra_percents: Record<string, number>;
    talent: string | null;
    variants: string[];
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

export interface Enchant {
    name: string;
    category: string;
    info: string;
    in_game_desc?: string;
    obtainable_in?: string;
    stats?: Record<string, StatFormula>;
    conditional_stats?: Record<string, StatFormula>;
    multiplicative_percents?: Record<string, StatFormula>;
    conditional_multiplicative_percents?: Record<string, StatFormula>;
}

export interface Preset {
    name: string;
    desc: string;
    opts: string;
}

export interface StatSource {
    value: number;
    source: string;
    /** Pre-formatted display string, e.g. `+15%`, `×10%`, or a custom label. */
    display_value: string;
}

export interface EquipmentSelection {
    name: string;
    pips?: Record<string, string[]>;
    /** Quality stars, 0 to 3. */
    stars?: number;
    enchant?: string;
}

export interface WeaponSelection {
    name: string;
    /** Quality stars, 0 to 3. */
    stars?: number;
    /** The star buff, or '' for none. */
    starBuff?: 'DMG%' | 'PEN%' | 'WGT%' | '';
    enchant?: string;
}

export interface MantraSelection {
    name: string;
    /** 1 to 5. */
    level?: number;
    gem?: string;
    sparks?: string[];
    modifiers?: Record<string, number>;
}

export interface BuildSnapshot {
    stats?: Partial<Record<Stat, number>>;
    race?: string;
    talents?: string[];
    boons?: string[];
    traits?: Record<string, number>;
    equipment?: EquipmentSelection[];
    outfit?: string | null;
    weapon?: WeaponSelection | null;
    mantras?: MantraSelection[];
}

export interface BuildTotalStats {
    flat: Record<string, StatSource[]>;
    percents: Record<string, StatSource[]>;
    derived: Record<string, number>;
}

export type CombatState = 'OutOfCombat' | 'Pve' | 'Pvp';
export type AggregateMode = 'Base' | 'Optimistic';

/** The conditions a build's total stats is evaluated under */
export interface Scenario {
    mode?: AggregateMode;
    combatState?: CombatState;
    /** Attacker penetration percent for our own EHP calcs. */
    enemyPen?: number;
    /** Target resistance percent for damage-output related stat derivations. */
    enemyResistance?: number;
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
    accountWideUnlock: boolean;
    reqs: string;
    prereqs: string[];
}
