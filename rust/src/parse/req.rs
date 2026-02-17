use crate::model::req::{Atom, Clause, Reducability, Requirement};
use crate::error::{DeepError, Result };
use crate::Stat;
use log::warn;
use winnow::ascii::{alpha1, digit1, multispace0, Caseless};
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated};
use winnow::prelude::*;
use winnow::token::one_of;

/// Parse a string into a Requirement
///
/// If reducibility is unspecified:
/// - Unspecified atoms in OR clauses are reducible
/// - Unspecified atoms in AND clauses are strict iff they have a singular stat.
/// - Unspecified atoms in AND clauses are reducible if they are SUM types
///
/// Examples:
/// - "90 FTD" -> AND clause with strict 90 Fortitude
/// - "FTD = 90" -> Same thing but diff syntax, "ftd=90", "90ftd" also are valid
/// - "25 STR OR 25 AGL" -> OR clause with reducible atoms
/// - "25S STR OR 25 AGL" -> OR clause with asymmetric reducability
/// - "(LHT + MED + HVY = 90)" -> AND clause with sum atom (reducible by default)
/// - "(LHT + MED + HVY = 90S)" -> Any stat that make up the sum cannot be reduced
/// - "25S STR" -> strict atom
/// - "25R STR" -> reducible atom
/// - "reinforced = 90 FTD" -> named requirement (assignment syntax)
/// - "base, armor => reinforced := 90 FTD" -> named requirement with prerequisites
/// - "base => 90 FTD" -> anonymous requirement with a prerequisite
pub(crate) fn parse_req(input: &str) -> Result<Requirement> {
    let input = input.trim();
    requirement
        .parse(&input)
        .map_err(|e| DeepError::Req(e.to_string()))
}

// requirement = prefix? bare_requirement
// prefix = prereq_prefix | name_prefix
pub(crate) fn requirement(input: &mut &str) -> ModalResult<Requirement> {
    let _ = multispace0.parse_next(input)?;

    let prefix = opt(alt((prereq_prefix, name_prefix))).parse_next(input)?;

    let mut req = bare_requirement.parse_next(input)?;

    if let Some((prereqs, name)) = prefix {
        req.prereqs = prereqs;
        req.name = name;
    }

    Ok(req)
}

// prereq_prefix = identifier (',' identifier)* '=>' (identifier ':=')?
fn prereq_prefix(input: &mut &str) -> ModalResult<(Vec<String>, Option<String>)> {
    let prereqs: Vec<String> =
        separated(1.., identifier, (multispace0, ',', multispace0)).parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = "=>".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let name = opt((identifier, multispace0, ":=", multispace0)).parse_next(input)?;

    Ok((prereqs, name.map(|(n, _, _, _)| n)))
}

// name_prefix = identifier ':='
fn name_prefix(input: &mut &str) -> ModalResult<(Vec<String>, Option<String>)> {
    let name = identifier.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let _ = ":=".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    Ok((Vec::new(), Some(name)))
}

// identifier = (alpha | digit | '_')+
pub(crate) fn identifier(input: &mut &str) -> ModalResult<String> {
    let id: String =
        repeat(1.., one_of(('A'..='Z', 'a'..='z', '0'..='9', '_'))).parse_next(input)?;
    Ok(id)
}

// requirement = '(' ')' | clause (',' clause)*
fn bare_requirement(input: &mut &str) -> ModalResult<Requirement> {
    let clauses: Vec<Clause> = alt((
        // if () then its an empty req
        ('(', multispace0, ')').map(|_| Vec::new()),
        // Normal: 1+ clauses (clauses can have their own parens)
        separated(1.., clause, (multispace0, ',', multispace0)),
    ))
    .parse_next(input)?;

    Ok(Requirement {
        name: None,
        prereqs: Vec::new(),
        clauses,
    })
}

// clause = '(' clause_inner ')' | clause_inner
// clause_inner = atom ('OR' atom)*
// TODO! this is lacking an explicit 'AND', though you
// can just implicitly create new ANDs by making a new single atom clause!
fn clause(input: &mut &str) -> ModalResult<Clause> {
    let _ = multispace0.parse_next(input)?;

    // try (clause) first
    let result = alt((
        delimited(('(', multispace0), clause_inner, (multispace0, ')')),
        clause_inner,
    ))
    .parse_next(input)?;

    let _ = multispace0.parse_next(input)?;

    Ok(result)
}

fn clause_inner(input: &mut &str) -> ModalResult<Clause> {
    let first = atom.parse_next(input)?;
    let rest: Vec<ParsedAtom> = repeat(
        0..,
        preceded((multispace0, Caseless("OR"), multispace0), atom),
    )
    .parse_next(input)?;

    if rest.is_empty() {
        // single atom -> AND clause
        let atom = first.into_atom(false);
        Ok(Clause::and().atom(atom))
    } else {
        // multiple atoms -> OR clause (no AND support YET..)
        let mut clause = Clause::or();
        clause = clause.atom(first.into_atom(true));
        for parsed in rest {
            clause = clause.atom(parsed.into_atom(true));
        }

        Ok(clause)
    }
}

