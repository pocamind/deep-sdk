use std::collections::{HashMap, HashSet};
use std::path::Path;
use crate::model::reqfile::Reqfile;
use crate::util::traits::ReqVecExt;
use crate::model::req::{Requirement, Timing};
use crate::util::reqtree::ReqTree;
use crate::error::{Result, DeepError};
use crate::model::{opt::OptionalGroup};
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::{alt, eof, separated};
use winnow::prelude::*;

use super::req::{identifier, requirement};

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
    Optional { base: BaseReqfileLine, weight: i64 }
}

impl ReqfileLine {
    pub fn base(&self) -> &BaseReqfileLine {
        match self {
            ReqfileLine::Unspecified(base)
            | ReqfileLine::ForceRequired(base) 
            | ReqfileLine::Optional { base, .. } => base,
        }
    }

    pub fn base_mut(&mut self) -> &mut BaseReqfileLine {
        match self {
            ReqfileLine::Unspecified(base)
            | ReqfileLine::ForceRequired(base) 
            | ReqfileLine::Optional { base, .. } => base,
        }
    }

    pub fn is_explicit_optional(&self) -> bool {
        match self {
            ReqfileLine::Optional { .. } => true,
            _ => false
        }
    }
}

fn parse_reqfile_line(input: &str) -> std::result::Result<ReqfileLine, String> {
    let input = input.trim();
    reqfile_line
        .parse(&input)
        .map_err(|e| format!("Parse error: {}", e))
}

fn reqfile_line(input: &mut &str) -> ModalResult<ReqfileLine> {
    let _ = multispace0.parse_next(input)?;
    alt((
        optional_line,
        force_required_line,
        base_reqfile_line.map(ReqfileLine::Unspecified),
    ))
    .parse_next(input)
}

// optional_line = weight ';' base_reqfile_line
fn optional_line(input: &mut &str) -> ModalResult<ReqfileLine> {
    let weight = 
        digit1.try_map(|s: &str| s.parse::<i64>())
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

    Ok(BaseReqfileLine::DependencyWithIdentifier {
        prereqs,
        dependent,
    })
}

struct ParsedLine {
    rf_line: ReqfileLine,
    line_num: usize,
    timing: Timing
}

struct ReqfileIndex {
    named: HashMap<String, usize>,
    str_to_idx: HashMap<String, usize>,
    dependency_statements: Vec<(Vec<String>, String, u64)>,
}

fn build_index(lines: &[ParsedLine]) -> Result<ReqfileIndex> {
    let mut named: HashMap<String, usize> = HashMap::new();
    let mut dependency_statements: Vec<(Vec<String>, String, u64)> = vec![];

    let str_to_idx: HashMap<String, usize> = lines.iter().enumerate()
        .filter_map(|(i, l)| {
        match l.rf_line.base() {
            BaseReqfileLine::Requirement(req)
                => Some((req.name_or_default(), i)),
            _ => None
        }
    }).collect();

    for (vec_idx, line) in lines.iter().enumerate() {
        let base = line.rf_line.base();

        match base {
            BaseReqfileLine::DependencyWithIdentifier { prereqs, dependent }
            => {
                // TODO! DependencyWithId should actually be a top level enum variant.
                // since its not affected by required, forced, unmarked semantics
                // so yea for now we error if the user misuses the api (FOR NOW)
                if let ReqfileLine::Unspecified(_) = &line.rf_line {

                } else {
                    return Err(DeepError::Reqfile {
                        line: line.line_num,
                        message: "Optional annotations '+' or ';' must be used \
                        at the requirement definition, not in a dependency statement, unless \
                        the definition is in the dependency statement itself.".into()
                    })
                };

                dependency_statements.push(
                    (prereqs.clone(), dependent.clone(), line.line_num as u64)
                );
            },
            BaseReqfileLine::Requirement(req) => {
                if let Some(name) = &req.name {
                    if named.insert(name.clone(), vec_idx).is_some() {
                        return Err(DeepError::Reqfile {
                            line: (line.line_num + 1) as usize,
                            message: format!("Duplicate identifier: {}", name),
                        });
                    }
                }
            }
        };
    }

    Ok(ReqfileIndex { named, str_to_idx, dependency_statements })
}

fn validate_no_ambiguous_anonymous(lines: &[ParsedLine]) -> Result<()> {
    for line in lines {
        let base = line.rf_line.base();

        if let BaseReqfileLine::Requirement(req) = base {
            // only lf anon reqs
            if req.name.is_some() { continue }

            let other_anon = lines.iter().map(|line| line.rf_line.base())
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
                    )
                })
            }
        }
    }

    Ok(())
}

fn resolve_dependencies(lines: &mut [ParsedLine], index: &ReqfileIndex) -> Result<()> {
    for (prereqs, name, line_num) in &index.dependency_statements {
        match index.named.get(name) {
            Some(vec_idx) => {
                for prereq in prereqs {
                    if !index.named.contains_key(prereq) {
                        return Err(DeepError::Reqfile {
                            line: *line_num as usize,
                            message: format!("Prerequisite: no variable named '{name}'.")
                        })
                    }
                }

                let line = &mut lines[*vec_idx];

                let base: &mut BaseReqfileLine = line.rf_line.base_mut();

                match base {
                    BaseReqfileLine::Requirement(req) => {
                        if !req.prereqs.is_empty() {
                            return Err(DeepError::Reqfile {
                                line: *line_num as usize,
                                message: format!("'{name}' has multiple prerequisite assignments.")
                            })
                        }

                        req.prereqs = prereqs.clone();
                    },
                    _ => {}
                };
            },
            None => {
                return Err(DeepError::Reqfile {
                    line: *line_num as usize,
                    message: format!("Dependent: no variable named '{name}'.")
                })
            }
        }
    }

    Ok(())
}

