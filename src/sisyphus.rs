use anyhow::{anyhow, Result};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use crate::ziper::Ziper;

#[derive(Debug)]
pub struct Sisyphus {
    /// Target directory
    pub directory: PathBuf,
    // Target zip files
    file_list: Vec<PathBuf>,
}

impl Sisyphus {
    /// Sisyphus builder.
    ///
    /// Create new Sisyphus struct.
    ///
    /// - `directory`: target diretory. Sisyphus will read all zip file in this directory.
    pub fn new(directory: &PathBuf) -> Result<Self> {
        let target_paths = fs::read_dir(directory)?;
        let file_list = target_paths.fold(vec![], |mut prev, path| {
            let target = match path {
                Ok(p) => {
                    let file_name = p.file_name();
                    let file_type = if let Ok(t) = p.file_type() {
                        t
                    } else {
                        eprintln!(
                            "Error: read file {} file type failed",
                            file_name.to_string_lossy()
                        );
                        return prev;
                    };
                    if file_type.is_file() && file_name.to_string_lossy().ends_with(".zip") {
                        p.path()
                    } else {
                        return prev;
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return prev;
                }
            };
            prev.push(target);
            prev
        });

        let s = Self {
            directory: PathBuf::from(directory),
            file_list,
        };
        dbg!(&s);
        Ok(s)
    }

    /// Unzip target
    ///
    /// - `path` unzip to folder
    fn unzip(&self, path: &Path) -> Result<()> {
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("conver filename failed")),
        };
        // create same name folder
        let name = self.format_name(&file_name);
        let mut dir_path = PathBuf::from(&self.directory);
        dir_path.push(name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }
        let mut ziper = Ziper::new(path)?;
        let dir_path = dir_path.to_string_lossy();
        ziper.unzip(Some(&dir_path))?;
        Ok(())
    }

    pub fn process(&self) -> Result<()> {
        for file in &self.file_list {
            self.unzip(file)?;
        }
        Ok(())
    }

    /// Format filename with extention
    ///
    /// - `file_name` target file name, such as `test.zip`
    fn format_name<'a>(&self, file_name: &'a str) -> &'a str {
        let name = file_name.split('.');
        let name = name.collect::<Vec<_>>();
        let name = name.first();
        if let Some(n) = name {
            n
        } else {
            ""
        }
    }
}
