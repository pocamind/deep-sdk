# Building requirements
The _requirement_ is a fundamental object, representing a stat requirement for anything you could probably want to obtain.  In this model, we express a requirement as a comma-separated list of clauses, that have to be simultaneously satisfied in order to acquire whatever that requirement is for. Each clause can contain multiple `OR` requirements to be satisfied.

For example, the requirement for **Perseverance**:
```
30 ftd, 30 wll
╰────╯ <-- a single clause
```

For **Neuroplasticity**:
```
35 int or 35 cha or 35 wll
╰────────────────────────╯ <-- still a single clause!
```

For a more complex requirement like **Silentheart**:
```
25r str, (25 agl or 25 cha), (hvy + med + lht = 75)
╰─────╯  ╰────────────────╯  ╰─────────────────────╯
parantheses are optional ^^   ^^^ a sum requirement
```

An insane, arbitrary example to show what's expressible:
```
35 cha or (flm + wnd = 50), (lht + med + hvy = 90) or (lht + mtl + str = 75), 90 wll or 30 int
```

Below are examples of different syntax you can use to build a requirement:
- `ftd = 40`
- `ftd = 40`
- `FTD=40`
- `40 FTD`
- `40 Fortitude`
- `1 cha OR 2 int`
- `STr=1 oR 95 cha`

## Strict or reducible
Some requirements prevent you from using SoM (Shrine of Mastery) if acquired, and some don't. We refer to the blocking ones as _strict_, and the rest as _reducible_ (the stat is reducible via SoM).

To express a strict requirement, append an 's' to the end of a value:
- `40s ftd`
- `25s wnd or 25s ltn`

To express a reducible requirement, append an 'r' instead:
- `40r ftd`
- `25r wnd or 25r ltn`

If unspecified, reducability has defaults depending on the kind of clause it's in.

There's a few rules on reducability to mirror in-game requirements:
- Any single stat clause (i.e. `90 ftd`) is strict by default
> Most single-stat requirement talents are strict.
- Any stat in an `OR` clause (i.e. `35 int or 35 cha`) is reducible by default
> There are no known `OR` clauses with any strict components, think oaths, the Mind and Body stat, etc
- Strict sum components do **not** exist, they are all reducible even when specified strict.
> Due to the fact we don't need strict sum representations for anything in-game, and complications defining how a strict sum should gate SoM usage, we leave it undefined.
