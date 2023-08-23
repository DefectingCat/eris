use std::path::PathBuf;

pub struct Sisyphus {
    pub directory: PathBuf,
}

impl Sisyphus {
    pub fn new(directory: &PathBuf) -> Self {
        Self {
            directory: PathBuf::from(directory),
        }
    }
}
