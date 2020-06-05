mod endpoint;

use std::option::Option;
use std::path::PathBuf;
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
    pub fn new(path: &PathBuf, endpoint_path: &str) -> Option<Card> {
        let full_endpoint_path = path.join(endpoint_path);

        let card = Card {
            path: path.to_path_buf(),
            endpoint_path: full_endpoint_path
        };

        if card.exists() && card.verify() {
            Some(card)
        } else {
            None
        }
    }

    pub fn control(&self, fan_wind_down: usize) -> std::thread::JoinHandle<()> {
        self.assume_software_control();
        let mut recent_temps: Vec<i32> = vec![];

        loop {
            self.adjust_fan(&fan_wind_down, &mut recent_temps)
                .expect("Failed to adjust the fan");
            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    pub fn adjust_fan(&self, fan_wind_down: &usize, recent_temps: &mut Vec<i32>) -> std::io::Result<()> {
        let temp = self.current_temperature();

        // Keep the last <fan_wind_down> seconds of readings
        recent_temps.insert(0, temp);
        recent_temps.truncate(*fan_wind_down);

        let max_recent_temp = recent_temps.iter().max().unwrap();
        let current_fan_speed = self.get_fan_speed();

        println!("card={:?} current={} window={} fanspeed={}", self.path, &temp, &max_recent_temp, &current_fan_speed);

        // Change fan speed with <fan_wind_down> seconds of wind-down delay
        match max_recent_temp {
            0..=40 => self.set_fan_speed(self.min_fan_speed()),
            41..=45 => self.set_fan_speed(self.max_fan_speed() / 5),
            46..=50 => self.set_fan_speed(self.max_fan_speed() / 4),
            51..=60 => self.set_fan_speed(self.max_fan_speed() / 3),
            61..=65 => self.set_fan_speed(self.max_fan_speed() / 2),
            66..=70 => self.set_fan_speed((self.max_fan_speed() / 2) + (self.max_fan_speed() / 4)),
            _ => self.set_fan_speed(self.max_fan_speed()),
        }
    }

    fn assume_software_control(&self) {
        match self.endpoint("pwm1_enable").write("1") {
            Ok(()) => println!("Assumed control"),
            Err(err) => panic!(err),
        }
    }

    fn restore_hardware_control(&self) {
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

    fn set_fan_speed(&self, speed: i32) -> std::io::Result<()> {
        self.endpoint("pwm1").write(&speed.to_string())
    }

    fn get_fan_speed(&self) -> i32 {
        self.endpoint("pwm1")
            .read()
            .parse()
            .unwrap()
    }

    fn exists(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup() {
        fs::write(Path::new("test/card0/device/hwmon/hwmon0/temp1_input"), "35000")
            .expect("Couldn't write to temp1_input");

        fs::write(Path::new("test/card0/device/hwmon/hwmon0/pwm1_min"), "0")
            .expect("Couldn't write to pwm1_min");

        fs::write(Path::new("test/card0/device/hwmon/hwmon0/pwm1_max"), "255")
            .expect("Couldn't write to pwm1_max");

        fs::write(Path::new("test/card0/device/hwmon/hwmon0/pwm1"), "30")
            .expect("Couldn't write to pwm1");
    }

    #[test]
    fn test_new() {
        let path = Path::new("test/card0").to_path_buf();
        let card = Card::new(&path, "device/hwmon/hwmon0").unwrap();

        assert_eq!(card.path, Path::new("test/card0"));
        assert_eq!(card.endpoint_path, Path::new("test/card0/device/hwmon/hwmon0"));
    }

    #[test]
    fn test_adjust_fan() {
        setup();

        let path = Path::new("test/card0").to_path_buf();
        let card = Card::new(&path, "device/hwmon/hwmon0").unwrap();
        let fan_wind_down: usize = 30;
        let mut recent_temps: Vec<i32> = vec![];

        // Check temperature from setup
        assert_eq!(fs::read_to_string("test/card0/device/hwmon/hwmon0/pwm1").unwrap(), "30");

        // Run the fan adjustment based on the setup temperature
        let adjust = card.adjust_fan(&fan_wind_down, &mut recent_temps).unwrap();
        assert_eq!(adjust, ());

        // Ensure the fan was correctly updated
        assert_eq!(fs::read_to_string("test/card0/device/hwmon/hwmon0/pwm1").unwrap(), "0");
    }
}
