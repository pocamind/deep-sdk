use std::collections::HashMap;

use deepwoken_rs::Stat;
use deepwoken_rs::data::DeepData;
use deepwoken_rs::model::aggregate::{BuildParams, Scenario};
use deepwoken_rs::model::req::Requirement;
use deepwoken_rs::util::aggregate;
use deepwoken_rs::util::graph::PrereqGraph;
use deepwoken_rs::util::statmap::StatMap;
use deepwoken_rs::util::{algos, name_to_identifier};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "DeepData")]
pub struct JsDeepData {
    inner: DeepData,
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true))
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_class = "DeepData")]
impl JsDeepData {
    /// Fetch the latest data bundle from pocamind/data on GitHub
    #[wasm_bindgen(js_name = "fetchLatest")]
    pub async fn fetch_latest() -> Result<JsDeepData, JsError> {
        let release = DeepData::latest_release()
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data = DeepData::from_release(&release)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsDeepData { inner: data })
    }

    /// Fetch the latest data bundle from a fork
    #[wasm_bindgen(js_name = "fetchLatestFrom")]
    pub async fn fetch_latest_from(owner: &str, repo: &str) -> Result<JsDeepData, JsError> {
        let release = DeepData::latest_release_from(owner, repo)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data = DeepData::from_release(&release)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsDeepData { inner: data })
    }

    /// Parse data from a JSON string
    #[wasm_bindgen(js_name = "fromJson")]
    pub fn from_json(json: &str) -> Result<JsDeepData, JsError> {
        let data = DeepData::from_json(json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsDeepData { inner: data })
    }

    #[wasm_bindgen(js_name = "getTalent")]
    pub fn get_talent(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_talent(name))
    }

    #[wasm_bindgen(js_name = "getMantra")]
    pub fn get_mantra(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_mantra(name))
    }

    #[wasm_bindgen(js_name = "getWeapon")]
    pub fn get_weapon(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_weapon(name))
    }

    #[wasm_bindgen(js_name = "getOutfit")]
    pub fn get_outfit(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_outfit(name))
    }

    #[wasm_bindgen(js_name = "getEquipment")]
    pub fn get_equipment(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_equipment(name))
    }

    #[wasm_bindgen(js_name = "getAspect")]
    pub fn get_aspect(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_aspect(name))
    }

    #[wasm_bindgen(js_name = "getEnchant")]
    pub fn get_enchant(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_enchant(name))
    }

    #[wasm_bindgen(js_name = "getPreset")]
    pub fn get_preset(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_preset(name))
    }

    #[wasm_bindgen(js_name = "getOrigin")]
    pub fn get_origin(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_origin(name))
    }

    #[wasm_bindgen(js_name = "getResonance")]
    pub fn get_resonance(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_resonance(name))
    }

    #[wasm_bindgen(js_name = "getObjective")]
    pub fn get_objective(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_objective(name))
    }

    pub fn requirement(&self, id: &str) -> Option<JsRequirement> {
        self.inner.requirement(id).map(|inner| JsRequirement { inner })
    }

    #[wasm_bindgen(js_name = "prereqGraph")]
    pub fn prereq_graph(&self) -> JsPrereqGraph {
        JsPrereqGraph {
            inner: self.inner.prereq_graph(),
        }
    }

    pub fn talents(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.talents().collect::<Vec<_>>())
    }

    pub fn mantras(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.mantras().collect::<Vec<_>>())
    }

    pub fn weapons(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.weapons().collect::<Vec<_>>())
    }

    pub fn outfits(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.outfits().collect::<Vec<_>>())
    }

    pub fn equipment(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.equipment().collect::<Vec<_>>())
    }

    pub fn aspects(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.aspects().collect::<Vec<_>>())
    }

    pub fn enchants(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.enchants().collect::<Vec<_>>())
    }

    pub fn origins(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.origins().collect::<Vec<_>>())
    }

    pub fn resonances(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.resonances().collect::<Vec<_>>())
    }

    pub fn objectives(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.objectives().collect::<Vec<_>>())
    }

    pub fn presets(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.presets().collect::<Vec<_>>())
    }

    #[wasm_bindgen(js_name = "aggregateStats")]
    pub fn aggregate_stats(
        &self,
        snapshot: JsValue,
        scenario: JsValue,
    ) -> Result<JsValue, JsError> {
        let snapshot: BuildParams =
            serde_wasm_bindgen::from_value(snapshot).map_err(|e| JsError::new(&e.to_string()))?;
        let scenario: Scenario = if scenario.is_undefined() || scenario.is_null() {
            Scenario::default()
        } else {
            serde_wasm_bindgen::from_value(scenario).map_err(|e| JsError::new(&e.to_string()))?
        };
        to_js(&aggregate::aggregate(&self.inner, &snapshot, scenario))
    }

    #[wasm_bindgen(js_name = "grantedTalents")]
    pub fn granted_talents(&self, snapshot: JsValue) -> Result<JsValue, JsError> {
        let snapshot: BuildParams =
            serde_wasm_bindgen::from_value(snapshot).map_err(|e| JsError::new(&e.to_string()))?;
        to_js(&aggregate::granted_talents(&self.inner, &snapshot))
    }
}

