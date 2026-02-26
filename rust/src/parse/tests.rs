use std::collections::{BTreeSet, HashSet};

use super::reqfile::{gen_reqfile, parse_reqfile_str};

#[test]
fn reqfile_prereqs() {
    let content = r"
        # base reqs
        base := 25 STR
        armor := 90 FTD

        # dep for an inline req
        base => advanced := 50 INT

        # dep for an identifier
        base, armor => upgraded

        # the identifier in question
        upgraded := 75 WLL

        base, armor => 100 CHA
        ";

    let payload = parse_reqfile_str(content).unwrap();

    assert_eq!(payload.general.len(), 5);

    let base = payload
        .general
        .iter()
        .find(|r| r.name == Some("base".to_string()))
        .unwrap();
    assert!(base.prereqs.is_empty());

    let armor = payload
        .general
        .iter()
        .find(|r| r.name == Some("armor".to_string()))
        .unwrap();
    assert!(armor.prereqs.is_empty());

    let advanced = payload
        .general
        .iter()
        .find(|r| r.name == Some("advanced".to_string()))
        .unwrap();
    assert_eq!(advanced.prereqs, BTreeSet::from(["base".to_owned()]));
    let upgraded = payload
        .general
        .iter()
        .find(|r| r.name == Some("upgraded".to_string()))
        .unwrap();
    assert_eq!(
        upgraded.prereqs,
        BTreeSet::from(["base".to_owned(), "armor".to_owned()])
    );

    let anon = payload.general.iter().find(|r| r.name.is_none()).unwrap();
    assert_eq!(
        anon.prereqs,
        BTreeSet::from(["base".to_owned(), "armor".to_owned()])
    );
}

#[test]
fn reqfile_gen_no_optional() {
    let content = r"
        Free:
        crystal := 40 ice
        surge := 40 ltn
        scrapsinger := 35 mtl

        fulgurite_formation := 50 ice, 50 ltn

        90 ltn, 90r hvy
        40 ftd
        25 cha
        25 int

        Post:
        75r hvy
        20r ftd, 20r flm, 20r ltn
        80r flm
        20r mtl

        # prereqs
        crystal, surge => fulgurite_formation
        battleaxe := ()
        ()
        ";

    let payload = parse_reqfile_str(content).unwrap();
    let gen_content = gen_reqfile(&payload);

    let new_payload = parse_reqfile_str(&gen_content).expect(&gen_content);

    // assert the set of requirements are equal
    let a = payload.general.into_iter().collect::<HashSet<_>>();
    let b = new_payload.general.into_iter().collect::<HashSet<_>>();
    assert_eq!(a, b);

    let a = payload.post.into_iter().collect::<HashSet<_>>();
    let b = new_payload.post.into_iter().collect::<HashSet<_>>();
    assert_eq!(a, b);
}

// === Tests involving optional reqs and more complex layouts ===

#[test]
fn optional_basic_parsing() {
    let content = r"
        Free:
        1; exoskeleton := 40 ftd
        neural_overload := 85 int
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // neural_overload should be in general (required)
    assert_eq!(payload.general.len(), 1);
    let neural = payload
        .general
        .iter()
        .find(|r| r.name == Some("neural_overload".to_string()));
    assert!(neural.is_some());

    // exoskeleton should be in an optional group
    assert_eq!(payload.optional.len(), 1);
    let opt_group = &payload.optional[0];
    assert_eq!(opt_group.weight, 1);
    assert_eq!(opt_group.general.len(), 1);

    let exo = opt_group
        .general
        .iter()
        .find(|r| r.name == Some("exoskeleton".to_string()));
    assert!(exo.is_some());
}

#[test]
fn optional_weight_range() {
    // weights 1-20 should all parse
    for w in 1..=20 {
        let content = format!(
            r"
            Free:
            {w}; some_req := 40 ftd
            "
        );

        let payload = parse_reqfile_str(&content).unwrap();
        assert_eq!(payload.optional.len(), 1);
        assert_eq!(payload.optional[0].weight, w);
    }

    // weight 21 should fail
    let content = r"
        Free:
        21; some_req := 40 ftd
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());

    // weight 0 should fail
    let content = r"
        Free:
        0; some_req := 40 ftd
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());
}

#[test]
fn optional_prereq_of_required_is_invalid() {
    // making a prereq optional when its dependent is required should error
    let content = r"
        Free:
        1; optional_prereq := 30 ftd
        required_dependent := 50 int

        optional_prereq => required_dependent
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("optional") || err_msg.contains("dependents are required"));
}

