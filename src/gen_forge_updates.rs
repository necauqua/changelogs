use anyhow::{Context, Result};
use std::{collections::HashMap, env::Args, fs::File};

use serde::Deserialize;

use crate::{extract::Changelog, render::write_section};

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

pub fn run(mut args: Args) -> Result<()> {
    let changelog_file = args.next().unwrap();
    let filename = args.next().unwrap();
    let template = args.next();

    let mc_versions = get_versions()?;

    let data: Changelog = serde_json::from_reader(File::open(changelog_file)?)?;

    let mut result = match template {
        None => HashMap::<String, HashMap<String, String>>::new(),
        Some(template) => serde_json::from_reader(File::open(template)?)?,
    };

    let mut recommended = HashMap::new();

    for release in data.releases {
        let mut versions = release
            .tag
            .strip_prefix('v')
            .unwrap_or(&release.tag)
            .split('-');
        let err_msg = "Expected tags to be of the form v{MC}-{MOD}";
        let mc_version = versions.next().context(err_msg)?;
        let mod_version = versions.next().context(err_msg)?;
        for version in &mc_versions {
            if !version.starts_with(mc_version) {
                continue;
            }
            let version = version.strip_suffix(".0").unwrap_or(version);

            recommended
                .entry(version)
                .or_insert_with(|| mod_version.to_owned());

            result
                .entry(version.to_owned())
                .or_default()
                .insert(mod_version.to_owned(), {
                    let mut section = String::new();
                    write_section(&mut section, &release.log)?;
                    section
                });
        }
    }

    let promos = result.entry("promos".into()).or_default();

    for (mc_version, mod_version) in recommended {
        promos.insert(format!("{mc_version}-recommended"), mod_version.clone());
        promos.insert(format!("{mc_version}-latest"), mod_version);
    }

    serde_json::to_writer(File::create(filename)?, &result)?;

    Ok(())
}
