// copy the rust provided types (is there a better way to mirror types? man)

export type Stat =
    | "Strength" | "Fortitude" | "Agility" | "Intelligence" | "Willpower" | "Charisma"
    | "Heavy" | "Medium" | "Light"
    | "Frostdraw" | "Flamecharm" | "Thundercall" | "Galebreathe" | "Shadowcast" | "Ironsing" | "Bloodrend"
    | "Total";

export interface Talent {
    name: string;
    desc: string;
    rarity: string;
    category: string;
    reqs: string;
    exclusive: string[];
    innates: Record<string, number>;
    not_counted_towards_total: boolean;
    vaulted: boolean;
}

export interface Weapon {
    name: string;
    type: string;
    damage_type: string;
    reqs: string;
    damage: number;
    pen: number;
    chip: number;
    weight: number;
    range: number;
    speed: number;
    endlag: number;
    scaling: Record<string, number>;
}

export interface Mantra {
    name: string;
    desc: string;
    stars: number;
    category: string;
    type: string;
    attributes: string[];
    reqs: string;
    vaulted: boolean;
}

export interface Outfit {
    name: string;
    category: string;
    durability: number;
    resistances: Record<string, number>;
    extra_percents: Record<string, number>;
    talent: string | null;
    reqs: string;
    mats: Record<string, number>;
    notes: number;
}

export interface Aspect {
    name: string;
    innate: Partial<Record<Stat, number>>;
    is_pathfinder: boolean;
}
