use anyhow::Result;
use clap::Parser;

mod extract;
mod gen_forge_updates;
mod render;

#[derive(Debug, clap::Parser)]
#[command(author, version, about)]
enum Command {
    /// Extracts a changelog from a git repository into a shared JSON format.
    Extract(extract::Args),
    /// Renders extracted JSON changelog into a Keep a Changelog markdown.
    Render(render::Args),
    /// Generates a Forge updates file from the extracted changelog JSON.
    GenForgeUpdates(gen_forge_updates::Args),
}

fn main() -> Result<()> {
    use Command::*;

    match Command::parse() {
        Extract(args) => extract::run(args),
        Render(args) => render::run(args),
        GenForgeUpdates(args) => gen_forge_updates::run(args),
    }
}
