mod endpoint;

use regex::Regex;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::thread;
use std::time;

use crate::config::Config;
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
    endpoint_path: PathBuf,
    debug_endpoint_path: PathBuf
}

impl Card {
    pub fn new(path: &PathBuf, config: &Config) -> Option<Card> {
        let full_endpoint_path = path.join(&config.endpoint_path);
        let debug_endpoint_path = Path::new(&config.debug_endpoint_path);

        let card = Card {
            path: path.to_path_buf(),
            endpoint_path: full_endpoint_path,
            debug_endpoint_path: debug_endpoint_path.to_path_buf()
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
        let mut recent_load: Vec<i32> = vec![];

        loop {
            self.adjust_fan(&fan_wind_down, &mut recent_temps, &mut recent_load)
                .expect("Failed to adjust the fan");
            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    pub fn adjust_fan(&self, fan_wind_down: &usize, recent_temps: &mut Vec<i32>, recent_load: &mut Vec<i32>) -> std::io::Result<()> {
        let temp = self.current_temperature();
        let gpu_load = self.gpu_usage_percentage();

        // Keep the last <fan_wind_down> seconds of readings
        recent_temps.insert(0, temp);
        recent_temps.truncate(*fan_wind_down);

        // Keep a window of GPU load percentages
        recent_load.insert(0, gpu_load);
        recent_load.truncate(*fan_wind_down);

        let max_recent_temp = recent_temps.iter().max().unwrap();
        let current_fan_speed = self.get_fan_speed();
        let new_fan_speed = self.calculate_new_fan_speed(&max_recent_temp);

        println!("card={:?} current={} window={} fanspeed={} load={}", self.path, &temp, &max_recent_temp, &current_fan_speed, &gpu_load);

        self.set_fan_speed(new_fan_speed)
    }

    fn calculate_new_fan_speed(&self, max_recent_temp: &i32) -> i32 {
        // Change fan speed with <fan_wind_down> seconds of wind-down delay
        match max_recent_temp {
            0..=40 => self.min_fan_speed(),
            41..=45 => self.max_fan_speed() / 5,
            46..=50 => self.max_fan_speed() / 4,
            51..=60 => self.max_fan_speed() / 3,
            61..=65 => self.max_fan_speed() / 2,
            66..=70 => (self.max_fan_speed() / 2) + (self.max_fan_speed() / 4),
            _ => self.max_fan_speed(),
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

    fn gpu_usage_percentage(&self) -> i32 {
        let regex = Regex::new(r"GPU Load: (?P<load>\d+) %").unwrap();
        let debug_info = self.debug_endpoint().read();
        let caps = regex.captures(&debug_info).unwrap();

        println!("{:?}", caps);

        caps["load"].parse().unwrap()
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

    fn debug_endpoint(&self) -> Endpoint {
        Endpoint::new(self.debug_endpoint_path.to_path_buf())
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
        fs::write(Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/temp1_input"), "35000")
            .expect("Couldn't write to temp1_input");

        fs::write(Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1_min"), "0")
            .expect("Couldn't write to pwm1_min");

        fs::write(Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1_max"), "255")
            .expect("Couldn't write to pwm1_max");

        fs::write(Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1"), "30")
            .expect("Couldn't write to pwm1");
    }

    fn config() -> Config {
        Config {
            cards_path: "test/sys/class/drm".to_string(),
            cards: vec!["card0".to_string()],
            endpoint_path: "device/hwmon/hwmon0".to_string(),
            fan_wind_down: 30,
            debug_endpoint_path: "test/sys/kernel/debug/dri/0/amdgpu_pm_info".to_string()
        }
    }

    fn card() -> Card {
        let config = config();
        let path = config.card_path("card0");

        Card::new(&path, &config).unwrap()
    }

    #[test]
    fn test_new() {
        let card = card();

        assert_eq!(card.path, Path::new("test/sys/class/drm/card0"));
        assert_eq!(card.endpoint_path, Path::new("test/sys/class/drm/card0/device/hwmon/hwmon0"));
    }

    #[test]
    fn test_adjust_fan() {
        setup();

        let card = card();

        let fan_wind_down: usize = 30;
        let mut recent_temps: Vec<i32> = vec![];
        let mut recent_load: Vec<i32> = vec![];

        // Check temperature from setup
        assert_eq!(fs::read_to_string("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").unwrap(), "30");

        // Run the fan adjustment based on the setup temperature
        let adjust = card.adjust_fan(&fan_wind_down, &mut recent_temps, &mut recent_load).unwrap();
        assert_eq!(adjust, ());

        // Ensure the fan was correctly updated
        assert_eq!(fs::read_to_string("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").unwrap(), "0");
    }

    #[test]
    fn test_gpu_load() {
        let card = card();

        assert_eq!(card.gpu_usage_percentage(), 10);
    }
}
