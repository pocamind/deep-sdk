export { ATTUNEMENT_STATS, CORE_STATS, WEAPON_STATS, ITEM_RARITIES, TALENT_RARITIES, WEAPON_TYPES, EQUIPMENT_SLOTS } from './types.js';
export type { Aspect, Enchant, Equipment, EquipmentSlot, ItemRarity, Mantra, MantraType, Outfit, Preset, RangeType, Stat, Talent, TalentRarity, Weapon, WeaponType } from './types.js';
export type { Atom, Clause, ClauseType, Reducability } from './requirement.js';

import type { Aspect, Enchant, Equipment, Mantra, Outfit, Preset, Stat, Talent, Weapon } from './types.js';
import type { Clause } from './requirement.js';

const wasm = await import('./pkg/deepwoken.js');

if (typeof process !== 'undefined' && process.versions?.node) {
    const { readFile } = await import('node:fs/promises');
    const { createRequire } = await import('node:module');
    const require = createRequire(import.meta.url);
    const wasmPath = require.resolve('deepwoken/pkg/deepwoken_bg.wasm');
    await wasm.default(await readFile(wasmPath));
} else {
    await wasm.default();
}

const WasmDeepData = wasm.DeepData;
const WasmStatMap = wasm.StatMap;
const WasmRequirement = wasm.Requirement;
const wasmNameToIdentifier = wasm.nameToIdentifier;

export class DeepData {
    /** @internal */
    _wasm: any;

    private constructor(wasm: any) {
        this._wasm = wasm;
    }

    static async fetchLatest(): Promise<DeepData> {
        return new DeepData(await WasmDeepData.fetchLatest());
    }

    static async fetchLatestFrom(owner: string, repo: string): Promise<DeepData> {
        return new DeepData(await WasmDeepData.fetchLatestFrom(owner, repo));
    }

    static fromJson(json: string): DeepData {
        return new DeepData(WasmDeepData.fromJson(json));
    }

    getTalent(name: string): Talent | null { return this._wasm.getTalent(name); }
    getMantra(name: string): Mantra | null { return this._wasm.getMantra(name); }
    getWeapon(name: string): Weapon | null { return this._wasm.getWeapon(name); }
    getOutfit(name: string): Outfit | null { return this._wasm.getOutfit(name); }
    getEquipment(name: string): Equipment | null { return this._wasm.getEquipment(name); }
    getAspect(name: string): Aspect | null { return this._wasm.getAspect(name); }
    getEnchant(name: string): Enchant | null { return this._wasm.getEnchant(name); }
    getPreset(name: string): Preset | null { return this._wasm.getPreset(name); }

    talents(): Talent[] { return this._wasm.talents(); }
    mantras(): Mantra[] { return this._wasm.mantras(); }
    weapons(): Weapon[] { return this._wasm.weapons(); }
    outfits(): Outfit[] { return this._wasm.outfits(); }
    equipment(): Equipment[] { return this._wasm.equipment(); }
    aspects(): Aspect[] { return this._wasm.aspects(); }
    enchants(): Enchant[] { return this._wasm.enchants(); }
    presets(): Preset[] { return this._wasm.presets(); }
}

/** Transforms the name of things ingame into a parsable identifier/key used in the database */
export function nameToIdentifier(name: string): string {
    return wasmNameToIdentifier(name);
}

export class StatMap {
    /** @internal */
    _wasm: any;

    constructor(map: Partial<Record<Stat, number>> = {}) {
        this._wasm = new WasmStatMap(map);
    }

    /* The total build cost, accounting for multi-attunement shenanigans */
    cost(): number { return this._wasm.cost(); }
    /* The points remaining available to invest */
    remaining(): number { return this._wasm.remaining(); }
    /* The level the character is at */
    level(): number { return this._wasm.level(); }

    get(stat: Stat): number { return this._wasm.get(stat); }
    set(stat: Stat, value: number) { this._wasm.set(stat, value); }
    shrineOrder(racial: StatMap): StatMap {
        const result = new StatMap();
        result._wasm = this._wasm.shrineOrder(racial._wasm);
        return result;
    }
    toJSON(): Partial<Record<Stat, number>> { return this._wasm.toJSON(); }

    /** The implicit talents (attunement milestones for now) granted by this stat map */
    implicitTalents(data: DeepData): Talent[] { return this._wasm.implicitTalents(data._wasm); }
}

export class Requirement {
    private _wasm: any;

    constructor(input: string) {
        this._wasm = new WasmRequirement(input);
    }

    satisfiedBy(stats: StatMap): boolean { return this._wasm.satisfiedBy(stats._wasm); }
    isEmpty(): boolean { return this._wasm.isEmpty(); }
    usedStats(): Stat[] { return this._wasm.usedStats(); }
    name(): string | null { return this._wasm.name(); }
    prereqs(): string[] { return this._wasm.prereqs(); }
    clauses(): Clause[] { return this._wasm.clauses(); }
    toString(): string { return this._wasm.toString(); }
}
