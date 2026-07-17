use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{DeepError, Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WikiMeta {
    pub source: String,
    pub license: String,
    pub pages: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WikiNode {
    pub title: Option<String>,
    pub revid: Option<u64>,
    pub source: Option<String>,
    pub file: Option<String>,
    pub categories: Vec<String>,
    pub notices: Vec<String>,
    pub content: Option<String>,
    pub children: Vec<String>,
}

/// A struct mirroring the structure of the 'wiki.json'
/// bundle found on [pocamind/deepwoken-wiki releases](https://github.com/pocamind/deepwoken-wiki/releases).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeepWiki {
    meta: WikiMeta,
    nodes: BTreeMap<String, WikiNode>,
}

impl DeepWiki {
    pub fn from_json(json: &str) -> Result<DeepWiki> {
        serde_json::from_str(json).map_err(DeepError::from)
    }

    #[must_use]
    pub fn meta(&self) -> &WikiMeta {
        &self.meta
    }

    #[must_use]
    pub fn nodes(&self) -> &BTreeMap<String, WikiNode> {
        &self.nodes
    }

    /// Read a 'path' on the wiki, similar to navigating the wiki via routes.
    /// Paths are of the format "segment1/segment2", and 
    /// can be read with the natural name, e.g. "Deep Shrines/Shrine of Order" (spaces dots and colons allowed) 
    #[must_use]
    pub fn read(&self, path: &str) -> Option<&WikiNode> {
        let path = path.trim_matches('/');
        if let Some(node) = self.nodes.get(path) {
            return Some(node);
        }
        self.nodes
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(path))
            .map(|(_, node)| node)
    }
}
