use anyhow::Result;
use clap::Parser;

use sisyphus::Sisyphus;

use crate::args::Args;

mod args;
mod consts;
mod sisyphus;
mod ziper;

fn main() -> Result<()> {
    let args = Args::parse();

    let sisyphus = Sisyphus::new(&args.target_directory, &args.output)?;
    sisyphus.process()?;
    Ok(())
}
