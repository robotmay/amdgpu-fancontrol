mod card;

use std::io;
use std::fs;
use std::thread;

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
    cards: Vec<String>,
    fan_wind_down: usize
}

fn main() {
    let opts = Opts::parse();
    let config = load_config(&opts).unwrap();
    let mut threads = vec![];

    for card_name in config.cards {
        let path_str = format!("/sys/class/drm/{}", card_name);

        match Card::new(&path_str) {
            Some(card) => {
                let fan_wind_down = usize::clone(&config.fan_wind_down);

                threads.push(
                    thread::spawn(move || {
                        card.control(fan_wind_down)
                    })
                )
            },
            None => panic!("Couldn't find card {}", card_name),
        }
    }

    for handle in threads {
        handle.join();
    }
}

fn load_config(opts: &Opts) -> io::Result<Config> {
    let content = fs::read_to_string(&opts.config)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
