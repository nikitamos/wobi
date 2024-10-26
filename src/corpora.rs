use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use config::Config;
use regex::{Regex, RegexBuilder};

pub mod config {
    use std::{
        error::Error,
        fs,
        io::{Read, Write},
        path::Path,
    };

    use serde::{Deserialize, Serialize};

    fn serde_false_workaround() -> bool {
        false
    }

    #[derive(Deserialize, Serialize)]
    pub struct Tokenize {
        #[serde(default)]
        pub token: String,
        #[serde(default = "serde_false_workaround")]
        pub ignore_case: bool,
    }
    impl Default for Tokenize {
        fn default() -> Self {
            Self {
                token: String::new(),
                ignore_case: false,
            }
        }
    }
    #[derive(Deserialize, Serialize)]
    pub struct Corpora {
        pub name: String,
        pub texts: Vec<String>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct Config {
        #[serde(default)]
        pub tokenize: Tokenize,
        pub corpora: Corpora,
    }
    impl Config {
        pub fn read_from(path: &Path) -> Result<Self, Box<dyn Error>> {
            let mut f = fs::File::open(path)?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            Ok(toml::from_str(&buf)?)
        }
        pub fn write_to(&self, path: &Path) -> Result<(), Box<dyn Error>> {
            let mut f = fs::File::create(path)?;
            let buf: String = toml::to_string(self)?;
            f.write_all(buf.as_bytes())?;
            Ok(())
        }
    }
}

pub struct Corpora {
    cfg: config::Config,
    directory: PathBuf,
    tokenizer: Regex,
    tokens: HashMap<String, u32>,
}

impl Corpora {
    /// Opens a corpora with configuration file at `path`
    pub fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut path = path.to_owned();
        let cfg = Config::read_from(&path)?;
        let tokenizer = RegexBuilder::new(&cfg.tokenize.token)
            .case_insensitive(cfg.tokenize.ignore_case)
            .build()?;
        path.pop();
        Ok(Corpora {
            cfg,
            directory: path,
            tokenizer,
            tokens: HashMap::new(),
        })
    }
    /// Tokenizes the `text` and returns the map
    /// that matches token and number of its
    /// occurencies in the `text`
    fn tokenize_file(&self, tokenized: &mut HashMap<String, u32>, path: String) -> Option<()> {
        let mut text = String::new();
        match File::open(&path) {
            Ok(mut f) => {
                if let Err(e) = f.read_to_string(&mut text) {
                    println!("Unable to read {path}: {}", e.to_string());
                    return None;
                }
            }
            Err(e) => {
                println!("Unable to open {path}: {}", e.to_string());
                return None;
            }
        }

        let tokens = self.tokenizer.find_iter(&text);
        for token in tokens {
            let s = token.as_str().to_owned();
            tokenized.entry(s).and_modify(|x| *x += 1).or_insert(0);
        }
        Some(())
    }

    pub fn tokenize_all(&mut self, mut jobs: usize) {
        thread::scope(|scope| {
            let mut all_paths = Arc::new(Mutex::new(
                self.cfg
                    .corpora
                    .texts
                    .clone()
            ));

            jobs = jobs.min(self.cfg.corpora.texts.len());
            let mut pool = Vec::with_capacity(jobs);
            for i in 0..jobs {
                let corpora = Arc::new(&self);
                let mut paths = all_paths.clone();
                pool.push(
                    thread::Builder::new()
                        .name(format!("Tokenize #{i}"))
                        .spawn_scoped(scope, move || -> HashMap<String, u32> {
                            let mut tokens = HashMap::new();

                            loop {
                                let path = {   
                                    let mut lock = paths.lock().unwrap();
                                    if lock.len() == 0 {break;}
                                    lock.pop().unwrap()
                                };
                                corpora.tokenize_file(&mut tokens, path);
                            }
                            tokens
                        })
                        .expect("Unable to spawn a thread"),
                );
            }
            for thr in pool {
                let r = thr.join().expect("Thread finished with an error");
            }
        });
    }

    pub fn get_config(&self) -> &config::Config {
        &self.cfg
    }
}
