use crate::Stat;
use crate::error::{DeepError, Result};
use crate::model::opt::OptionalGroup;
use crate::model::req::{Requirement, Timing};
use crate::model::reqfile::Reqfile;
use crate::model::stat::StatRange;
use crate::util::reqtree::ReqTree;
use crate::util::traits::ReqVecExt;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::path::Path;
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::{alt, eof, separated};
use winnow::prelude::*;

use super::req::{identifier, requirement, stat};

enum BaseReqfileLine {
    Requirement(Requirement),
    DependencyWithIdentifier {
        prereqs: Vec<String>,
        dependent: String,
    },
}

/// A full reqfile line.
/// Note a required requirement cannot have optional prereqs.
enum ReqfileLine {
    /// The regular requirement line.
    Unspecified(BaseReqfileLine),
    /// A line with the prefix '+', that forces it and its dependents to all be required.
    /// Used to force a prereq of an optional req to be required.
    ForceRequired(BaseReqfileLine),
    /// A line with the prefix 'n ;', where n is an integer from 0-5. Marks the req as optional
    /// and assigns n as the weight. Recursively marks all prereqs as optional and ties their obtainment
    /// to each other.  
    Optional { base: BaseReqfileLine, weight: i64 },
    /// A line of the form 'n <= STAT <= m'
    /// Used to specify a range of stats for the final stat stage (OINLY FINAL SUPPORTED FOR NOW,
    /// maybe preshrine soon)
    RangeSpecifier {
        stat: Stat,
        range: RangeInclusive<u32>,
    },
}

impl ReqfileLine {
    pub fn base(&self) -> Option<&BaseReqfileLine> {
        match self {
            ReqfileLine::Unspecified(base)
            | ReqfileLine::ForceRequired(base)
            | ReqfileLine::Optional { base, .. } => Some(base),
            ReqfileLine::RangeSpecifier { .. } => None,
        }
    }

    pub fn base_mut(&mut self) -> Option<&mut BaseReqfileLine> {
        match self {
            ReqfileLine::Unspecified(base)
            | ReqfileLine::ForceRequired(base)
            | ReqfileLine::Optional { base, .. } => Some(base),
            ReqfileLine::RangeSpecifier { .. } => None,
        }
    }

    pub fn is_explicit_optional(&self) -> bool {
        matches!(self, ReqfileLine::Optional { .. })
    }
}

fn parse_reqfile_line(input: &str) -> std::result::Result<ReqfileLine, String> {
    let input = input.trim();
    reqfile_line
        .parse(input)
        .map_err(|e| format!("Parse error: {e}"))
}

fn reqfile_line(input: &mut &str) -> ModalResult<ReqfileLine> {
    let _ = multispace0.parse_next(input)?;
    alt((
        optional_line,
        force_required_line,
        range_specifier,
        base_reqfile_line.map(ReqfileLine::Unspecified),
    ))
    .parse_next(input)
}

// optional_line = weight ';' base_reqfile_line
fn optional_line(input: &mut &str) -> ModalResult<ReqfileLine> {
    let weight = digit1
        .try_map(|s: &str| s.parse::<i64>())
        .verify(|&n| (1..=20).contains(&n))
        .parse_next(input)?;

    let _ = (multispace0, ';', multispace0).parse_next(input)?;
    let base = base_reqfile_line.parse_next(input)?;
    Ok(ReqfileLine::Optional { base, weight })
}

// force_reqfile_line = '+' base_reqfile_line
fn force_required_line(input: &mut &str) -> ModalResult<ReqfileLine> {
    let _ = ('+', multispace0).parse_next(input)?;
    let base = base_reqfile_line.parse_next(input)?;
    Ok(ReqfileLine::ForceRequired(base))
}

// range_specifier = number "<=" stat "<=" number eof
fn range_specifier(input: &mut &str) -> ModalResult<ReqfileLine> {
    let lower = range_bound.parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = "<=".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let s = stat.parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = "<=".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let upper = range_bound.parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    eof.parse_next(input)?;

    Ok(ReqfileLine::RangeSpecifier {
        stat: s,
        range: lower..=upper,
    })
}

fn range_bound(input: &mut &str) -> ModalResult<u32> {
    digit1.try_map(|s: &str| s.parse::<u32>()).parse_next(input)
}

// base_reqfile_line = dependency_with_identifier | requirement
fn base_reqfile_line(input: &mut &str) -> ModalResult<BaseReqfileLine> {
    let _ = multispace0.parse_next(input)?;

    alt((
        dependency_with_identifier,
        requirement.map(BaseReqfileLine::Requirement),
    ))
    .parse_next(input)
}

