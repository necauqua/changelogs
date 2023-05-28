use anyhow::{Context, Result};

use git2::Repository;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    eprintln, format,
    io::stdout,
    iter::repeat,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Change {
    pub is_release: bool,
    pub name: String,
    pub hash: String,
    pub timestamp: i64,
    pub sections: Vec<Section>,
}

fn load_changelog(repo: Repository, root: Option<&str>) -> Result<Vec<Change>> {
    // fail early on root revparse
    let root = root
        .map(|r| repo.revparse_single(r).map(|r| r.id()))
        .transpose()?;

    let mut interesting_refs = vec![];
    let mut vtagged_commits = HashSet::new();

    // zips are to avoid checking the refname prefix everytime if we know what
    // iterator yields vtags
    let reference_iter = repo
        .references_glob("refs/heads/main")?
        .zip(repeat(false))
        .chain(
            repo.references_glob("refs/heads/backport/*")?
                .zip(repeat(false)),
        )
        .chain(repo.references_glob("refs/tags/v*")?.zip(repeat(true)));

    // a separate loop to fully populate vtagged_commits set before the next one
    for (reference, is_vtag) in reference_iter {
        let reference = reference?;

        if let Some((name, commit)) = reference.name().zip(reference.peel_to_commit().ok()) {
            let id = commit.id();

            let tag_time = if is_vtag {
                reference
                    .peel_to_tag()
                    .ok() // ignore non-annotated tags or other errors
                    .and_then(|t| t.tagger().map(|t| t.when()))
            } else {
                None
            };
            let time = tag_time.unwrap_or_else(|| commit.time()).seconds();

            vtagged_commits.insert(id);
            interesting_refs.push((id, name.to_owned(), time, is_vtag));
        }
    }

    let mut changes = vec![];

    let mut walker = repo.revwalk()?;

    'outer: for (id, refname, timestamp, is_vtag) in interesting_refs {
        walker.reset()?;

        walker.push(id)?;
        if let Some(root) = root {
            walker.hide(root)?;
        }
        walker.simplify_first_parent()?;

        let mut sections = BTreeMap::<_, Vec<String>>::new();
        let mut current_section = None;

        // walk up the ancestors until we hit another tag
        let mut first = true;
        for ancestor in &mut walker {
            let ancestor = ancestor?;

            if first {
                first = false;

                // forget tagged heads completely, to avoid duplication
                if !is_vtag && vtagged_commits.contains(&ancestor) {
                    continue 'outer;
                }
            } else if vtagged_commits.contains(&ancestor) {
                // finished walking up, not on first as that would've hit itself
                break;
            }

            // and finally add the commit body log to current set
            if let Some(body) = repo.find_commit(ancestor)?.body() {
                for line in body.split('\n') {
                    if let Some(line) = line.strip_suffix(':') {
                        current_section = Some(sections.entry(line.to_lowercase()).or_default());
                    } else if let Some(line) = line.strip_prefix("  - ") {
                        current_section.as_mut()
                            .with_context(|| format!("Malformed commit changelog, a section entry without defined section: {line}"))?
                            .push(line.to_owned());
                    }
                }
            }
        }

        fn short(refname: &str, is_vtag: bool) -> &str {
            let prefix = match is_vtag {
                true => "refs/tags/",
                false => "refs/heads/",
            };
            refname.strip_prefix(prefix).unwrap_or(refname)
        }

        if sections.is_empty() {
            // if there was no commits then the ref was past the explicit root
            if !first {
                eprintln!("skipping empty change {}", short(&refname, is_vtag));
            }
            continue;
        }

        let sections = sections
            .into_iter()
            .map(|(name, changes)| Section { name, changes })
            .collect();

        changes.push(Change {
            is_release: is_vtag,
            name: short(&refname, is_vtag).to_owned(),
            hash: id.to_string(),
            timestamp,
            sections,
        });
    }

    changes.sort_unstable_by_key(|change| -change.timestamp);

    Ok(changes)
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// The commit starting from the children of which the log will be
    /// extracted. If not specified, the log will be extracted from the root
    /// commit of the repository. Empty string is treated as no value
    /// specified (a GitHub Actions special).
    root_commit: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    let root = args.root_commit.filter(|c| !c.is_empty());

    let changelog = load_changelog(Repository::open_from_env()?, root.as_deref())?;

    serde_json::to_writer(stdout(), &changelog)?;

    Ok(())
}
