use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    data::DeepData,
    model::aggregate::{BuildParams, EquipmentSelection},
    util::{aggregate::granted_talents, name_to_identifier, statmap::StatMap},
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FactsParams {
    pub race: String,
    pub origin: Option<String>,
    pub oath: Option<String>,
    pub talents: Vec<String>,
    pub equipment: Vec<EquipmentSelection>,
    pub outfit: Option<String>,
    pub stages: Vec<StatMap>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BuildFacts {
    pub picked: Vec<String>,
    pub given: Vec<String>,
    pub granted: Vec<String>,
    pub implicit_stages: HashMap<String, Vec<usize>>,
    pub khan: bool,
}

fn qualify(namespace: &str, name: &str) -> String {
    format!("{namespace}:{}", name_to_identifier(name))
}

#[must_use]
pub fn build_facts(data: &DeepData, params: &FactsParams) -> BuildFacts {
    let mut given = Vec::new();

    if let Some(origin) = params.origin.as_deref().filter(|s| !s.is_empty()) {
        given.push(qualify("origin", origin));
    }
    if !params.race.is_empty() {
        given.push(qualify("aspect", &params.race));
    }
    if let Some(oath) = params.oath.as_deref().filter(|s| !s.is_empty()) {
        given.push(qualify("talent", &format!("Oath: {oath}")));
    }
    if let Some(outfit) = params.outfit.as_deref().filter(|s| !s.is_empty()) {
        given.push(qualify("outfit", outfit));
    }

    let equipment_only = BuildParams {
        race: params.race.clone(),
        equipment: params.equipment.clone(),
        outfit: params.outfit.clone(),
        ..BuildParams::default()
    };

    let granted = granted_talents(data, &equipment_only)
        .iter()
        .map(|name| qualify("talent", name))
        .collect();

    let mut implicit_stages: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, stats) in params.stages.iter().enumerate() {
        for talent in stats.implicit_talents(data) {
            implicit_stages
                .entry(qualify("talent", &talent.name))
                .or_default()
                .push(index);
        }
    }

    BuildFacts {
        picked: params.talents.clone(),
        given,
        granted,
        implicit_stages,
        khan: params.race == "Khan",
    }
}
