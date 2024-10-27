#![allow(dead_code)]

use corpora::Corpora;
use std::io::{self, Write};
use std::str::FromStr;
use std::{error::Error, path::PathBuf, time::Instant};

mod corpora;
mod markov_chain;

fn main() -> Result<(), Box<dyn Error>> {
    let corpora_path: PathBuf =
        PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?.join("assets/tokenize.toml");
    let mut max_length: usize = 6;

    let mut corp = Corpora::open(&corpora_path)?;

    let analyze = Instant::now();
    corp.analyze_all();
    let build = Instant::now();
    let mut chain = corp.build_markov_chain().unwrap();
    let finish = Instant::now();
    print!(
        "Chain created in {:.2}s (analyze {:.2}s + build {:.2}s), {} job",
        (finish - analyze).as_secs_f32(),
        (build - analyze).as_secs_f32(),
        (finish - build).as_secs_f32(),
        corp.get_config().tokenize.jobs
    );
    if corp.get_config().tokenize.jobs > 1 {
        println!("s");
    } else {
        println!();
    }

    let mut buf = String::new();
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
                        Err(_) => println!("`=set` requires a positive interger argument"),
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
