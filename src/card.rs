use std::fs;
use std::io;
use std::option::Option;
use std::path::Path;

#[derive(Debug)]
pub struct Card {
    path: String
}

impl Card {
    pub fn new(path_string: &str) -> Option<Card> {
        let card = Card {
            path: path_string.to_string()
        };

        if card.exists() {
            Some(card)
        } else {
            None
        }
    }

    pub fn control(&self) {
        let endpoints = self.endpoints();
    }

    pub fn restore_hardware_control(&self) {
       println!("Restoring hardware fan control");
    }

    pub fn exists(&self) -> bool {
        let path = Path::new(&self.path);

        path.is_dir()
    }

    fn endpoints(&self) {

    }
}

impl Drop for Card {
    fn drop(&mut self) {
        self.restore_hardware_control()
    }
}
