use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env::Args, fs::File, iter::once, process::Command};

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

const HEAD: &str = "HEAD";

fn git(args: &[&str]) -> Result<Vec<String>> {
    let stdout = Command::new("git")
        .args(args)
        .output()
        .context("Failed to run git")?
        .stdout;
    if stdout.is_empty() {
        Ok(vec![])
    } else {
        Ok(String::from_utf8(stdout)?
            .trim()
            .split('\n')
            .map(ToOwned::to_owned)
            .collect())
    }
}

fn get_commit_timestamp(refname: &str) -> Result<i64> {
    Ok(git(&[
        "show",
        "-s",
        "--format=%ct",
        &format!("{refname}^{{commit}}"),
    ])?
    .get(0)
    .context("Failed to get git commit timestamp")?
    .parse()?)
}

fn get_commit_hash(refname: &str) -> Result<String> {
    git(&["rev-list", "-n1", refname])?
        .into_iter()
        .next()
        .context("Failed to get commit hash")
}

fn get_and_merge_sections(start_ref: Option<&str>, end_ref: Option<&str>) -> Result<Vec<Section>> {
    let mut cmd = vec!["log", "--reverse", "--format=%b"];
    cmd.extend(start_ref);
    let end_ref = end_ref.map(|end_ref| format!("^{end_ref}"));
    cmd.extend(end_ref.as_deref());

    let mut log = BTreeMap::<_, Vec<String>>::new();
    let mut current = None;

    for line in git(&cmd)? {
        if let Some(line) = line.strip_suffix(':') {
            current = Some(log.entry(line.to_lowercase()).or_default());
        } else if let Some(line) = line.strip_prefix("  - ") {
            current.as_mut()
                .with_context(|| format!("Malformed commit changelog, a section entry without defined section: {line}"))?
                .push(line.to_owned());
        }
    }
    Ok(log
        .into_iter()
        .map(|(name, changes)| Section { name, changes })
        .collect())
}

fn load_changelog(root_commit: Option<String>) -> Result<Changelog> {
    let tags_with_dates: Vec<_> = git(&{
        let mut cmd = vec![
            "for-each-ref",
            "--merged",
            HEAD,
            "--sort=-creatordate",
            "--format",
            "%(refname)|%(creatordate:unix)",
            "refs/tags",
        ];
        for root_commit in &root_commit {
            cmd.extend(["--contains", root_commit]);
        }
        cmd
    })?
    .into_iter()
    .map(|line| {
        let err_msg = "expected git to return info in our format";
        let mut parts = line.split('|');
        let refname = parts.next().expect(err_msg);
        let creatordate: i64 = parts
            .next()
            .and_then(|line| line.parse().ok())
            .expect(err_msg);
        (refname.to_owned(), creatordate)
    })
    .collect();

    match tags_with_dates.first() {
        None => {
            // no tags -> everything is unreleased
            Ok(Changelog {
                unreleased: Changeset {
                    timestamp: get_commit_timestamp(HEAD)?,
                    hash: get_commit_hash(HEAD)?,
                    sections: get_and_merge_sections(Some(HEAD), root_commit.as_deref())?,
                },
                releases: vec![],
            })
        }
        Some((last_tag, _)) => {
            let unreleased = Changeset {
                hash: get_commit_hash(HEAD)?,
                timestamp: get_commit_timestamp(HEAD)?,
                sections: get_and_merge_sections(Some(HEAD), Some(last_tag))?,
            };

            // we need an explicit root so that
            // we can always pair it with the last
            // tag in the iter concatenation below
            let root = match root_commit {
                Some(root_commit) => root_commit,
                None => git(&["rev-list", "--max-parents=0", HEAD])?
                    .into_iter()
                    .next()
                    .context("Failed to get the root commit of the repository")?,
            };

            let mut releases = vec![];

            let end_refs = tags_with_dates
                .iter()
                .skip(1)
                .map(|(next_ref, _)| next_ref)
                .chain(once(&root));

            for ((tag, timestamp), next_ref) in tags_with_dates.iter().zip(end_refs) {
                let sections = get_and_merge_sections(Some(tag), Some(next_ref))?;
                if sections.is_empty() {
                    continue;
                }
                let log = Changeset {
                    hash: get_commit_hash(tag)?,
                    timestamp: *timestamp,
                    sections,
                };
                let tag = tag.strip_prefix("refs/tags/").unwrap_or(tag).to_owned();
                releases.push(Release { tag, log });
            }

            Ok(Changelog {
                unreleased,
                releases,
            })
        }
    }
}

pub fn run(mut args: Args) -> Result<()> {
    let filename = args.next().unwrap();
    let root_commit = args.next().unwrap();
    let root_commit = match &*root_commit { // ugh
        "" => None,
        _ => Some(root_commit),
    };

    serde_json::to_writer(File::create(filename)?, &load_changelog(root_commit)?)?;

    Ok(())
}
