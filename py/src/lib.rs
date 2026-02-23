use std::collections::HashMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use pyo3_stub_gen::define_stub_info_gatherer;

use deepwoken_rs::model::opt::OptionalGroup;
use deepwoken_rs::model::req::{Atom, Clause, ClauseType, Reducability, Requirement};
use deepwoken_rs::model::reqfile::Reqfile;
use deepwoken_rs::util::statmap::StatMap;
use deepwoken_rs::{data::DeepData, Stat};

fn to_json<T: serde::Serialize>(v: &T) -> PyResult<String> {
    serde_json::to_string(v).map_err(|e| PyValueError::new_err(e.to_string()))
}

// --- StatMap ---

#[gen_stub_pyclass]
#[pyclass(name = "StatMap")]
pub struct PyStatMap {
    inner: StatMap,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyStatMap {
    #[new]
    pub fn new(map: HashMap<String, i64>) -> PyResult<Self> {
        let mut inner = HashMap::new();
        for (k, v) in map {
            let stat: Stat = k.parse().map_err(|e: &str| PyValueError::new_err(e))?;
            inner.insert(stat, v);
        }
        Ok(PyStatMap { inner: StatMap::from(inner) })
    }

    pub fn cost(&self) -> i64 {
        self.inner.cost()
    }

    pub fn remaining(&self) -> i64 {
        self.inner.remaining()
    }

    pub fn level(&self) -> i64 {
        self.inner.level()
    }

    pub fn get(&self, stat: &str) -> PyResult<i64> {
        let stat: Stat = stat.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        Ok(self.inner.get(&stat))
    }

    pub fn set(&mut self, stat: &str, value: i64) -> PyResult<()> {
        let stat: Stat = stat.parse().map_err(|e: &str| PyValueError::new_err(e))?;
        self.inner.insert(stat, value);
        Ok(())
    }

    pub fn to_json(&self) -> PyResult<String> {
        to_json(&self.inner)
    }
}

// --- Atom ---

#[gen_stub_pyclass]
#[pyclass(name = "Atom")]
#[derive(Clone)]
pub struct PyAtom {
    inner: Atom,
}

impl From<Atom> for PyAtom {
    fn from(inner: Atom) -> Self {
        PyAtom { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyAtom {
    #[getter]
    pub fn value(&self) -> i64 {
        self.inner.value
    }

    /// True if this atom is strict (cannot be reduced by oaths/builds)
    #[getter]
    pub fn strict(&self) -> bool {
        self.inner.reducability == Reducability::Strict
    }

    /// The stat names (full names) that sum to meet the value
    #[getter]
    pub fn stats(&self) -> Vec<String> {
        self.inner.stats.iter().map(|s| s.name().to_string()).collect()
    }

    pub fn satisfied_by(&self, statmap: &PyStatMap) -> bool {
        self.inner.satisfied_by(&statmap.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Atom({})", self.inner)
    }
}

// --- Clause ---

#[gen_stub_pyclass]
#[pyclass(name = "Clause")]
#[derive(Clone)]
pub struct PyClause {
    inner: Clause,
}

impl From<Clause> for PyClause {
    fn from(inner: Clause) -> Self {
        PyClause { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyClause {
    /// True if all atoms must be satisfied (AND clause)
    #[getter]
    pub fn is_and(&self) -> bool {
        self.inner.clause_type == ClauseType::And
    }

    /// True if any atom satisfies the clause (OR clause)
    #[getter]
    pub fn is_or(&self) -> bool {
        self.inner.clause_type == ClauseType::Or
    }

    #[getter]
    pub fn atoms(&self) -> Vec<PyAtom> {
        self.inner.atoms().iter().map(|a| PyAtom::from(a.clone())).collect()
    }

    pub fn satisfied_by(&self, statmap: &PyStatMap) -> bool {
        self.inner.satisfied_by(&statmap.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Clause({})", self.inner)
    }
}

// --- Requirement ---

#[gen_stub_pyclass]
#[pyclass(name = "Requirement")]
#[derive(Clone)]
pub struct PyRequirement {
    inner: Requirement,
}

impl From<Requirement> for PyRequirement {
    fn from(inner: Requirement) -> Self {
        PyRequirement { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyRequirement {
    #[staticmethod]
    pub fn from_str(s: &str) -> PyResult<Self> {
        s.parse::<Requirement>()
            .map(PyRequirement::from)
            .map_err(PyValueError::new_err)
    }

    #[getter]
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    /// DIRECT prerequisites by name (does not include transitive)
    #[getter]
    pub fn prereqs(&self) -> Vec<String> {
        self.inner.prereqs.clone()
    }

    #[getter]
    pub fn clauses(&self) -> Vec<PyClause> {
        self.inner.clauses.iter().map(|c| PyClause::from(c.clone())).collect()
    }

    pub fn satisfied_by(&self, statmap: &PyStatMap) -> bool {
        self.inner.satisfied_by(&statmap.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Flat iterator over all atoms across all clauses
    pub fn atoms(&self) -> Vec<PyAtom> {
        self.inner.atoms().map(|a| PyAtom::from(a.clone())).collect()
    }

    /// All stats referenced in this requirement (sorted, excludes Total)
    pub fn used_stats(&self) -> Vec<String> {
        let mut stats: Vec<String> = self.inner
            .used_stats()
            .iter()
            .map(|s| s.name().to_string())
            .collect();
        stats.sort();
        stats
    }

    pub fn __str__(&self) -> String {
        self.inner.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Requirement({})", self.inner)
    }
}

// --- OptionalGroup ---

#[gen_stub_pyclass]
#[pyclass(name = "OptionalGroup")]
#[derive(Clone)]
pub struct PyOptionalGroup {
    inner: OptionalGroup,
}

impl From<OptionalGroup> for PyOptionalGroup {
    fn from(inner: OptionalGroup) -> Self {
        PyOptionalGroup { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyOptionalGroup {
    /// Requirements acquired freely (before power gate)
    #[getter]
    pub fn general(&self) -> Vec<PyRequirement> {
        self.inner.general.iter().map(|r| PyRequirement::from(r.clone())).collect()
    }

    /// Requirements acquired after a power gate
    #[getter]
    pub fn post(&self) -> Vec<PyRequirement> {
        self.inner.post.iter().map(|r| PyRequirement::from(r.clone())).collect()
    }

    #[getter]
    pub fn weight(&self) -> i64 {
        self.inner.weight
    }
}

// --- Reqfile ---

#[gen_stub_pyclass]
#[pyclass(name = "Reqfile")]
pub struct PyReqfile {
    inner: Reqfile,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyReqfile {
    #[staticmethod]
    pub fn from_str(s: &str) -> PyResult<Self> {
        s.parse::<Reqfile>()
            .map(|inner| PyReqfile { inner })
            .map_err(PyValueError::new_err)
    }

    /// Required requirements acquired freely (before power gate)
    #[getter]
    pub fn general(&self) -> Vec<PyRequirement> {
        self.inner.general.iter().map(|r| PyRequirement::from(r.clone())).collect()
    }

    /// Required requirements acquired after a power gate
    #[getter]
    pub fn post(&self) -> Vec<PyRequirement> {
        self.inner.post.iter().map(|r| PyRequirement::from(r.clone())).collect()
    }

    /// Optional groups — each group is either all acquired or all not
    #[getter]
    pub fn optional(&self) -> Vec<PyOptionalGroup> {
        self.inner.optional.iter().map(|g| PyOptionalGroup::from(g.clone())).collect()
    }

    /// Regenerate the reqfile string from the parsed representation
    pub fn generate(&self) -> String {
        self.inner.generate()
    }
}

// --- DeepData ---

#[gen_stub_pyclass]
#[pyclass(name = "DeepData")]
pub struct PyDeepData {
    inner: DeepData,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyDeepData {
    /// Fetch the latest data bundle from pocamind/data on GitHub
    #[staticmethod]
    pub fn fetch_latest() -> PyResult<Self> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| PyValueError::new_err(e.to_string()))?
            .block_on(async {
                let release = DeepData::latest_release().await
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let data = DeepData::from_release(&release).await
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(PyDeepData { inner: data })
            })
    }

    /// Fetch the latest data bundle from a fork
    #[staticmethod]
    pub fn fetch_latest_from(owner: &str, repo: &str) -> PyResult<Self> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| PyValueError::new_err(e.to_string()))?
            .block_on(async {
                let release = DeepData::latest_release_from(owner, repo).await
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                let data = DeepData::from_release(&release).await
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(PyDeepData { inner: data })
            })
    }

    /// Parse data from a JSON string
    #[staticmethod]
    pub fn from_json(json: &str) -> PyResult<Self> {
        DeepData::from_json(json)
            .map(|inner| PyDeepData { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Returns JSON string or None
    pub fn get_talent(&self, name: &str) -> PyResult<Option<String>> {
        self.inner.get_talent(name).map(to_json).transpose()
    }

    /// Returns JSON string or None
    pub fn get_mantra(&self, name: &str) -> PyResult<Option<String>> {
        self.inner.get_mantra(name).map(to_json).transpose()
    }

    /// Returns JSON string or None
    pub fn get_weapon(&self, name: &str) -> PyResult<Option<String>> {
        self.inner.get_weapon(name).map(to_json).transpose()
    }

    /// Returns JSON string or None
    pub fn get_outfit(&self, name: &str) -> PyResult<Option<String>> {
        self.inner.get_outfit(name).map(to_json).transpose()
    }

    /// Returns JSON string or None
    pub fn get_aspect(&self, name: &str) -> PyResult<Option<String>> {
        self.inner.get_aspect(name).map(to_json).transpose()
    }

    /// Returns JSON array string of all talents
    pub fn talents(&self) -> PyResult<String> {
        to_json(&self.inner.talents().collect::<Vec<_>>())
    }

    /// Returns JSON array string of all mantras
    pub fn mantras(&self) -> PyResult<String> {
        to_json(&self.inner.mantras().collect::<Vec<_>>())
    }

    /// Returns JSON array string of all weapons
    pub fn weapons(&self) -> PyResult<String> {
        to_json(&self.inner.weapons().collect::<Vec<_>>())
    }

    /// Returns JSON array string of all outfits
    pub fn outfits(&self) -> PyResult<String> {
        to_json(&self.inner.outfits().collect::<Vec<_>>())
    }

    /// Returns JSON array string of all aspects
    pub fn aspects(&self) -> PyResult<String> {
        to_json(&self.inner.aspects().collect::<Vec<_>>())
    }
}

#[pymodule]
fn deepwoken(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyStatMap>()?;
    m.add_class::<PyDeepData>()?;
    m.add_class::<PyAtom>()?;
    m.add_class::<PyClause>()?;
    m.add_class::<PyRequirement>()?;
    m.add_class::<PyOptionalGroup>()?;
    m.add_class::<PyReqfile>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
