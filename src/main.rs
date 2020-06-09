mod card;
mod config;

use std::io;
use std::fs;
use std::thread;

use clap::Clap;
use toml;

use card::Card;
use config::Config;

#[derive(Clap)]
#[clap(version = "1.0", author = "Robert May <rob@afternoonrobot.co.uk>")]
struct Opts {
    #[clap(short, long, default_value = "/etc/amdgpu-fancontrol/config.toml")]
    config: String
}

fn main() {
    let opts = Opts::parse();
    let config = load_config(&opts).unwrap();
    let mut threads = vec![];

    for card_name in &config.cards {
        let path = config.card_path(&card_name);

        match Card::new(&path, config.clone()) {
            Some(mut card) => {
                threads.push(
                    thread::spawn(move || {
                        card.control()
                    })
                )
            },
            None => panic!("Couldn't find card {}", card_name),
        }
    }

    for handle in threads {
        handle.join().unwrap();
    }
}

fn load_config(opts: &Opts) -> io::Result<Config> {
    let content = fs::read_to_string(&opts.config)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let opts = Opts { config: "assets/config.toml".to_string() };
        let config = load_config(&opts).unwrap();

        assert_eq!(config.cards, ["card0"]);
        assert_eq!(config.fan_wind_down, 30);
    }
}
