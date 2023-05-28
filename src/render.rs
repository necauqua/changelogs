use crate::extract::Change;
use anyhow::{Context, Result};
use clap::ArgAction;
use lazy_regex::regex_replace_all;
use std::{fmt::Write, fs::File, io::stdin};

pub fn write_section(result: &mut String, change: &Change, issue_format: &str) -> Result<()> {
    for section in &change.sections {
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
    changeset: &Change,
) -> Result<String> {
    let timestamp = chrono::NaiveDateTime::from_timestamp_opt(changeset.timestamp, 0)
        .context("Invalid timestamp")?;

    let header = header_template
        .replace("{hash}", &changeset.hash)
        .replace("{name}", &changeset.name)
        .replace("{timestamp}", &timestamp.format(date_format).to_string())
        .replace("{name}", &changeset.name);

    Ok(header)
}

fn write_full_section(
    result: &mut String,
    args: &Args,
    changeset: &Change,
) -> Result<()> {
    if changeset.sections.is_empty() {
        return Ok(());
    }
    let header_format = match changeset.is_release {
        true => &args.tag_format,
        false => &args.unreleased_header,
    };
    let header = format_header(header_format, &args.date_format, changeset)?;
    writeln!(result, "## {header}")?;
    write_section(result, changeset, &args.issue_format)?;
    writeln!(result)?;
    Ok(())
}

#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Input changelog JSON file. Can be '-' for stdin.
    changelog: String,
    /// Format of the release header.
    tag_format: String,
    /// Format of the date used in tag format.
    date_format: String,
    /// If non-empty, replaces all occurences of `#<number>` with a link with
    /// given format with {number} placeholder replaced by the that number.
    issue_format: String,
    /// Header for the unreleased section.
    unreleased_header: String,
    /// Only write the last release.
    #[clap(action = ArgAction::Set)] // positional bool, kek lol arbidol
    only_last: bool,
    /// A template file to be used as a base.
    /// The rendered file will look like this file with {changelog} placeholder
    /// with the rendered changelog.
    /// Empty string is same as not specifying it (hello GitHub Actions again).
    template: Option<String>,
}

pub fn run(args: Args) -> Result<()> {
    let changes: Vec<Change> = match args.changelog.as_ref() {
        "-" => serde_json::from_reader(stdin())?,
        name => serde_json::from_reader(File::open(name)?)?,
    };

    let mut result = String::with_capacity(64 * 1024); // 64 KiB, idk ¯\_(ツ)_/¯

    if args.only_last {
        if let Some(change) = changes.first() {
            write_section(&mut result, change, &args.issue_format)?;
        }
        print!("{result}");
        return Ok(());
    }

    for change in changes {
        write_full_section(&mut result, &args, &change)?;
    }

    let result = match args.template.filter(|s| !s.is_empty()) {
        Some(template) => std::fs::read_to_string(template)?.replace("{changelog}", &result),
        None => result,
    };
    print!("{result}");

    Ok(())
}
