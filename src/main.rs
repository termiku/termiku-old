use std::fs;

use clap::{App, AppSettings, Arg};

use termiku::config::Config as TermikuConfig;
use termiku::window::window;

use toml;

fn main() {
    let args = App::new("Termiku")
                   .version("0.1.0")
                   .author("ShinySaana & LunarLambda")
                   .arg(Arg::with_name("config")
                            .long("config")
                            .value_name("PATH")
                            .help("Path to the configuration file to use.")
                            .default_value("config/termiku.toml"))
                   .setting(AppSettings::ColoredHelp)
                   .get_matches();

    let cpath = args.value_of("config").unwrap();

    let config = fs::read_to_string(&cpath)
                    .expect("Unable to open config file.");

    let config = toml::from_str::<TermikuConfig>(&config).unwrap();

    if let Some(ref envs) = config.env {
        for (key, val) in envs.iter() {
            println!("{} = {}", key, val);
        }
    }

    window("sh", &[], &config.env);
}
