use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub cards: Vec<String>,
    pub fan_wind_down: usize,
    pub cards_path: String,
    pub endpoint_path: String,
    pub debug_endpoint_path: String
}

impl Config {
    pub fn card_path(&self, card_name: &str) -> PathBuf {
        Path::new(&self.cards_path).join(&card_name)
    }
}
