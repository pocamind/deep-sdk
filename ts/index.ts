export { ATTUNEMENT_STATS, CORE_STATS, WEAPON_STATS, ITEM_RARITIES, TALENT_RARITIES, WEAPON_TYPES, EQUIPMENT_SLOTS } from './types.js';
export type { AggregatedStats, Aspect, BuildSnapshot, Enchant, Equipment, EquipmentSelection, EquipmentSlot, ItemRarity, Mantra, MantraType, Outfit, Preset, RangeType, Stat, StatSource, Talent, TalentRarity, Weapon, WeaponType } from './types.js';
export type { Atom, Clause, ClauseType, Reducability } from './requirement.js';

import type { AggregatedStats, Aspect, BuildSnapshot, Enchant, Equipment, Mantra, Outfit, Preset, Stat, Talent, Weapon } from './types.js';
import type { Clause } from './requirement.js';

// a top-level await here breaks older webkit stuff
let wasm: any = null;
let initPromise: Promise<void> | null = null;

/** Loads and instantiates the wasm module. Await this once before using anything
 * else in the SDK. Safe to call multiple times, retries after a failed attempt. */
export function init(): Promise<void> {
    initPromise ??= (async () => {
        const mod = await import('./pkg/deepwoken.js');

        if (typeof process !== 'undefined' && process.versions?.node) {
            const { readFile } = await import('node:fs/promises');
            const { createRequire } = await import('node:module');
            const require = createRequire(import.meta.url);
            const wasmPath = require.resolve('deepwoken/pkg/deepwoken_bg.wasm');
            await mod.default(await readFile(wasmPath));
        } else {
            await mod.default();
        }

        wasm = mod;
    })().catch((e) => {
        initPromise = null;
        throw e;
    });
    return initPromise;
}

function w(): any {
    if (!wasm) throw new Error('deepwoken: not initialized, await init() first');
    return wasm;
}

export class DeepData {
    /** @internal */
    _wasm: any;

    private constructor(wasm: any) {
        this._wasm = wasm;
    }

    static async fetchLatest(): Promise<DeepData> {
        return new DeepData(await w().DeepData.fetchLatest());
    }

    static async fetchLatestFrom(owner: string, repo: string): Promise<DeepData> {
        return new DeepData(await w().DeepData.fetchLatestFrom(owner, repo));
    }

    static fromJson(json: string): DeepData {
        return new DeepData(w().DeepData.fromJson(json));
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

    aggregateStats(snapshot: BuildSnapshot): AggregatedStats { return this._wasm.aggregateStats(snapshot); }
}

/** Transforms the name of things ingame into a parsable identifier/key used in the database */
export function nameToIdentifier(name: string): string {
    return w().nameToIdentifier(name);
}

export class StatMap {
    /** @internal */
    _wasm: any;

    constructor(map: Partial<Record<Stat, number>> = {}) {
        this._wasm = new (w().StatMap)(map);
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
        this._wasm = new (w().Requirement)(input);
    }

    satisfiedBy(stats: StatMap): boolean { return this._wasm.satisfiedBy(stats._wasm); }
    isEmpty(): boolean { return this._wasm.isEmpty(); }
    usedStats(): Stat[] { return this._wasm.usedStats(); }
    name(): string | null { return this._wasm.name(); }
    prereqs(): string[] { return this._wasm.prereqs(); }
    clauses(): Clause[] { return this._wasm.clauses(); }
    toString(): string { return this._wasm.toString(); }
}
