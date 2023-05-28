use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::{collections::HashMap, fs::File, io::{stdout, stdin}};

use serde::Deserialize;

use crate::{render::write_section, extract::Change};

fn get_versions() -> Result<Vec<String>> {
    #[derive(Debug, Deserialize)]
    struct Version {
        id: String,
        #[serde(rename = "type")]
        version_type: String,
    }

    #[derive(Debug, Deserialize)]
    struct VersionManifest {
        versions: Vec<Version>,
    }

    let res =
        reqwest::blocking::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")?
            .json::<VersionManifest>()?
            .versions
            .into_iter()
            .filter(|v| v.version_type == "release")
            .map(|v| {
                if v.id.chars().filter(|&ch| ch == '.').count() == 1 {
                    format!("{}.0", v.id)
                } else {
                    v.id
                }
            })
            .collect();
    Ok(res)
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Input changelog JSON file. Can be '-' for stdin.
    changelog: String,
    /// JSON template file with predefined values etc.
    /// The result of the changelog gets merged over the object from the template.
    template: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    let changes: Vec<Change> = match args.changelog.as_ref() {
        "-" => serde_json::from_reader(stdin())?,
        name => serde_json::from_reader(File::open(name)?)?,
    };
    let mut result = match args.template.filter(|t| !t.is_empty()) {
        Some(template) => serde_json::from_reader(File::open(template)?)?,
        None => Map::new(),
    };

    let mc_versions = get_versions()?;

    let mut recommended = HashMap::new();
    let mut latest = HashMap::new();

    for change in changes {
        if !change.is_release {
            continue;
        }

        let mut versions = change
            .name
            .strip_prefix('v')
            .unwrap_or(&change.name)
            .split('-');
        let err_msg = "Expected tags to be of the form v{MC}-{MOD}";
        let mc_version = versions.next().context(err_msg)?;
        let mod_version = versions.next().context(err_msg)?;

        let suffix = versions.next(); // aplha, beta, rc, etc.

        for version in &mc_versions {
            if !version.starts_with(mc_version) {
                continue;
            }
            let version = version.strip_suffix(".0").unwrap_or(version);

            // this works because changes are sorted newest to oldest
            latest
                .entry(version)
                .or_insert_with(|| mod_version.to_owned());
            if suffix.is_none() {
                recommended
                    .entry(version)
                    .or_insert_with(|| mod_version.to_owned());
            }

            result
                .entry(version)
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .context("Template version value was not an object")?
                .insert(mod_version.to_owned(), {
                    let mut section = String::new();
                    write_section(&mut section, &change, "")?;
                    Value::String(section)
                });
        }
    }

    let promos = result
        .entry("promos")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .context("Template promos was not an object")?;

    for (mc_version, mod_version) in recommended {
        promos.insert(format!("{mc_version}-recommended"), Value::String(mod_version.clone()));
    }
    for (mc_version, mod_version) in latest {
        promos.insert(format!("{mc_version}-latest"), Value::String(mod_version));
    }

    serde_json::to_writer(stdout(), &result)?;

    Ok(())
}
