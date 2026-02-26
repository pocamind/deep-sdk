use std::collections::HashSet;

use crate::{model::req::Timing, req::Requirement};

/// Represents a group of requirements that are optional, but will be
/// either all acquired or all not
#[derive(Clone, Default, Debug)]
pub struct OptionalGroup {
    pub general: HashSet<Requirement>,
    pub post: HashSet<Requirement>,

    pub weight: i64,
}

impl OptionalGroup {
    pub fn get_set(&mut self, timing: Timing) -> &mut HashSet<Requirement> {
        match timing {
            Timing::Free => &mut self.general,
            Timing::Post => &mut self.post,
        }
    }
}
