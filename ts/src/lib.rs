use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use deepwoken_rs::Stat;
use deepwoken_rs::data::DeepData;
use deepwoken_rs::util::statmap::StatMap;
use deepwoken_rs::util::algos;

#[wasm_bindgen(js_name = "DeepData")]
pub struct JsDeepData {
    inner: DeepData,
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_class = "DeepData")]
impl JsDeepData {
    /// Fetch the latest data bundle from pocamind/data on GitHub
    #[wasm_bindgen(js_name = "fetchLatest")]
    pub async fn fetch_latest() -> Result<JsDeepData, JsError> {
        let release = DeepData::latest_release().await
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data = DeepData::from_release(&release).await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsDeepData { inner: data })
    }

    /// Fetch the latest data bundle from a fork
    #[wasm_bindgen(js_name = "fetchLatestFrom")]
    pub async fn fetch_latest_from(owner: &str, repo: &str) -> Result<JsDeepData, JsError> {
        let release = DeepData::latest_release_from(owner, repo).await
            .map_err(|e| JsError::new(&e.to_string()))?;
        let data = DeepData::from_release(&release).await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsDeepData { inner: data })
    }

    /// Parse data from a JSON string
    #[wasm_bindgen(js_name = "fromJson")]
    pub fn from_json(json: &str) -> Result<JsDeepData, JsError> {
        let data = DeepData::from_json(json)
            .map_err(|e| JsError::new(&e.to_string()))?;
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

    #[wasm_bindgen(js_name = "getAspect")]
    pub fn get_aspect(&self, name: &str) -> Result<JsValue, JsError> {
        to_js(&self.inner.get_aspect(name))
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

    pub fn aspects(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner.aspects().collect::<Vec<_>>())
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
        let map: HashMap<Stat, i64> = serde_wasm_bindgen::from_value(map)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsStatMap { inner: StatMap::from(map) })
    }

    pub fn cost(&self) -> i32 {
        self.inner.cost() as i32
    }

    pub fn remaining(&self) -> i32 {
        self.inner.remaining() as i32
    }

    pub fn level(&self) -> i32 {
        self.inner.level() as i32
    }

    pub fn get(&self, stat: &str) -> Result<i32, JsError> {
        let stat: Stat = stat.parse()
            .map_err(|e: &str| JsError::new(e))?;
        Ok(self.inner.get(&stat) as i32)
    }

    pub fn set(&mut self, stat: &str, value: i32) -> Result<(), JsError> {
        let stat: Stat = stat.parse()
            .map_err(|e: &str| JsError::new(e))?;
        self.inner.insert(stat, value as i64);
        Ok(())
    }

    #[wasm_bindgen(js_name = "toJSON")]
    pub fn to_json(&self) -> Result<JsValue, JsError> {
        to_js(&self.inner)
    }

    #[wasm_bindgen(js_name = "shrineOrder")]
    pub fn shrine_order(&self, racial: &JsStatMap) -> JsStatMap {
        JsStatMap { inner: algos::shrine_order_dwb(&self.inner, &racial.inner) }
    }
}

#[wasm_bindgen(js_name = "shrineOrderDwb")]
pub fn shrine_order_dwb(pre: &JsStatMap, racial: &JsStatMap) -> JsStatMap {
    JsStatMap { inner: algos::shrine_order_dwb(&pre.inner, &racial.inner) }
}

