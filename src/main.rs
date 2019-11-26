use std::fs;

use clap::{App, AppSettings, Arg};

use termiku::config::Config as TermikuConfig;
use termiku::window::window;

use termiku::pty_buffer::*;

use toml;

fn main() {
    test();
    
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

fn test() {
    // let mut data = [1, 2, 3].iter().peekable();
    // 
    // loop {
    //     let is_last = data.peek()
    // 
    //     match data.next() {
    //         Some(data) => {
    //             println!("")
    // 
    //         },
    //         None => break
    //     }
    // }
    // 
    // let mut buffer = PtyBuffer::new();
    // buffer.add_input("1111111111111111\n");
    // buffer.add_input("2222222222222222");
    // buffer.add_input("2222222222222222\n");
    // buffer.add_input("3333333333333");
    // buffer.add_input("3333333333333");
    // buffer.add_input("3333333333333\n");
    // buffer.add_input("4444444444");
    // buffer.add_input("4444444\n555555");
    // buffer.add_input("55555555555");
    // buffer.add_input("\n6666666666666");
    // buffer.add_input("666666666\n");
    // buffer.add_input("7777777");
    // 
    // println!("{:#2?}", buffer.get_range(0, 7));
}