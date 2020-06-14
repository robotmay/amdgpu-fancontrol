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
    config: Config,
    path: PathBuf,
    endpoint_path: PathBuf,
    monitoring_path: PathBuf,
    temp_window: Vec<i32>,
    load_window: Vec<i32>,
    fan_window: Vec<i32>,
    current_fan_speed: i32
}

impl Card {
    pub fn new(path: &PathBuf, config: Config) -> Option<Card> {
        let full_endpoint_path = path.join(&config.endpoint_path);
        let monitoring_path = Path::new(&config.monitoring_path);

        let card = Card {
            config: config.clone(),
            path: path.to_path_buf(),
            endpoint_path: full_endpoint_path,
            monitoring_path: monitoring_path.to_path_buf(),
            temp_window: vec![],
            load_window: vec![],
            fan_window: vec![],
            current_fan_speed: 0
        };

        if card.exists() && card.verify() {
            Some(card)
        } else {
            None
        }
    }

    pub fn control(&mut self) -> std::thread::JoinHandle<()> {
        self.assume_software_control();

        loop {
            self.adjust_fan()
                .expect("Failed to adjust the fan");

            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    pub fn adjust_fan(&mut self) -> std::io::Result<()> {
        let temp = self.current_temperature();
        let gpu_load = self.gpu_usage_percentage();

        // Keep a window of temperature readings
        self.temp_window.insert(0, temp);
        self.temp_window.truncate(self.config.measurement_window);

        // Keep a window of GPU load percentages
        self.load_window.insert(0, gpu_load);
        self.load_window.truncate(self.config.measurement_window);

        let max_recent_temp = self.temp_window.iter().max().unwrap();
        let min_recent_temp = self.temp_window.iter().min().unwrap();
        let load_average = self.calculate_avg_load(&self.load_window);
        let current_fan_speed: i32 = self.get_fan_speed();
        let new_fan_speed: i32 = self.calculate_new_fan_speed(&max_recent_temp);
        let bouncing = self.bouncing();

        println!(
            "card={:?} current_temp={} max_temp={} min_temp={} fanspeed={} load={} load_avg={} bouncing={}",
            self.path,
            &temp,
            &max_recent_temp,
            &min_recent_temp,
            &current_fan_speed,
            &gpu_load,
            &load_average,
            &bouncing
        );

        if (new_fan_speed < current_fan_speed) && bouncing {
            Ok(())
        } else if new_fan_speed == current_fan_speed {
            Ok(())
        } else {
            self.set_fan_speed(new_fan_speed)
        }
    }

    fn bouncing(&self) -> bool {
        let max_recent_temp: i32 = *self.temp_window.iter().max().unwrap();
        let min_recent_temp: i32 = *self.temp_window.iter().min().unwrap();
        let difference: i32 = max_recent_temp - min_recent_temp;

        min_recent_temp < max_recent_temp && difference < 5
    }

    fn calculate_avg_load(&self, recent_load: &Vec<i32>) -> i32 {
        let sum: i32 = recent_load.iter().sum();

        sum as i32 / recent_load.len() as i32
    }

    fn calculate_new_fan_speed(&self, max_recent_temp: &i32) -> i32 {
        match max_recent_temp {
            0..=45 => self.min_fan_speed(),
            46..=50 => self.speed_step(2),
            51..=55 => self.speed_step(3),
            56..=60 => self.speed_step(4),
            61..=65 => self.speed_step(5),
            66..=70 => self.speed_step(6),
            71..=75 => self.speed_step(7),
            _ => self.max_fan_speed(),
        }
    }

    fn speed_step(&self, step: i32) -> i32 {
        let step = step as f32;
        let base = (self.max_fan_speed() / 12) as f32;
        let load_average = self.calculate_avg_load(&self.load_window);

        let multiplier = match load_average {
            51..=75 => 1.1,
            76..=100 => 1.2,
            _ => 1.0,
        };

        let speed = (base * step * multiplier) as i32;

        if speed < self.max_fan_speed() {
            speed
        } else {
            self.max_fan_speed()
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
        if self.monitoring_endpoint().exists() {
            let regex = Regex::new(r"GPU Load: (?P<load>\d+) %").unwrap();
            let monitoring_info = self.monitoring_endpoint().read();
            let caps = regex.captures(&monitoring_info).unwrap();

            caps["load"].parse().unwrap()
        } else {
            0
        }
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

    fn monitoring_endpoint(&self) -> Endpoint {
        Endpoint::new(self.monitoring_path.to_path_buf())
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
            measurement_window: 30,
            endpoint_path: "device/hwmon/hwmon0".to_string(),
            monitoring_path: "test/sys/kernel/debug/dri/0/amdgpu_pm_info".to_string()
        }
    }

    fn card() -> Card {
        let config = config();
        let path = config.card_path("card0");

        Card::new(&path, config.clone()).unwrap()
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

        let mut card = card();

        // Check temperature from setup
        assert_eq!(fs::read_to_string("test/sys/class/drm/card0/device/hwmon/hwmon0/pwm1").unwrap(), "30");

        // Run the fan adjustment based on the setup temperature
        let adjust = card.adjust_fan().unwrap();
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
