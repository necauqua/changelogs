use std::fmt::Write;

use anyhow::Result;

use extract::Changeset;

pub use extract;

pub fn write_section(result: &mut String, changeset: &Changeset) -> Result<()> {
    for section in &changeset.sections {
        let mut name = section.name.clone();
        if !name.is_empty() {
            name[..1].make_ascii_uppercase();
        }
        writeln!(result, "### {}", name)?;
        for change in &section.changes {
            writeln!(result, "  - {change}")?;
        }
    }
    Ok(())
}
