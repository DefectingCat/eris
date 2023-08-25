use std::path::PathBuf;

use clap::{command, Parser};

/// HTML Template processer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Target directory
    #[arg(short, long, default_value = ".")]
    pub directory: PathBuf,
}
