use anyhow::Result;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
use zip::ZipArchive;
use ziper::Ziper;

mod ziper;

fn main() -> Result<()> {
    let path = PathBuf::from("./test/test.zip");
    let mut ziper = Ziper::new(&path)?;
    ziper.unzip()?;

    Ok(())
}