#[test]
fn optional_prereqs_become_optional() {
    // prereqs of an optional req should be recursively marked optional
    let content = r"
        Free:
        p1 := 10 str
        p2 := 20 int
        p3 := 30 ftd

        1; has_prereqs := 42 hvy

        p1, p2, p3 => has_prereqs
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // all reqs should be optional, none in general
    assert!(payload.general.is_empty());
    assert_eq!(payload.optional.len(), 1);

    let group = &payload.optional[0];
    assert_eq!(group.weight, 1);
    // should have 4 reqs: p1, p2, p3, has_prereqs
    assert_eq!(group.general.len(), 4);

    // verify all are present
    let names: HashSet<_> = group
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    assert!(names.contains("p1"));
    assert!(names.contains("p2"));
    assert!(names.contains("p3"));
    assert!(names.contains("has_prereqs"));
}

#[test]
fn optional_force_required_directive() {
    // the + directive should force a prereq back to required
    let content = r"
        Free:
        p1 := 10 str
        + p2 := 20 int
        p3 := 30 ftd

        1; has_prereqs := 42 hvy

        p1, p2, p3 => has_prereqs
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // p2 should be required (in general), not optional
    assert_eq!(payload.general.len(), 1);
    let p2 = payload
        .general
        .iter()
        .find(|r| r.name == Some("p2".to_string()));
    assert!(p2.is_some());

    // the optional group should have p1, p3, and has_prereqs (not p2)
    assert_eq!(payload.optional.len(), 1);
    let group = &payload.optional[0];
    assert_eq!(group.general.len(), 3);

    let opt_names: HashSet<_> = group
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    assert!(opt_names.contains("p1"));
    assert!(opt_names.contains("p3"));
    assert!(opt_names.contains("has_prereqs"));
    assert!(!opt_names.contains("p2"));
}

#[test]
fn optional_inline_prereqs_syntax() {
    // the syntax `1; p1, p2 => 42 hvy` should work
    let content = r"
        Free:
        p1 := 10 str
        p2 := 20 int

        1; p1, p2 => 42 hvy
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // all should be in the optional group
    assert!(payload.general.is_empty());
    assert_eq!(payload.optional.len(), 1);

    let group = &payload.optional[0];
    // p1, p2, and the anonymous req (42 hvy)
    assert_eq!(group.general.len(), 3);
}

#[test]
fn optional_timing_respected() {
    // optional reqs should respect Free/Post timing
    let content = r"
        Free:
        1; free_opt := 40 ftd

        Post:
        2; post_opt := 50 int
        ";

    let payload = parse_reqfile_str(content).unwrap();

    assert_eq!(payload.optional.len(), 2);

    // find the groups by weight
    let free_group = payload.optional.iter().find(|g| g.weight == 1).unwrap();
    let post_group = payload.optional.iter().find(|g| g.weight == 2).unwrap();

    assert_eq!(free_group.general.len(), 1);
    assert!(free_group.post.is_empty());

    assert!(post_group.general.is_empty());
    assert_eq!(post_group.post.len(), 1);
}

#[test]
fn optional_annotation_on_dependency_statement_errors() {
    // optional annotation must be at definition, not dependency statement
    let content = r"
        Free:
        prereq := 10 str
        dependent := 20 int

        1; prereq => dependent
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("definition") || err_msg.contains("dependency statement"));
}

#[test]
fn optional_empty_req_with_prereqs() {
    // empty optional req with prereqs should work (golden_age pattern)
    let content = r"
        Free:
        scrapsinger := 35 mtl
        crystal := 40 ice
        surge := 40 ltn

        1; golden_age := ()

        scrapsinger, crystal, surge => golden_age
        ";

    let payload = parse_reqfile_str(content).unwrap();

    assert!(payload.general.is_empty());
    assert_eq!(payload.optional.len(), 1);

    let group = &payload.optional[0];
    // all 4 reqs should be in the group
    assert_eq!(group.general.len(), 4);
}

#[test]
fn optional_transitive_prereqs() {
    // prereqs of prereqs should also become optional
    let content = r"
        Free:
        root := 10 str
        mid := 20 int
        1; leaf := 30 ftd

        root => mid
        mid => leaf
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // all should be optional
    assert!(payload.general.is_empty());
    assert_eq!(payload.optional.len(), 1);

    let group = &payload.optional[0];
    assert_eq!(group.general.len(), 3);

    let names: HashSet<_> = group
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    assert!(names.contains("root"));
    assert!(names.contains("mid"));
    assert!(names.contains("leaf"));
}

