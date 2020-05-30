mod card;

use std::io;
use std::fs;
use std::path::Path;

use clap::Clap;
use serde::Deserialize;
use toml;

use card::Card;

#[derive(Clap)]
#[clap(version = "1.0", author = "Robert May <rob@afternoonrobot.co.uk>")]
struct Opts {
    #[clap(short, long, default_value = "/etc/amdgpu-fancontrol/config.toml")]
    config: String
}

#[derive(Deserialize, Debug)]
struct Config {
    cards: Vec<String>
}

fn main() {
    let opts = Opts::parse();
    let config = load_config(&opts).unwrap();

    println!("{}", opts.config);
    println!("{:?}", config);

    for card_name in config.cards {
        let path_str = format!("/sys/class/drm/{}", card_name);

        match Card::new(&path_str) {
            Some(card) => card.control(),
            None => panic!("Couldn't find card {}", card_name),
        }
    }
}

fn load_config(opts: &Opts) -> io::Result<Config> {
    let content = fs::read_to_string(&opts.config)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
