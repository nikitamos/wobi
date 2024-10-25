use regex::Regex;
use std::{error::Error, fs::File, io::Read};

mod config;
mod markov_chain;

fn main() -> Result<(), Box<dyn Error>> {
    const TOKENIZE_CONFIG: &str = "/HDD/home/nikita/Projects/Rust/t10/assets/tokenize.toml";
    const UDHR_PATH: &str = "/HDD/home/nikita/Projects/Rust/t10/assets/udhr.txt";

    let mut conf = String::new();
    File::open(TOKENIZE_CONFIG)?.read_to_string(&mut conf)?;
    let conf: config::Config = toml::from_str(&conf)?;

    let mut datatext = String::new();
    File::open(UDHR_PATH)?.read_to_string(&mut datatext)?;

    for i in &conf.tokenize.ignore {
        match Regex::new(i) {
            Ok(regex) => {
                datatext = regex.replace_all(&datatext, "").to_string();
            }
            Err(_) => println!("[WARN] {:?} is not a valid regex", i),
        }
    }
    if conf.tokenize.ignore_case {
        let nullify = Regex::new(r"([A-Z])").unwrap();
    }
    println!("{datatext}");

    Ok(())
}
