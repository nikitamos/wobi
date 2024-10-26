use corpora::Corpora;
use regex::Regex;
use std::{error::Error, fs::File, io::Read, path::Path};

mod corpora;
mod markov_chain;

fn main() -> Result<(), Box<dyn Error>> {
    const TOKENIZE_CONFIG: &str = "/home/m0sni/t10/assets/tokenize.toml";
    let mut corp = Corpora::open(Path::new(TOKENIZE_CONFIG))?;
    corp.tokenize_all(1);

    Ok(())
}
