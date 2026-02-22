export type { Talent, Weapon, Mantra, Outfit, Aspect, Stat } from './types.js';

import wasmInit, { DeepData as WasmDeepData } from './pkg/deepwoken.js';
import type { Talent, Weapon, Mantra, Outfit, Aspect } from './types.js';

await wasmInit();

export class DeepData {
    private _wasm: WasmDeepData;

    private constructor(wasm: WasmDeepData) {
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
