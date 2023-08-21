use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

use anyhow::Result;
use zip::ZipArchive;

pub struct Ziper {
    archive: ZipArchive<File>,
}

impl Ziper {
    pub fn new(path: &Path) -> Result<Self> {
        let file = File::open(path)?;

        Ok(Self {
            archive: ZipArchive::new(file)?,
        })
    }

    pub fn unzip(&mut self, prefix: Option<&str>) -> Result<()> {
        for i in 0..self.archive.len() {
            let mut file = self.archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let mut p = if let Some(prefix) = prefix {
                        PathBuf::from(prefix)
                    } else {
                        PathBuf::new()
                    };
                    p.push(path);
                    p
                }
                None => continue,
            };

            if file.name().ends_with('/') {
                println!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath)?;
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }
}
