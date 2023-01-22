use crate::extract::{Changelog, Changeset};
use anyhow::{Context, Result};
use lazy_regex::regex_replace_all;
use std::{env::Args, fmt::Write, fs::File};

pub fn write_section(result: &mut String, changeset: &Changeset, issue_format: &str) -> Result<()> {
    for section in &changeset.sections {
        let mut name = section.name.clone();
        if !name.is_empty() {
            name[..1].make_ascii_uppercase();
        }
        writeln!(result, "### {}", name)?;
        for change in &section.changes {
            let change = match issue_format {
                "" => change.into(),
                _ => regex_replace_all!("#(\\d+)", change, |_, n| issue_format
                    .replace("{number}", n)),
            };
            writeln!(result, "  - {change}")?;
        }
    }
    Ok(())
}

fn format_header(
    header_template: &str,
    date_format: &str,
    changeset: &Changeset,
    tag: Option<&str>,
) -> Result<String> {
    let timestamp = chrono::NaiveDateTime::from_timestamp_opt(changeset.timestamp as i64, 0)
        .context("Invalid timestamp")?;

    let mut header = header_template
        .replace("{hash}", &changeset.hash)
        .replace(
            "{short_hash}",
            changeset.hash.get(..7).unwrap_or(&*changeset.hash),
        )
        .replace("{timestamp}", &timestamp.format(date_format).to_string());

    if let Some(tag) = tag {
        header = header.replace("{tag}", tag);
    }

    Ok(header)
}

fn write_full_section(
    result: &mut String,
    header_template: &str,
    date_format: &str,
    issue_format: &str,
    changeset: &Changeset,
    tag: Option<&str>,
) -> Result<()> {
    if !changeset.sections.is_empty() {
        writeln!(
            result,
            "## {}",
            format_header(header_template, date_format, changeset, tag)?
        )?;
        write_section(result, changeset, issue_format)?;
        writeln!(result)?;
    }
    Ok(())
}

pub fn run(mut args: Args) -> Result<()> {
    let changelog_file = args.next().unwrap();
    let tag_format = args.next().unwrap();
    let date_format = args.next().unwrap();
    let issue_format = args.next().unwrap();
    let unreleased_header = args.next().unwrap();
    let filename = args.next().unwrap();
    let only_last = args.next().unwrap() == "true";
    let template = args.next().unwrap();

    let data: Changelog = serde_json::from_reader(File::open(changelog_file)?)?;

    let mut result = String::with_capacity(64 * 1024);

    if only_last {
        if !data.unreleased.sections.is_empty() {
            write_section(&mut result, &data.unreleased, &issue_format)?;
        } else if let Some(release) = data.releases.first() {
            write_section(&mut result, &release.log, &issue_format)?;
        }
        std::fs::write(filename, result)?;
        return Ok(());
    }

    write_full_section(
        &mut result,
        &unreleased_header,
        &date_format,
        &issue_format,
        &data.unreleased,
        None,
    )?;

    for release in data.releases {
        write_full_section(
            &mut result,
            &tag_format,
            &date_format,
            &issue_format,
            &release.log,
            Some(&release.tag),
        )?;
    }

    let result = match &*template {
        "" => result,
        _ => std::fs::read_to_string(template)?.replace("{changelog}", &result),
    };
    std::fs::write(filename, result)?;

    Ok(())
}
