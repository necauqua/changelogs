use anyhow::{bail, Result};

mod extract;
mod gen_forge_updates;
mod render;

fn main() -> Result<()> {
    let mut args = std::env::args();
    let _exe = args.next().unwrap();
    match &*args.next().unwrap() {
        "extract" => extract::run(args),
        "render" => render::run(args),
        "gen-forge-updates" => gen_forge_updates::run(args),
        _ => bail!("Unexpected command"),
    }
}
