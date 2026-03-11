import type { Stat } from './generated.js';

export type Reducability = "reducible" | "strict";
export type ClauseType = "and" | "or";

export interface Atom {
    reducability: Reducability;
    value: number;
    stats: Stat[];
}

export interface Clause {
    clause_type: ClauseType;
    atoms: Atom[];
}
