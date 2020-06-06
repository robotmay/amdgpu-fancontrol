use std::fs;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_new() {
        let path = Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").to_path_buf();
        let endpoint = Endpoint::new(path.clone());

        assert_eq!(endpoint.path, path);
    }

    #[test]
    fn test_exists() {
        let path = Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").to_path_buf();
        let endpoint = Endpoint::new(path.clone());

        assert_eq!(endpoint.exists(), true);
    }

    #[test]
    fn test_read_write() {
        let path = Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").to_path_buf();
        let endpoint = Endpoint::new(path.clone());

        let first_write = endpoint.write("30").unwrap();
        let first_read = endpoint.read();

        assert_eq!(first_write, ());
        assert_eq!(first_read, "30");

        let second_write = endpoint.write("11").unwrap();
        let second_read = endpoint.read();

        assert_eq!(second_write, ());
        assert_eq!(second_read, "11");
    }
}