// dependency_with_identifier = identifier (',' identifier)* '=>' identifier eof
// links prereqs to an existing named requirement (no inline definition)
fn dependency_with_identifier(input: &mut &str) -> ModalResult<BaseReqfileLine> {
    let prereqs: Vec<String> =
        separated(1.., identifier, (multispace0, ',', multispace0)).parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    let _ = "=>".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let dependent = identifier.parse_next(input)?;

    let _ = multispace0.parse_next(input)?;
    eof.parse_next(input)?;

    Ok(BaseReqfileLine::DependencyWithIdentifier { prereqs, dependent })
}

struct ParsedLine {
    rf_line: ReqfileLine,
    line_num: usize,
    timing: Timing,
}

struct ReqfileIndex {
    named: HashMap<String, usize>,
    str_to_idx: HashMap<String, usize>,
    dependency_statements: Vec<(Vec<String>, String, u64)>,
}

fn build_index(lines: &[ParsedLine]) -> Result<ReqfileIndex> {
    let mut named: HashMap<String, usize> = HashMap::new();
    let mut dependency_statements: Vec<(Vec<String>, String, u64)> = vec![];

    let str_to_idx: HashMap<String, usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| match l.rf_line.base() {
            Some(BaseReqfileLine::Requirement(req)) => Some((req.name_or_default(), i)),
            _ => None,
        })
        .collect();

    for (vec_idx, line) in lines.iter().enumerate() {
        let Some(base) = line.rf_line.base() else {
            continue;
        };

        match base {
            BaseReqfileLine::DependencyWithIdentifier { prereqs, dependent } => {
                // TODO! DependencyWithId should actually be a top level enum variant.
                // since its not affected by required, forced, unmarked semantics
                // so yea for now we error if the user misuses the api (FOR NOW)
                if let ReqfileLine::Unspecified(_) = &line.rf_line {
                } else {
                    return Err(DeepError::Reqfile {
                        line: line.line_num,
                        message: "Optional annotations '+' or ';' must be used \
                        at the requirement definition, not in a dependency statement, unless \
                        the definition is in the dependency statement itself."
                            .into(),
                    });
                }

                dependency_statements.push((
                    prereqs.clone(),
                    dependent.clone(),
                    line.line_num as u64,
                ));
            }
            BaseReqfileLine::Requirement(req) => {
                if let Some(name) = &req.name
                    && named.insert(name.clone(), vec_idx).is_some()
                {
                    return Err(DeepError::Reqfile {
                        line: line.line_num + 1,
                        message: format!("Duplicate identifier: {name}"),
                    });
                }
            }
        }
    }

    Ok(ReqfileIndex {
        named,
        str_to_idx,
        dependency_statements,
    })
}

fn validate_no_ambiguous_anonymous(lines: &[ParsedLine]) -> Result<()> {
    for line in lines {
        if let Some(BaseReqfileLine::Requirement(req)) = line.rf_line.base() {
            // only lf anon reqs
            if req.name.is_some() {
                continue;
            }

            let other_anon = lines
                .iter()
                .filter_map(|line| line.rf_line.base())
                .find(|other| {
                    if let BaseReqfileLine::Requirement(other_req) = other {
                        other_req.name.is_none()
                    && other_req.name_or_default() == req.name_or_default()
                    // if any one of them has prereqs, we want to raise this err
                    && (!other_req.prereqs.is_empty() || !req.prereqs.is_empty())
                    && other_req != req
                    } else {
                        false
                    }
                });

            if other_anon.is_some() {
                return Err(DeepError::Reqfile {
                    line: line.line_num,
                    message: format!(
                        "You may not have duplicate anonymous requirements if either of them have prerequisites: {}",
                        req.name_or_default()
                    ),
                });
            }
        }
    }

    Ok(())
}

fn resolve_dependencies(lines: &mut [ParsedLine], index: &ReqfileIndex) -> Result<()> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "line numbers will never get to u32 big"
    )]
    for (prereqs, name, line_num) in &index.dependency_statements {
        match index.named.get(name) {
            Some(vec_idx) => {
                // prereqs that don't resolve to an in-file req aren't a parse error since they may be
                // implicit talents (resolved from game data), which parsing is deliberately unaware of. actual
                // missing prereq errors are caught at solve-time.
                let line = &mut lines[*vec_idx];

                if let Some(BaseReqfileLine::Requirement(req)) = line.rf_line.base_mut() {
                    if !req.prereqs.is_empty() {
                        return Err(DeepError::Reqfile {
                            line: *line_num as usize,
                            message: format!("'{name}' has multiple prerequisite assignments."),
                        });
                    }

                    req.prereqs = prereqs.iter().cloned().collect();
                }
            }
            None => {
                return Err(DeepError::Reqfile {
                    line: *line_num as usize,
                    message: format!("Dependent: no variable named '{name}'."),
                });
            }
        }
    }

    Ok(())
}

