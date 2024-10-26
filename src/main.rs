#![allow(dead_code)]

use std::io::{self, Write};
use corpora::Corpora;
use std::{
    error::Error,
    path::Path,
};

mod corpora;
mod markov_chain;

fn main() -> Result<(), Box<dyn Error>> {
    const TOKENIZE_CONFIG: &str = "/home/m0sni/t10/assets/tokenize.toml";
    let mut max_length: usize = 6;

    let mut corp = Corpora::open(Path::new(TOKENIZE_CONFIG))?;
    corp.analyze_all(2);
    let mut chain = corp.build_markov_chain().unwrap();
    let mut buf = String::new();
    println!("Chain created!");

    'outer: loop {
        loop {
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            buf = buf.trim().to_string();
            if let Ok(()) = chain.reset(&buf) {
                print!("{buf} ");
                break;
            } else if buf == "=exit" {
                break 'outer;
            } else if buf.starts_with("=set") {
                if let Some(x) = buf.split(' ').skip(1).peekable().peek() {
                    match x.to_owned().parse() {
                        Ok(y) => max_length = y,
                        Err(_) => println!("`=set` requires a positive interger argument")
                    }
                } else {
                    println!("`=set` requires a positive integer argument")
                }
            } else {
                println!("There is no word {buf} in the dictionary");
            }
        }

        let mut i = 0;
        while let Some(x) = chain.next_state() {
            if i == max_length {
                break;
            }
            print!("{x} ");
            i += 1;
        }
        println!();
        io::stdout().flush()?;
    }
    Ok(())
}
