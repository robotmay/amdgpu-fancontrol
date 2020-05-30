mod endpoint;

use std::fs;
use std::io;
use std::option::Option;
use std::path::{Path, PathBuf};

use endpoint::Endpoint;

const REQUIRED_ENDPOINTS: [&str; 5] = [
    "temp1_input",
    "pwm1_max",
    "pwm1_min",
    "pwm1_enable",
    "pwm1"
];

#[derive(Debug)]
pub struct Card {
    path: PathBuf,
    endpoint_path: PathBuf
}

impl Card {
    pub fn new(path_string: &str) -> Option<Card> {
        let path = Path::new(path_string);
        let endpoint_path = path.join("device/hwmon/hwmon0");

        println!("Path: {:?}", path);
        println!("Endpoint path: {:?}", endpoint_path);

        let card = Card {
            path: path.to_path_buf(),
            endpoint_path: endpoint_path
        };

        if card.exists() && card.verify() {
            Some(card)
        } else {
            None
        }
    }

    pub fn control(&self) {
        self.assume_software_control();
    }

    fn assume_software_control(&self) {
        println!("Assuming software fan control for {:?}", self.path);
        self.endpoint("pwm1_enable").write("1")
    }

    fn restore_hardware_control(&self) {
       println!("Restoring hardware fan control for {:?}", self.path);
       self.endpoint("pwm1_enable").write("2")
    }

    fn exists(&self) -> bool {
        println!("Does {:?} exist?", self.path);
        self.path.is_dir()
    }

    fn verify(&self) -> bool {
        REQUIRED_ENDPOINTS
            .iter()
            .map(|endpoint| self.endpoint(endpoint))
            .all(|endpoint| endpoint.exists())
    }

    fn endpoint(&self, name: &str) -> Endpoint {
        Endpoint::new(self.endpoint_path.join(name))
    }
}

impl Drop for Card {
    fn drop(&mut self) {
        self.restore_hardware_control()
    }
}