#[test]
fn optional_force_required_transitive() {
    // force required should also force prereqs of that req to be required
    let content = r"
        Free:
        grandparent := 5 cha
        parent := 10 str
        + child := 20 int
        1; grandchild := 30 ftd

        grandparent => parent
        parent => child
        child => grandchild
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // grandparent, parent, child should all be required
    assert_eq!(payload.general.len(), 3);

    let req_names: HashSet<_> = payload
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    assert!(req_names.contains("grandparent"));
    assert!(req_names.contains("parent"));
    assert!(req_names.contains("child"));

    // only grandchild should be optional
    assert_eq!(payload.optional.len(), 1);
    let group = &payload.optional[0];
    assert_eq!(group.general.len(), 1);

    let opt_names: HashSet<_> = group
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    assert!(opt_names.contains("grandchild"));
}

#[test]
fn optional_shared_prereq_duplicated() {
    // when two optional reqs share a prereq, the prereq should be
    // duplicated into both groups
    let content = r"
        Free:
        shared := 10 str

        1; opt_a := 20 int
        2; opt_b := 30 ftd

        shared => opt_a
        shared => opt_b
        ";

    let payload = parse_reqfile_str(content).unwrap();

    // no required reqs
    assert!(payload.general.is_empty());

    // should have 2 optional groups
    assert_eq!(payload.optional.len(), 2);

    // find groups by weight
    let group_a = payload.optional.iter().find(|g| g.weight == 1).unwrap();
    let group_b = payload.optional.iter().find(|g| g.weight == 2).unwrap();

    // each group should have 2 reqs: the shared prereq and the optional itself
    assert_eq!(group_a.general.len(), 2);
    assert_eq!(group_b.general.len(), 2);

    // both groups should contain 'shared'
    let names_a: HashSet<_> = group_a
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    let names_b: HashSet<_> = group_b
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();

    assert!(names_a.contains("shared"));
    assert!(names_a.contains("opt_a"));
    assert!(names_b.contains("shared"));
    assert!(names_b.contains("opt_b"));
}

#[test]
fn optional_shared_prereq() {
    let content = r"
        Free:
        root := 5 cha
        shared := 10 str

        1; opt_a := 20 int
        2; opt_b := 30 ftd

        root => shared
        shared => opt_a
        shared => opt_b
        ";

    let payload = parse_reqfile_str(content).unwrap();

    assert!(payload.general.is_empty());
    assert_eq!(payload.optional.len(), 2);

    let group_a = payload.optional.iter().find(|g| g.weight == 1).unwrap();
    let group_b = payload.optional.iter().find(|g| g.weight == 2).unwrap();

    // each group should have: root, shared, and the leaf (3 total)
    assert_eq!(group_a.general.len(), 3);
    assert_eq!(group_b.general.len(), 3);

    let names_a: HashSet<_> = group_a
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();
    let names_b: HashSet<_> = group_b
        .general
        .iter()
        .filter_map(|r| r.name.clone())
        .collect();

    // both should have root and shared duplicated
    assert!(names_a.contains("root"));
    assert!(names_a.contains("shared"));
    assert!(names_a.contains("opt_a"));

    assert!(names_b.contains("root"));
    assert!(names_b.contains("shared"));
    assert!(names_b.contains("opt_b"));
}

// === Error Case Tests ===

#[test]
fn invalid_dependence_cycle() {
    let content = r"
        Free:
        a := 10 str
        b := 20 int
        c := 30 ftd

        a => b
        b => c
        c => a
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());

    let Err(err) = result else { panic!() };
    let err_msg = err.to_string();
    assert!(err_msg.contains("cycle") || err_msg.contains("Cycle"));
}

#[test]
fn invalid_dependence_cycle_2() {
    // cycle detection should work even with optional reqs
    let content = r"
        Free:
        1; a := 10 str
        b := 20 int

        a => b
        b => a
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());
}

#[test]
fn invalid_annotations_on_deps() {
    // annotations on dependency statement (not definition) should error
    let content = r"
        Free:
        prereq := 10 str
        dependent := 20 int

        + prereq => dependent
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());

    // annotations on dependency statement (not definition) should error
    let content = r"
        Free:
        prereq := 10 str
        dependent := 20 int

        1 ; prereq => dependent
        ";

    let result = parse_reqfile_str(content);
    assert!(result.is_err());
}
