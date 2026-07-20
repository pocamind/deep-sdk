# Stat Expressions

This SDK consumes the schema provided by [our data repository](https://github.com/pocamind/data). This is an example of what a stat map looks like: 

```json
{ 
    "Melee Pen": 5 
}
```

Some talents have their effectiveness reduced by a known factor, derived from current stat investments. Because of this, the value of a stat key can be either a number or a *stat expression* - a function of integer ending attributes (STR, FTD, etc) and boolean combat flags (PVE, PVP). 

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
    "Damage": "if(PVP, 20, 5)" 
}
```

Reinforced Armor should be self explanatory.
Heroism has a different buff when in PvE combat as opposed to PvP.

## Full list of variables

**Stats:**
STR FTD AGL INT WLL CHA
LHT MED HVY
FLM ICE LTN WND SDW MTL BLD

**Meta:**
TTL (total invested points / cost)
PWR (power level)
PVP (true if in PvP combat)
PVE (true if in PvE combat)