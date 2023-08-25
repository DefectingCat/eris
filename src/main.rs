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
    let target_directory = &args.directory;

    let sisyphus = Sisyphus::new(target_directory)?;
    sisyphus.process()?;
    Ok(())
}