// intermediate atom structure
struct ParsedAtom {
    stats: Vec<Stat>,
    value: i64,
    reducability: Option<Reducability>,
}

impl ParsedAtom {
    fn into_atom(self, is_or: bool) -> Atom {
        let reducability = self.reducability.unwrap_or_else(|| {
            if is_or {
                // OR clause atoms default to reducible
                Reducability::Reducible
            } else if self.stats.len() > 1 {
                // multi-stat (sum) AND atoms default to reducible
                Reducability::Reducible
            } else {
                // single stat AND atoms default to strict
                Reducability::Strict
            }
        });

        if reducability == Reducability::Strict && self.stats.len() > 1 {
            warn!(
                "You have specified a strict SUM requirement, please note that \
                strict SUM requirements' semantics are not properly defined currently. \
                You probably don't need it anyways."
            )
        }

        let mut atom = Atom::new(reducability).value(self.value);

        for stat in self.stats {
            atom.add_stat(stat);
        }

        atom
    }
}

// atom = sum_expr | single_expr
fn atom(input: &mut &str) -> ModalResult<ParsedAtom> {
    let _ = multispace0.parse_next(input)?;

    let result = alt((
        sum_expr_parens,
        sum_expr_no_parens,
        single_expr_eq,     // stat '=' value reducability?
        single_expr_prefix, // value reducability? stat
    ))
    .parse_next(input)?;

    let _ = multispace0.parse_next(input)?;

    Ok(result)
}

// sum_expr_parens = '(' stat ('+' stat)* '=' value reducability? ')'
fn sum_expr_parens(input: &mut &str) -> ModalResult<ParsedAtom> {
    let _ = '('.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let stats: Vec<Stat> =
        separated(1.., stat, (multispace0, '+', multispace0)).parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = '='.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let value = number.parse_next(input)?;
    let reducability = opt(reducability_marker).parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = ')'.parse_next(input)?;

    Ok(ParsedAtom {
        stats,
        value,
        reducability,
    })
}

// sum_expr_no_parens = stat '+' stat ('+' stat)* '=' value reducability?
// needs 2 or more stats
fn sum_expr_no_parens(input: &mut &str) -> ModalResult<ParsedAtom> {
    let first = stat.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let _ = '+'.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let rest: Vec<Stat> =
        separated(1.., stat, (multispace0, '+', multispace0)).parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = '='.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let value = number.parse_next(input)?;
    let reducability = opt(reducability_marker).parse_next(input)?;

    let mut stats = vec![first];
    stats.extend(rest);

    Ok(ParsedAtom {
        stats,
        value,
        reducability,
    })
}

// single_expr_eq = stat '=' value reducability?
fn single_expr_eq(input: &mut &str) -> ModalResult<ParsedAtom> {
    let s = stat.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let _ = '='.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let value = number.parse_next(input)?;
    let reducability = opt(reducability_marker).parse_next(input)?;

    Ok(ParsedAtom {
        stats: vec![s],
        value,
        reducability,
    })
}

// single_expr_prefix = value reducability? stat
fn single_expr_prefix(input: &mut &str) -> ModalResult<ParsedAtom> {
    let value = number.parse_next(input)?;
    let reducability = opt(reducability_marker).parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let s = stat.parse_next(input)?;

    Ok(ParsedAtom {
        stats: vec![s],
        value,
        reducability,
    })
}

fn reducability_marker(input: &mut &str) -> ModalResult<Reducability> {
    let c = one_of(['S', 'R', 's', 'r']).parse_next(input)?;
    Ok(match c {
        'S' | 's' => Reducability::Strict,
        'R' | 'r' => Reducability::Reducible,
        _ => unreachable!(),
    })
}

fn number(input: &mut &str) -> ModalResult<i64> {
    digit1.try_map(|s: &str| s.parse::<i64>()).parse_next(input)
}

