use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Endpoint {
    path: PathBuf
}

impl Endpoint {
    pub fn new(path: PathBuf) -> Endpoint {
        Endpoint {
            path: path
        }
    }

    pub fn exists(&self) -> bool {
        let existence: bool = self.path.is_file();
        println!("{:?} exists? {}", self.path, existence);
        existence
    }

    pub fn read(&self) {

    }

    pub fn write(&self, value: &str) {

    }
}
