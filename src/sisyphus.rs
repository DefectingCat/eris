use anyhow::{Ok, Result};
use std::{
    fs::{self},
    path::PathBuf,
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
        let file_list = target_paths
            .filter(|path| match path {
                std::result::Result::Ok(p) => {
                    let file_name = p.file_name();
                    let file_type = p.file_type().unwrap();
                    return file_type.is_file() && file_name.to_string_lossy().ends_with(".zip");
                }
                Err(err) => {
                    eprintln!("{}", err);
                    false
                }
            })
            .map(|path| path.unwrap().path())
            .collect::<Vec<_>>();

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
    fn unzip(&self, path: &PathBuf) -> Result<()> {
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => todo!(),
        };
        // create same name folder
        let name = file_name.split('.');
        let name: Vec<&str> = name.collect::<Vec<_>>();
        let name = name.first();
        let name = if let Some(n) = name { n } else { "" };
        let mut dir_path = PathBuf::from(&self.directory);
        dir_path.push(name);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path)?;
        }
        let mut ziper = Ziper::new(&path)?;
        ziper.unzip(Some(dir_path.to_str().unwrap()))?;
        Ok(())
    }

    pub fn process(&self) -> Result<()> {
        for file in &self.file_list {
            self.unzip(file)?;
        }
        Ok(())
    }
}
