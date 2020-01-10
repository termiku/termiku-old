use std::fs;

use clap::{App, AppSettings, Arg};

use termiku::config::Config as TermikuConfig;
use termiku::window::window;
use termiku::youtube::URL_PADDINGLESS_BASE64_RANGE;

use toml;

fn main() {
    let args = App::new("Termiku")
                   .version("0.1.0")
                   .author("ShinySaana & LunarLambda")
                   .arg(Arg::with_name("config")
                            .long("config")
                            .value_name("PATH")
                            .help("Path to the configuration file to use.")
                            .default_value("../config/termiku.toml"))
                    .arg(Arg::with_name("youtube")
                            .long("youtube")
                            .value_name("YOUTUBE_ID")
                            .help("Play youtube taken from its id"))
                   .setting(AppSettings::ColoredHelp)
                   .get_matches();

    if let Some(id) = args.value_of("youtube") {
        if id.len() == 11 {
            if id.as_bytes().iter().all(|x| URL_PADDINGLESS_BASE64_RANGE.contains(x)) {
                let formated = id.as_bytes().iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(";");
                print!("\x1B[{}y", formated);
                return;
            } else {
                println!("Usage: YOUTUBE_ID must be valid characters");
                std::process::exit(1);
            }
        } else {
            println!("Usage: YOUTUBE_ID must be 11 characters long");
            std::process::exit(1);
        }
    }

    let cpath = args.value_of("config").unwrap();

    let config = fs::read_to_string(&cpath)
                    .expect("Unable to open config file.");

    let config = toml::from_str::<TermikuConfig>(&config).unwrap();

    window(config);
}
