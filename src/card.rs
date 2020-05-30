mod endpoint;

use std::fs;
use std::io;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::thread;
use std::time;

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
        let mut recent_temps = vec![];

        loop {
            let temp = self.current_temperature();

            recent_temps.insert(0, temp);
            recent_temps.truncate(15);

            let max_recent_temp = recent_temps.iter().max().unwrap();

            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
            println!("Last temperature reading: {}", &temp);
            println!("Highest recent temperature reading: {}", &max_recent_temp);

            match max_recent_temp {
                0..=45 => self.set_fan_speed(self.min_fan_speed()),
                46..=55 => self.set_fan_speed(self.max_fan_speed() / 4),
                56..=65 => self.set_fan_speed(self.max_fan_speed() / 2),
                _ => self.set_fan_speed(self.max_fan_speed()),
            }

            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    fn assume_software_control(&self) {
        println!("Assuming software fan control for {:?}", self.path);

        match self.endpoint("pwm1_enable").write("1") {
            Ok(()) => println!("Assumed control"),
            Err(err) => panic!(err),
        }
    }

    fn restore_hardware_control(&self) {
        println!("Restoring hardware fan control for {:?}", self.path);

        match self.endpoint("pwm1_enable").write("2") {
            Ok(()) => println!("Restored control"),
            Err(err) => panic!(err),
        }
    }

    fn current_temperature(&self) -> i32 {
        self.endpoint("temp1_input")
            .read()
            .parse::<i32>()
            .unwrap() / 1000
    }

    fn min_fan_speed(&self) -> i32 {
        self.endpoint("pwm1_min")
            .read()
            .parse()
            .unwrap()
    }

    fn max_fan_speed(&self) -> i32 {
        self.endpoint("pwm1_max")
            .read()
            .parse()
            .unwrap()
    }

    fn set_fan_speed(&self, speed: i32) {
        match self.endpoint("pwm1").write(&speed.to_string()) {
            Ok(()) => println!("Fan speed set to {}", speed),
            Err(err) => panic!(err)
        }
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
