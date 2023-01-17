use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Changeset {
    pub hash: String,
    pub timestamp: i64,
    pub sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub tag: String,
    #[serde(flatten)]
    pub log: Changeset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Changelog {
    pub unreleased: Changeset,
    pub releases: Vec<Release>,
}
