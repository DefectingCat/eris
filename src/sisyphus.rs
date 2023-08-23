use anyhow::{Ok, Result};
use std::{
    fs::{self},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Sisyphus {
    /// Target directory
    pub directory: PathBuf,
    // Target zip files
    file_list: Vec<PathBuf>,
}

impl Sisyphus {
    pub fn new(directory: &PathBuf) -> Result<Self> {
        let target_paths = fs::read_dir(directory)?;
        let file_list = target_paths
            .filter(|path| {
                if let std::result::Result::Ok(p) = path {
                    let file_name = p.file_name();
                    let file_type = p.file_type().unwrap();
                    file_type.is_file() && file_name.to_string_lossy().ends_with(".zip")
                } else {
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
}