fn build_req_tree(lines: &[ParsedLine]) -> ReqTree {
    let mut tree = ReqTree::new();

    for line in lines {
        if let BaseReqfileLine::Requirement(req) = line.rf_line.base() {
            tree.insert(req.clone());
        }
    }

    tree
}

fn validate_tree(
    lines: &[ParsedLine],
    tree: &ReqTree,
    str_to_idx: &HashMap<String, usize>
) -> Result<()> {
    if let Some(cycle) = tree.find_cycle() {
        return Err(DeepError::Reqfile {
            line: 0,
            message: format!(
                "Prereqs cannot be dependent on each other. Found cycle: {}",
                cycle.join(" => ")
            )
        })
    }

    // a required req cannot have an optional prereq
    for line in lines {
        match &line.rf_line {
            ReqfileLine::Optional { base, .. } => {
                if let BaseReqfileLine::Requirement(req) = base {
                    if let Some(name) = &req.name {
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
                                        name,
                                        dependent,
                                        dependent_line.line_num,
                                        dependent
                                    )
                                })
                            }
                        }
                    }
                }
            },
            _ => {}
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
        match &line.rf_line {
            ReqfileLine::Optional { base, weight } => {
                if let BaseReqfileLine::Requirement(req) = base {
                    let mut group = OptionalGroup {
                        general: HashSet::new(),
                        post: HashSet::new(),
                        weight: *weight,
                    };

                    for req in tree
                        .all_prereqs(&req.name_or_default())
                        .iter().chain(&[req.name_or_default()]) {

                        let vec_idx = str_to_idx[req];
                        let req_line = &lines[vec_idx];

                        if let BaseReqfileLine::Requirement(req) = req_line.rf_line.base() {
                            group.get_set(req_line.timing).insert(req.clone());
                        }

                        marked_opt.insert(req.clone());
                    }

                    optional.push(group)
                }
            },
            _ => {}
        }
    }

    (optional, marked_opt)
}

fn apply_force_required(
    lines: &[ParsedLine],
    tree: &ReqTree,
    str_to_idx: &HashMap<String, usize>,
    optional: &mut Vec<OptionalGroup>,
    marked_opt: &mut HashSet<String>,
) {
    for line in lines {
        match &line.rf_line {
            ReqfileLine::ForceRequired(base) => {
                if let BaseReqfileLine::Requirement(req) = base {
                    for req in tree
                        .all_prereqs(&req.name_or_default())
                        .iter().chain(&[req.name_or_default()]) {

                        let vec_idx = str_to_idx[req];
                        let req_line = &lines[vec_idx];

                        if let BaseReqfileLine::Requirement(req) = req_line.rf_line.base() {
                            for group in optional.iter_mut() {
                                group.get_set(req_line.timing).remove(req);
                            }
                        }

                        marked_opt.remove(req);
                    }
                }
            },
            _ => {}
        }
    }
}

fn collect_required_reqs(
    lines: &[ParsedLine],
    marked_opt: &HashSet<String>
) -> (Vec<Requirement>, Vec<Requirement>) {
    let mut general: Vec<Requirement> = vec![];
    let mut post: Vec<Requirement> = vec![];

    for line in lines {
        let base = line.rf_line.base();
        if let BaseReqfileLine::Requirement(req) = base {
            if marked_opt.contains(&req.name_or_default()) { continue }

            match line.timing {
                Timing::Free => general.push(req.clone()),
                Timing::Post => post.push(req.clone()),
            }
        }
    }

    (general, post)
}

fn validate_and_transform(mut lines: Vec<ParsedLine>) -> Result<Reqfile> {
    let index = build_index(&lines)?;
    validate_no_ambiguous_anonymous(&lines)?;
    resolve_dependencies(&mut lines, &index)?;

    let tree = build_req_tree(&lines);
    validate_tree(&lines, &tree, &index.str_to_idx)?;

    let (mut optional, mut marked_opt) =
        build_optional_groups(&lines, &tree, &index.str_to_idx);
    apply_force_required(&lines, &tree, &index.str_to_idx, &mut optional, &mut marked_opt);

    let (general, post) = collect_required_reqs(&lines, &marked_opt);

    Ok(Reqfile { general, post, optional })
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

        let parsed = parse_reqfile_line(&line).map_err(|e| DeepError::Reqfile {
            line: i + 1,
            message: e.to_string(),
        })?;

        lines.push(ParsedLine {
            rf_line: parsed, 
            line_num: i, 
            timing: current 
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
        name.replace(" ", "_")
            .replace("[", "")
            .replace("]", "")
            .replace("'", "")
            .replace(":", "")
            .replace("(", "")
            .replace(")", "")
    };

    let mut i = 0;

    let mut general = payload
        .general
        .iter()
        .map(|req: &Requirement| {
            i += 1;

            let mut req = req.clone();

            req.name = req.name.clone().or_else(|| {
                if !req.prereqs.is_empty() {
                    Some(format!("id_{}", i))
                } else {
                    None
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
                if !req.prereqs.is_empty() {
                    Some(format!("id_{}", i))
                } else {
                    None
                }
            });

            req
        })
        .collect::<Vec<_>>();

    general.map_names(clean_name);

    post.map_names(clean_name);

    for req in &general {
        output.push_str(&format!("{}\n", req));
    }

    if !post.is_empty() {
        output.push_str("\nPost:\n");

        for req in &post {
            output.push_str(&format!("{}\n", req));
        }
    }

    output
}
