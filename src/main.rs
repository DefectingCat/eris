use anyhow::Result;
use std::path::PathBuf;

use ziper::Ziper;

mod ziper;

fn main() -> Result<()> {
    let path = PathBuf::from("./test/test.zip");
    let mut ziper = Ziper::new(&path)?;
    ziper.unzip(Some("./test"))?;

    Ok(())
}
