use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::req::Requirement;

pub struct ReqTree {
    // Keyed by name
    reqs: HashMap<String, Requirement>,
    // Stores a set of reqs that depend on the key
    dependents: HashMap<String, HashSet<String>>,
}

impl ReqTree {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reqs: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Insert a requirement
    pub fn insert(&mut self, req: Requirement) {
        let name = req.name_or_default();

        for prereq in &req.prereqs {
            self.dependents
                .entry(prereq.clone())
                .or_default()
                .insert(name.clone());
        }

        self.reqs.insert(name, req);
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Requirement> {
        self.reqs.get(name)
    }

    #[must_use]
    /// Retrieve direct prereqs as names
    pub fn prereqs(&self, name: &str) -> Option<&BTreeSet<String>> {
        self.reqs.get(name).map(|r| &r.prereqs)
    }

    #[must_use]
    /// Retrieve direct dependents as names
    pub fn dependents(&self, name: &str) -> Option<&HashSet<String>> {
        self.dependents.get(name)
    }

    #[must_use]
    /// All transitive prereqs via BFS
    pub fn all_prereqs(&self, name: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(req) = self.reqs.get(name) {
            queue.extend(req.prereqs.iter().cloned());
        }

        while let Some(current) = queue.pop_front() {
            if visited.insert(current.clone())
                && let Some(req) = self.reqs.get(&current)
            {
                queue.extend(req.prereqs.iter().cloned());
            }
        }

        visited
    }

    #[must_use]
    /// All transitive dependents via BFS
    pub fn all_dependents(&self, name: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(deps) = self.dependents.get(name) {
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
    /// Check for any cycles (shoudl be invalid for deep anyways)
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut path = Vec::new();

        for name in self.reqs.keys() {
            if let Some(cycle) = self.cycle_visit(name, &mut visited, &mut stack, &mut path) {
                return Some(cycle);
            }
        }
        None
    }

    fn cycle_visit(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if stack.contains(name) {
            let idx = path.iter().position(|n| n == name).unwrap();

            return Some(path[idx..].to_vec());
        }
        if visited.contains(name) {
            return None;
        }

        visited.insert(name.to_string());
        stack.insert(name.to_string());
        path.push(name.to_string());

        if let Some(req) = self.reqs.get(name) {
            for prereq in &req.prereqs {
                if let Some(cycle) = self.cycle_visit(prereq, visited, stack, path) {
                    return Some(cycle);
                }
            }
        }

        stack.remove(name);
        path.pop();
        None
    }
}

impl Default for ReqTree {
    fn default() -> Self {
        Self::new()
    }
}
