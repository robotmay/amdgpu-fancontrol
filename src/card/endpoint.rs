use std::fs;
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

    pub fn read(&self) -> String {
        let content = fs::read_to_string(&self.path)
            .expect(&format!("Failed to read endpoint {:?}", self.path));

        content.trim().to_string()
    }

    pub fn write(&self, value: &str) -> std::io::Result<()> {
        let result = fs::write(&self.path, value)
            .expect(&format!("Failed to write to endpoint {:?}", self.path));

        Ok(result)
    }
}