fn stat(input: &mut &str) -> ModalResult<Stat> {
    alpha1
        .verify_map(|s: &str| {
            let upper = s.to_uppercase();
            Stat::from_short_name(&upper)
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::model::req::ClauseType;

    use super::*;

    #[test]
    fn reinforced_armor() {
        let req = parse_req("90 FTD").unwrap();
        assert_eq!(req.clauses.len(), 1);

        let clause = &req.clauses[0];
        assert_eq!(clause.clause_type, ClauseType::And);
        assert_eq!(clause.atoms.len(), 1);

        let atom = clause.atoms.iter().next().unwrap();
        assert!(atom.stats.contains(&Stat::Fortitude));
        assert_eq!(atom.value, 90);
        assert_eq!(atom.reducability, Reducability::Strict);
    }

    #[test]
    fn bladeharper_variants() {
        // all valid representations of bladeharper requirements
        // for testing syntax stuff
        let variants = [
            "25 STR OR 25 AGL, 75 MED OR (LHT + MED + HVY = 90)",
            "(25 STR OR 25 AGL), (75 MED OR (LHT + MED + HVY = 90))",
            "STR = 25 OR AGL = 25, 75 MED OR (LHT + MED + HVY = 90)",
            "(STR = 25 OR AGL = 25), (75 MED OR (LHT + MED + HVY = 90))",
            "(STR = 25 OR AGL = 25),(75 MED OR (LHT + MED + HVY = 90))",
            "STR=25 OR AGL= 25,med=75 OR (lht + MED +hvy = 90)",
        ];

        let parsed: Vec<Requirement> = variants
            .iter()
            .map(|s| parse_req(s).expect(&format!("Failed to parse: {}", s)))
            .collect();

        // verify all parse successfully and are equal
        for (i, req) in parsed.iter().enumerate() {
            assert_eq!(req.clauses.len(), 2, "variant {} should have 2 clauses", i);
        }

        // all variants should be equal to each other
        for i in 1..parsed.len() {
            assert_eq!(parsed[0], parsed[i], "variant 0 should equal variant {}", i);
        }

        // verify structure of one of them (then they all are correct)
        let req = &parsed[0];

        // first clause: 25 STR OR 25 AGL
        let clause1 = &req.clauses[0];
        assert_eq!(clause1.clause_type, ClauseType::Or);
        assert_eq!(clause1.atoms.len(), 2);

        // second clause: 75 MED OR (LHT + MED + HVY = 90)
        let clause2 = &req.clauses[1];
        assert_eq!(clause2.clause_type, ClauseType::Or);
        assert_eq!(clause2.atoms.len(), 2);
    }

    #[test]
    fn bunch_of_random_stuff() {
        // silentheart reqs
        parse_req("25R STR, LHT + MED + HVY = 75, 25 CHA OR 25 AGL").unwrap();
        parse_req("(25R STR), LHT + MED + HVY = 75, 25 CHA OR 25 AGL").unwrap();
        parse_req("silentheart := str=25r,lht+med+hvy=75,25CHA OR agl=25r").unwrap();
        parse_req("silentheart := (str=25r),lht+med+hvy=75,25CHA OR agl=25r").unwrap();
        assert!(parse_req("silentheart := (str=25r),lht+med+hvy=75,25CHA OR agl=25r").is_ok());

        // neuro reqs
        assert!(parse_req("35cha OR 35wll OR 35int").is_ok());
        assert!(parse_req("35 cha OR 35 wll OR 35 int").is_ok());
        assert!(parse_req("()").unwrap().is_empty());

        // INVALID BAD REQ
        assert!(parse_req("(35 cha").is_err());
        assert!(parse_req("35 SBF").is_err());
        assert!(parse_req("35CHAOR35WLL").is_err());
    }

    #[test]
    fn explicit_reducability() {
        let req = parse_req("25S STR").unwrap();
        let atom = req.clauses[0].atoms.iter().next().unwrap();
        assert_eq!(atom.reducability, Reducability::Strict);

        let req = parse_req("25R STR").unwrap();
        let atom = req.clauses[0].atoms.iter().next().unwrap();
        assert_eq!(atom.reducability, Reducability::Reducible);

        let req = parse_req("25S STR OR 25R AGL").unwrap();
        assert_eq!(req.clauses[0].clause_type, ClauseType::Or);
    }

    #[test]
    fn prereq_prefix_parsing() {
        let req = parse_req("base, armor => reinforced := 90 FTD").unwrap();
        assert_eq!(req.prereqs, vec!["base", "armor"]);
        assert_eq!(req.name, Some("reinforced".to_string()));
        assert_eq!(req.clauses.len(), 1);

        let req = parse_req("base => 90 FTD").unwrap();
        assert_eq!(req.prereqs, vec!["base"]);
        assert!(req.name.is_none());

        let req = parse_req("base, armor => 50 INT, 25 STR OR 25 AGL").unwrap();
        assert_eq!(req.prereqs, vec!["base", "armor"]);
        assert_eq!(req.clauses.len(), 2);
    }

    #[test]
    fn casing_and_compactness() {
        let req1 = parse_req("25 str or 25 agl").unwrap();
        let req2 = parse_req("25 STR or 25 AGL").unwrap();
        assert_eq!(req1, req2);

        assert!(parse_req("25 Str OR 25 AgL").is_ok());

        assert!(parse_req("lht+hvy=90").is_ok());
        assert!(parse_req("lht+med+hvy=90").is_ok());
        assert!(parse_req("25 STR OR AGL=25,75S MED OR (LHT+MED+HVY=90)").is_ok());

        let compact = parse_req("str=25 OR agl=25").unwrap();
        let spaced = parse_req("STR = 25 OR AGL = 25").unwrap();
        assert_eq!(compact, spaced);
    }
}
