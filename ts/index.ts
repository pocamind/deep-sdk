export { ATTUNEMENT_STATS, CORE_STATS, WEAPON_STATS } from './types.js';
export type { Aspect, Mantra, Outfit, Stat, Talent, Weapon } from './types.js';

import type { Aspect, Mantra, Outfit, Stat, Talent, Weapon } from './types.js';

// dynamically import wasm bc the server will try to load wasm regardless man
let WasmDeepData: any;
let WasmStatMap: any;

if (typeof window !== 'undefined') {
    const wasm = await import('./pkg/deepwoken.js');
    await wasm.default();
    WasmDeepData = wasm.DeepData;
    WasmStatMap = wasm.StatMap;
}

export class DeepData {
    private _wasm: any;

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
    getAspect(name: string): Aspect | null { return this._wasm.getAspect(name); }

    talents(): Talent[] { return this._wasm.talents(); }
    mantras(): Mantra[] { return this._wasm.mantras(); }
    weapons(): Weapon[] { return this._wasm.weapons(); }
    outfits(): Outfit[] { return this._wasm.outfits(); }
    aspects(): Aspect[] { return this._wasm.aspects(); }
}

export class StatMap {
    private _wasm: any;

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
    toJSON(): Partial<Record<Stat, number>> { return this._wasm.toJSON(); }
}