fn build_req_tree(lines: &[ParsedLine]) -> ReqTree {
    let mut tree = ReqTree::new();

    for line in lines {
        if let Some(BaseReqfileLine::Requirement(req)) = line.rf_line.base() {
            tree.insert(req.clone());
        }
    }

    tree
}

fn validate_tree(
    lines: &[ParsedLine],
    tree: &ReqTree,
    str_to_idx: &HashMap<String, usize>,
) -> Result<()> {
    if let Some(cycle) = tree.find_cycle() {
        return Err(DeepError::Reqfile {
            line: 0,
            message: format!(
                "Prereqs cannot be dependent on each other. Found cycle: {}",
                cycle.join(" => ")
            ),
        });
    }

    // a required req cannot have an optional prereq
    for line in lines {
        if let ReqfileLine::Optional { base, .. } = &line.rf_line
            && let BaseReqfileLine::Requirement(req) = base
            && let Some(name) = &req.name
        {
            for dependent in tree.all_dependents(name) {
                let vec_idx = str_to_idx[&dependent];
                let dependent_line = &lines[vec_idx];

                if !dependent_line.rf_line.is_explicit_optional() {
                    return Err(DeepError::Reqfile {
                        line: line.line_num,
                        message: format!(
                            "'{}' was declared as optional, however one of its \
                                    dependents are required: '{} at line {}'.\n\
                                    Try marking '{}' as optional instead.",
                            name, dependent, dependent_line.line_num, dependent
                        ),
                    });
                }
            }
        }
    }

    Ok(())
}

fn build_optional_groups(
    lines: &[ParsedLine],
    tree: &ReqTree,
    str_to_idx: &HashMap<String, usize>,
) -> (Vec<OptionalGroup>, HashSet<String>) {
    let mut optional: Vec<OptionalGroup> = vec![];
    let mut marked_opt: HashSet<String> = HashSet::new();

    for line in lines {
        if let ReqfileLine::Optional { base, weight } = &line.rf_line
            && let BaseReqfileLine::Requirement(req) = base
        {
            let mut group = OptionalGroup {
                general: HashSet::new(),
                post: HashSet::new(),
                weight: *weight,
            };

            for req in tree
                .all_prereqs(&req.name_or_default())
                .iter()
                .chain(&[req.name_or_default()])
            {
                let vec_idx = str_to_idx[req];
                let req_line = &lines[vec_idx];

                if let Some(BaseReqfileLine::Requirement(req)) = req_line.rf_line.base() {
                    group.get_set(req_line.timing).insert(req.clone());
                }

                marked_opt.insert(req.clone());
            }

            optional.push(group);
        }
    }

    (optional, marked_opt)
}

fn apply_force_required(
    lines: &[ParsedLine],
    tree: &ReqTree,
    str_to_idx: &HashMap<String, usize>,
    optional: &mut [OptionalGroup],
    marked_opt: &mut HashSet<String>,
) {
    for line in lines {
        if let ReqfileLine::ForceRequired(base) = &line.rf_line
            && let BaseReqfileLine::Requirement(req) = base
        {
            for req in tree
                .all_prereqs(&req.name_or_default())
                .iter()
                .chain(&[req.name_or_default()])
            {
                let vec_idx = str_to_idx[req];
                let req_line = &lines[vec_idx];

                if let Some(BaseReqfileLine::Requirement(req)) = req_line.rf_line.base() {
                    for group in optional.iter_mut() {
                        group.get_set(req_line.timing).remove(req);
                    }
                }

                marked_opt.remove(req);
            }
        }
    }
}

fn collect_required_reqs(
    lines: &[ParsedLine],
    marked_opt: &HashSet<String>,
) -> (Vec<Requirement>, Vec<Requirement>) {
    let mut general: Vec<Requirement> = vec![];
    let mut post: Vec<Requirement> = vec![];

    for line in lines {
        if let Some(BaseReqfileLine::Requirement(req)) = line.rf_line.base() {
            if marked_opt.contains(&req.name_or_default()) {
                continue;
            }

            match line.timing {
                Timing::Free => general.push(req.clone()),
                Timing::Post => post.push(req.clone()),
            }
        }
    }

    (general, post)
}

