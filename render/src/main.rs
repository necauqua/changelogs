use std::{fmt::Write, fs::File};

use anyhow::{Context, Result};

use extract::{Changelog, Changeset};
use render::write_section;

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
    changeset: &Changeset,
    tag: Option<&str>,
) -> Result<()> {
    if !changeset.sections.is_empty() {
        writeln!(
            result,
            "## {}",
            format_header(header_template, date_format, changeset, tag)?
        )?;
        write_section(result, changeset)?;
        writeln!(result)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _exe = args.next().unwrap();
    let changelog_file = args.next().unwrap();
    let tag_format = args.next().unwrap();
    let date_format = args.next().unwrap();
    let unreleased_header = args.next().unwrap();
    let filename = args.next().unwrap();
    let only_last = args.next().unwrap() == "true";
    let template_file = args.next();

    let data: Changelog = serde_json::from_reader(File::open(changelog_file)?)?;

    let mut result = String::with_capacity(64 * 1024);

    if only_last {
        if !data.unreleased.sections.is_empty() {
            write_section(&mut result, &data.unreleased)?;
        } else if let Some(release) = data.releases.first() {
            write_section(&mut result, &release.log)?;
        }
        std::fs::write(filename, result)?;
        return Ok(());
    }

    write_full_section(
        &mut result,
        &unreleased_header,
        &date_format,
        &data.unreleased,
        None,
    )?;

    for release in data.releases {
        write_full_section(
            &mut result,
            &tag_format,
            &date_format,
            &release.log,
            Some(&release.tag),
        )?;
    }

    std::fs::write(
        filename,
        match template_file {
            None => result,
            Some(template_file) => {
                let template = std::fs::read_to_string(template_file)?;
                template.replace("{changelog}", &result)
            }
        },
    )?;

    Ok(())
}