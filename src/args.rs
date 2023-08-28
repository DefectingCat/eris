use std::path::PathBuf;

use clap::{command, Parser, ValueEnum};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    #[default]
    Format,
    Compress,
}

/// HTML Template processer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Eris mode
    #[arg(value_enum, default_value_t = Mode::Format)]
    pub mode: Mode,
    /// Target directory
    #[arg(short, long, default_value = ".")]
    pub directory: PathBuf,
    /// Compress output directory. default 'directory'/output
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}
