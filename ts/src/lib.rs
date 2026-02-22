use wasm_bindgen::prelude::*;
use deepwoken_rs::{Stat, data::DeepData, model::stat};

#[wasm_bindgen(js_name = "DeepData")]
pub struct JsDeepData {
    inner: DeepData,
}

fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "coreStats")]
pub fn core_stats() -> Result<JsValue, JsError> {                                                                                                                                                                                                                                  
    to_js(&stat::CORE)                                                                                                                                                                                                                                                           
}

#[wasm_bindgen(js_name = "weaponStats")]
pub fn weapon_stats() -> Result<JsValue, JsError> {
    to_js(&stat::WEAPON)
}

#[wasm_bindgen(js_name = "attunementStats")]
pub fn attunement_stats() -> Result<JsValue, JsError> {
    to_js(&stat::ATTUNEMENT)
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