/// Collect the post-shrine stat ranges, validating that range directives only
/// appear in the Post stage and that each stat is constrained at most once per stage.
fn build_final_ranges(lines: &[ParsedLine]) -> Result<Vec<StatRange>> {
    let mut ranges: Vec<StatRange> = vec![];
    let mut seen: HashSet<Stat> = HashSet::new();

    for line in lines {
        if let ReqfileLine::RangeSpecifier { stat, range } = &line.rf_line {
            if !matches!(line.timing, Timing::Post) {
                return Err(DeepError::Reqfile {
                    line: line.line_num,
                    message: format!(
                        "Range directives are only allowed in the Post stage for now, \
                        but one was found not in Post: '{}'.",
                        stat.name()
                    ),
                });
            }

            if range.start() > range.end() {
                return Err(DeepError::Reqfile {
                    line: line.line_num,
                    message: format!(
                        "Range directive for '{}' is inverted. The lower bound must not \
                        exceed the upper bound.",
                        stat.name()
                    ),
                });
            }

            if !seen.insert(*stat) {
                return Err(DeepError::Reqfile {
                    line: line.line_num,
                    message: format!(
                        "'{}' already has a range directive in this stage.",
                        stat.name()
                    ),
                });
            }

            ranges.push(StatRange {
                stat: *stat,
                range: range.clone(),
            });
        }
    }

    Ok(ranges)
}

fn validate_and_transform(mut lines: Vec<ParsedLine>) -> Result<Reqfile> {
    let index = build_index(&lines)?;
    validate_no_ambiguous_anonymous(&lines)?;
    resolve_dependencies(&mut lines, &index)?;

    let tree = build_req_tree(&lines);
    validate_tree(&lines, &tree, &index.str_to_idx)?;

    let (mut optional, mut marked_opt) = build_optional_groups(&lines, &tree, &index.str_to_idx);
    apply_force_required(
        &lines,
        &tree,
        &index.str_to_idx,
        &mut optional,
        &mut marked_opt,
    );

    let (general, post) = collect_required_reqs(&lines, &marked_opt);
    let final_ranges = build_final_ranges(&lines)?;

    Ok(Reqfile {
        general,
        post,
        final_ranges,
        optional,
        implicit: HashMap::new(),
    })
}

// TODO! this should really be the only entry point to create a Reqfile,
// since it also validates if the payload will be semantically correct
pub(crate) fn parse_reqfile_str(content: &str) -> Result<Reqfile> {
    let mut lines: Vec<ParsedLine> = vec![];

    let mut current = Timing::Free;

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        if line.to_uppercase().starts_with("FREE") {
            current = Timing::Free;
            continue;
        }

        if line.to_uppercase().starts_with("POST") {
            current = Timing::Post;
            continue;
        }

        let parsed = parse_reqfile_line(line).map_err(|e| DeepError::Reqfile {
            line: i + 1,
            message: e,
        })?;

        lines.push(ParsedLine {
            rf_line: parsed,
            line_num: i,
            timing: current,
        });
    }

    validate_and_transform(lines)
}

/// Parse '.req' files into a Reqfile struct
pub(crate) fn parse_reqfile(path: &Path) -> Result<Reqfile> {
    use std::fs;

    let content = fs::read_to_string(path)?;

    parse_reqfile_str(&content)
}

/// Generate a reqfile string from a Reqfile struct. This is outdated and
/// does not preserve optional groups or forced required annotations.
pub(crate) fn gen_reqfile(payload: &Reqfile) -> String {
    let mut output = String::new();

    output.push_str("# Auto-generated reqfile\n\n");
    output.push_str("Free:\n");

    // remove spaces from names
    //
    // we also give anonymous reqs with prereqs an identifier
    // (we don't assign names to potentially unnammed prereqs bc
    // it is a requirement that prereqs are already named)

    let clean_name = |name: &str| {
        name.replace(' ', "_")
            .replace(['[', ']', '\'', ':', '(', ')'], "")
    };

    let mut i = 0;

    let mut general = payload
        .general
        .iter()
        .map(|req: &Requirement| {
            i += 1;

            let mut req = req.clone();

            req.name = req.name.clone().or_else(|| {
                if req.prereqs.is_empty() {
                    None
                } else {
                    Some(format!("id_{i}"))
                }
            });

            req
        })
        .collect::<Vec<_>>();

    let mut post = payload
        .post
        .iter()
        .map(|req: &Requirement| {
            i += 1;

            let mut req = req.clone();

            req.name = req.name.clone().or_else(|| {
                if req.prereqs.is_empty() {
                    None
                } else {
                    Some(format!("id_{i}"))
                }
            });

            req
        })
        .collect::<Vec<_>>();

    general.map_names(clean_name);

    post.map_names(clean_name);

    for req in &general {
        use std::fmt::Write as _;
        let _ = writeln!(output, "{req}");
    }

    if !post.is_empty() {
        output.push_str("\nPost:\n");

        for req in &post {
            use std::fmt::Write as _;
            let _ = writeln!(output, "{req}");
        }
    }

    output
}