#[wasm_bindgen(js_name = "PrereqGraph")]
pub struct JsPrereqGraph {
    inner: PrereqGraph,
}

#[wasm_bindgen(js_class = "PrereqGraph")]
impl JsPrereqGraph {
    pub fn contains(&self, id: &str) -> bool {
        self.inner.contains(id)
    }

    pub fn nodes(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.nodes().collect::<Vec<_>>())
    }

    pub fn prereqs(&self, id: &str) -> Result<JsValue, JsError> {
        let groups: Option<Vec<Vec<String>>> = self.inner.prereqs(id).map(|gs| {
            gs.iter()
                .map(|g| g.alternatives().cloned().collect())
                .collect()
        });
        to_js(&groups)
    }

    pub fn dependents(&self, id: &str) -> Result<JsValue, JsError> {
        let deps: Option<Vec<&String>> = self.inner.dependents(id).map(|d| d.iter().collect());
        to_js(&deps)
    }

    #[wasm_bindgen(js_name = "allPrereqs")]
    pub fn all_prereqs(&self, id: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.all_prereqs(id).into_iter().collect::<Vec<_>>())
    }

    #[wasm_bindgen(js_name = "allDependents")]
    pub fn all_dependents(&self, id: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.all_dependents(id).into_iter().collect::<Vec<_>>())
    }

    #[wasm_bindgen(js_name = "findCycle")]
    pub fn find_cycle(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.find_cycle())
    }
}

#[wasm_bindgen(js_name = "StatMap")]
pub struct JsStatMap {
    inner: StatMap,
}

#[wasm_bindgen(js_class = "StatMap")]
impl JsStatMap {
    #[wasm_bindgen(constructor)]
    pub fn new(map: JsValue) -> Result<JsStatMap, JsError> {
        let map: HashMap<Stat, i64> =
            serde_wasm_bindgen::from_value(map).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsStatMap {
            inner: StatMap::from(map),
        })
    }

    pub fn cost(&self) -> i32 {
        self.inner.cost() as i32
    }

    pub fn remaining(&self) -> i32 {
        self.inner.remaining() as i32
    }

    pub fn level(&self, max_level: Option<u32>) -> i32 {
        self.inner.level(max_level) as i32
    }

    pub fn get(&self, stat: &str) -> Result<i32, JsError> {
        let stat: Stat = stat.parse().map_err(|e: &str| JsError::new(e))?;
        Ok(self.inner.get(&stat) as i32)
    }

    pub fn set(&mut self, stat: &str, value: i32) -> Result<(), JsError> {
        let stat: Stat = stat.parse().map_err(|e: &str| JsError::new(e))?;
        self.inner.insert(stat, value as i64);
        Ok(())
    }

    #[wasm_bindgen(js_name = "toJSON")]
    pub fn to_json(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner)
    }

    #[wasm_bindgen(js_name = "shrineOrder")]
    pub fn shrine_order(&self, racial: &JsStatMap) -> JsStatMap {
        JsStatMap {
            inner: algos::shrine_order_dwb(&self.inner, &racial.inner),
        }
    }

    /// The implicit talents granted by this stat map
    #[wasm_bindgen(js_name = "implicitTalents")]
    pub fn implicit_talents(&self, data: &JsDeepData) -> Result<JsValue, JsError> {
        to_js(&self.inner.implicit_talents(&data.inner))
    }
}

#[wasm_bindgen(js_name = "shrineOrderDwb")]
pub fn shrine_order_dwb(pre: &JsStatMap, racial: &JsStatMap) -> JsStatMap {
    JsStatMap {
        inner: algos::shrine_order_dwb(&pre.inner, &racial.inner),
    }
}

/// Transforms the name of things ingame into an identifier/key used in the database
#[wasm_bindgen(js_name = "nameToIdentifier")]
pub fn js_name_to_identifier(name: &str) -> String {
    name_to_identifier(name)
}

#[wasm_bindgen(js_name = "Requirement")]
pub struct JsRequirement {
    inner: Requirement,
}

#[wasm_bindgen(js_class = "Requirement")]
impl JsRequirement {
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<JsRequirement, JsError> {
        let req = Requirement::parse(input).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsRequirement { inner: req })
    }

    #[wasm_bindgen(js_name = "satisfiedBy")]
    pub fn satisfied_by(&self, stats: &JsStatMap) -> bool {
        self.inner.satisfied_by(&stats.inner)
    }

    #[wasm_bindgen(js_name = "isEmpty")]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[wasm_bindgen(js_name = "addToStatAtoms")]
    pub fn add_to_stat_atoms(&mut self, val: i32) {
        self.inner.add_to_stat_atoms(i64::from(val));
    }

    #[wasm_bindgen(js_name = "usedStats")]
    pub fn used_stats(&self) -> Result<JsValue, JsError> {
        let stats: Vec<&str> = self.inner.used_stats().iter().map(Stat::name).collect();
        to_js(&stats)
    }

    pub fn name(&self) -> Option<String> {
        self.inner.name.clone()
    }

    pub fn prereqs(&self) -> Result<JsValue, JsError> {
        let groups: Vec<Vec<String>> = self
            .inner
            .prereqs
            .iter()
            .map(|g| g.alternatives().cloned().collect())
            .collect();
        to_js(&groups)
    }

    pub fn clauses(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.clauses)
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
