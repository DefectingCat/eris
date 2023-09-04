use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::DirEntry;
use zip::{write::FileOptions, ZipArchive};

#[derive(Debug)]
pub struct Ziper {}

impl Ziper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn zip_dir<T>(
        &self,
        // Walkdir iter
        iter: &mut dyn Iterator<Item = DirEntry>,
        prefix: &Path,
        // The file writer
        writer: T,
        method: zip::CompressionMethod,
    ) -> Result<()>
    where
        T: Write + Seek,
    {
        let mut zip = zip::ZipWriter::new(writer);
        let options = FileOptions::default()
            .compression_method(method)
            .unix_permissions(0o755);

        let mut buffer = Vec::new();
        for entry in iter {
            let path = entry.path();
            let name = path.strip_prefix(prefix)?;

            // Write file or directory explicitly
            // Some unzip tools unzip files with directory paths correctly, some do not!
            if path.is_file() {
                println!("Adding file {path:?} as {name:?} ...");
                zip.start_file(name.to_string_lossy(), options)?;

                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                println!("Adding dir {path:?} as {name:?} ...");
                zip.add_directory(name.to_string_lossy(), options)?;
            }
        }
        zip.finish()?;
        Ok(())
    }

    pub fn unzip(&self, prefix: Option<&str>, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
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
