use std::path::PathBuf;

use clap::{command, Parser, ValueEnum};

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    #[default]
    Format,
    Compress,
    Upload,
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
    /// Compress output directory. default [directory]/output.
    /// Specify target directory when use upload mode
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Upload API base url. default http://183.162.254.169:8086/
    #[arg(long = "url")]
    pub base_url: Option<String>,
    /// Upload API token, it's required in upload mode.
    #[arg(short, long)]
    pub token: Option<String>,
    /// Upload filename prefix. file `A002_GG42_1100X600.zip` if set name then will use name `[name]_GG42`, otherwise will use orginal name `A002_GG42_1100X600.zip` .
    #[arg(short = 'n', long = "name")]
    pub upload_name: Option<String>,
}
