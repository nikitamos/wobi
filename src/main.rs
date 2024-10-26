use corpora::Corpora;
use std::{error::Error, path::Path};

mod corpora;
mod markov_chain;

fn main() -> Result<(), Box<dyn Error>> {
    const TOKENIZE_CONFIG: &str = "/home/m0sni/t10/assets/tokenize.toml";
    let mut corp = Corpora::open(Path::new(TOKENIZE_CONFIG))?;
    corp.analyze_all(2);
    corp.build_markov_chain();

    Ok(())
}
