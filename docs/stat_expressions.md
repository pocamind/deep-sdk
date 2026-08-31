# Stat Expressions

This SDK consumes the schema provided by [our data repository](https://github.com/pocamind/data). This is an example of what a stat map looks like: 

```json
{ 
    "Melee Pen": 5 
}
```

Some talents have their effectiveness reduced by a known factor or depend on a combat condition. Because of this, the value of a stat key can be either a number or a *stat expression*. Expressions can read the fixed derived bindings below and variables declared by the data bundle's `variables` table.

Examples of such expressions:

**Reinforced Armor**
```json
{ 
    "Pen Resistance": "max(30 - 0.8 * max(90 - FTD, 0), 10)" 
}
```

**Heroism (enchant)**
```json
{ 
    "Damage": "if(HEALTH_PERCENT >= 75, 4 * min(floor((HEALTH_PERCENT - 75) / 5) + 1, 5), if(HEALTH_PERCENT <= 25, 4 * min(floor((25 - HEALTH_PERCENT) / 5) + 1, 5), 0)) * if(PVP, 1, 0.25)"
}
```

**Grim (enchant)**
```json
{ 
    "Damage": "if(GRIM, 25, 0)"
}
```

Reinforced Armor should be self explanatory.
Heroism reads the declared `HEALTH_PERCENT` variable (see Declared Varaibles section), granting 4% per 5% of health past 75% or past 25%, capped at 20%, and a quarter of that outside PvP.
Grim contributes nothing when its declared `GRIM` toggle is off.

## Full list of variables

**Stats:**
STR FTD AGL INT WLL CHA
LHT MED HVY
FLM ICE LTN WND SDW MTL BLD

**Fixed meta bindings:**
TTL (total invested points / cost)
PWR (power level)
PVP (true if in PvP combat)
PVE (true if in PvE combat)

**Declared variables:**

Each entry in the data repo's `variables` table contains an identifier, label, control kind, and default. There are two kinds: booleans and numbers (expressed with a toggle/slider on the frontend respectively). 

A formula will be rejected if it contains identifiers that are neither fixed bindings nor declared variables.

`DeepData::variable_users` returns the namespaced ids (e.g. talent:speed_demon) for variables that use a given variable (in this case, SPEED_BOOST). 
