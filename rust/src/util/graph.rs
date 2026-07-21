use std::collections::{HashMap, HashSet, VecDeque};

use crate::req::{PrereqGroup, Requirement};

pub struct PrereqGraph {
    prereqs: HashMap<String, Vec<PrereqGroup>>,
    dependents: HashMap<String, HashSet<String>>,
    nodes: HashSet<String>,
}

impl PrereqGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            prereqs: HashMap::new(),
            dependents: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn insert(&mut self, req: Requirement) {
        let id = req.name_or_default();

        for prereq in req.prereqs.iter().flat_map(PrereqGroup::alternatives) {
            self.dependents
                .entry(prereq.clone())
                .or_default()
                .insert(id.clone());
        }

        self.nodes.insert(id.clone());
        self.prereqs.insert(id, req.prereqs.into_iter().collect());
    }

    pub fn insert_node(&mut self, id: String) {
        self.nodes.insert(id.clone());
        self.prereqs.entry(id).or_default();
    }

    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.nodes.contains(id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &String> {
        self.nodes.iter()
    }

    #[must_use]
    pub fn prereqs(&self, id: &str) -> Option<&Vec<PrereqGroup>> {
        self.prereqs.get(id)
    }

    #[must_use]
    pub fn dependents(&self, id: &str) -> Option<&HashSet<String>> {
        self.dependents.get(id)
    }

    #[must_use]
    pub fn all_prereqs(&self, id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(groups) = self.prereqs.get(id) {
            queue.extend(groups.iter().flat_map(|g| g.alternatives().cloned()));
        }

        while let Some(current) = queue.pop_front() {
            if visited.insert(current.clone())
                && let Some(groups) = self.prereqs.get(&current)
            {
                queue.extend(groups.iter().flat_map(|g| g.alternatives().cloned()));
            }
        }

        visited
    }

    #[must_use]
    pub fn all_dependents(&self, id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(deps) = self.dependents.get(id) {
            queue.extend(deps.iter().cloned());
        }

        while let Some(current) = queue.pop_front() {
            if visited.insert(current.clone())
                && let Some(deps) = self.dependents.get(&current)
            {
                queue.extend(deps.iter().cloned());
            }
        }

        visited
    }

    #[must_use]
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut path = Vec::new();

        for id in self.prereqs.keys() {
            if let Some(cycle) = self.cycle_visit(id, &mut visited, &mut stack, &mut path) {
                return Some(cycle);
            }
        }
        None
    }

    fn cycle_visit(
        &self,
        id: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if stack.contains(id) {
            let idx = path.iter().position(|n| n == id).unwrap();

            return Some(path[idx..].to_vec());
        }
        if visited.contains(id) {
            return None;
        }

        visited.insert(id.to_string());
        stack.insert(id.to_string());
        path.push(id.to_string());

        if let Some(groups) = self.prereqs.get(id) {
            for prereq in groups.iter().flat_map(PrereqGroup::alternatives) {
                if let Some(cycle) = self.cycle_visit(prereq, visited, stack, path) {
                    return Some(cycle);
                }
            }
        }

        stack.remove(id);
        path.pop();
        None
    }
}

impl Default for PrereqGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::DeepData;

    const FIXTURE: &str = r#"{
        "origins": {
            "castaway": { "name": "Castaway", "desc": "", "outfit": "Stranded" },
            "lone_warrior": { "name": "Lone Warrior", "desc": "", "outfit": "Stranded" }
        },
        "resonances": {
            "crazy_slots": { "name": "Crazy Slots", "desc": "", "rarity": "Normal" }
        },
        "talents": {
            "voidwalker_contract": {
                "name": "Voidwalker Contract",
                "desc": "",
                "rarity": "Origin",
                "category": "Origin",
                "reqs": "()",
                "prereqs": ["origin:castaway | origin:lone_warrior"],
                "count_towards_talent_total": false,
                "vaulted": false,
                "voi": false
            },
            "voideye": {
                "name": "Voideye",
                "desc": "",
                "rarity": "Advanced",
                "category": "Void",
                "reqs": "40s WND",
                "prereqs": ["talent:voidwalker_contract"],
                "count_towards_talent_total": true,
                "vaulted": false,
                "voi": false
            }
        }
    }"#;

    #[test]
    fn graph_nodes_cover_reqless_tables() {
        let data = DeepData::from_json(FIXTURE).unwrap();
        let graph = data.prereq_graph();

        assert!(graph.contains("origin:castaway"));
        assert!(graph.contains("origin:lone_warrior"));
        assert!(graph.contains("resonance:crazy_slots"));
        assert!(graph.contains("talent:voidwalker_contract"));
        assert!(graph.contains("talent:voideye"));
        assert!(!graph.contains("talent:nonexistent"));
    }

    #[test]
    fn graph_direct_and_transitive_prereqs() {
        let data = DeepData::from_json(FIXTURE).unwrap();
        let graph = data.prereq_graph();

        let direct = graph.prereqs("talent:voideye").unwrap();
        assert_eq!(direct.len(), 1);

        let all = graph.all_prereqs("talent:voideye");
        assert!(all.contains("talent:voidwalker_contract"));
        assert!(all.contains("origin:castaway"));
        assert!(all.contains("origin:lone_warrior"));
    }

    #[test]
    fn graph_dependents() {
        let data = DeepData::from_json(FIXTURE).unwrap();
        let graph = data.prereq_graph();

        let deps = graph.dependents("origin:castaway").unwrap();
        assert!(deps.contains("talent:voidwalker_contract"));

        let all = graph.all_dependents("origin:castaway");
        assert!(all.contains("talent:voidwalker_contract"));
        assert!(all.contains("talent:voideye"));
    }

    #[test]
    fn graph_no_cycle_on_valid_data() {
        let data = DeepData::from_json(FIXTURE).unwrap();
        let graph = data.prereq_graph();
        assert!(graph.find_cycle().is_none());
    }
}
