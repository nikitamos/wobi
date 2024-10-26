use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use config::Config;
use regex::{Regex, RegexBuilder};

use crate::markov_chain::{MarkovChain, MarkovChainBuilder};

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
        #[serde(default = "serde_false_workaround")]
        pub save_statistics: bool,
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
    tokens: Vec<String>,
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
            tokens: Vec::new(),
        })
    }
    /// Tokenizes the `text` and returns the map
    /// that matches token and number of its
    /// occurencies in the `text`
    fn analyze_file(
        path: &Path,
        tokenizer: &Regex,
        ignore_case: bool,
    ) -> Option<HashMap<String, u32>> {
        let mut text = String::new();
        let mut tokenized = HashMap::new();
        match File::open(path) {
            Ok(mut f) => {
                if let Err(e) = f.read_to_string(&mut text) {
                    println!(
                        "Unable to read {}: {}",
                        path.to_str().unwrap(),
                        e.to_string()
                    );
                    return None;
                }
            }
            Err(e) => {
                println!(
                    "Unable to open {}: {}",
                    path.to_str().unwrap(),
                    e.to_string()
                );
                return None;
            }
        }

        if ignore_case {
            text = text.to_lowercase();
        }

        let tokens = tokenizer.find_iter(&text);
        for token in tokens {
            let s = token.as_str().to_owned();
            tokenized.entry(s).and_modify(|x| *x += 1).or_insert(1);
        }
        Some(tokenized)
    }

    fn sort_tokenization(tokenized: HashMap<String, u32>) -> Vec<(String, u32)> {
        let mut v = tokenized.into_iter().collect::<Vec<_>>();
        v.sort_by(|x, y| y.1.cmp(&x.1));
        v
    }

    fn merge_tokens<T: std::iter::IntoIterator<Item = (String, u32)>>(
        dst: &mut HashMap<String, u32>,
        src: T,
    ) {
        for i in src {
            dst.entry(i.0).and_modify(|x| *x += i.1).or_insert(i.1);
        }
    }

    pub fn analyze_all(&mut self, mut jobs: usize) {
        let stat_dir_ = self.directory.join("statistics");
        if self.cfg.corpora.save_statistics {
            if !stat_dir_.is_dir() {
                if let Err(e) = fs::create_dir(&stat_dir_) {
                    self.cfg.corpora.save_statistics = false;
                    println! {"[WARN] Error creating directory for statistics: {}\nNo statistics will be saved", e.to_string()};
                }
            }
        }
        let stat_dir_ = Arc::new(stat_dir_);
        let save_statistics_ = Arc::new(self.cfg.corpora.save_statistics);
        let directory_ = Arc::new(&self.directory);
        let tokenizer_ = Arc::new(&self.tokenizer);
        let ign_case_ = Arc::new(self.cfg.tokenize.ignore_case);

        thread::scope(|scope| {
            let all_paths = Arc::new(Mutex::new(self.cfg.corpora.texts.clone()));
            jobs = jobs.min(self.cfg.corpora.texts.len());
            let mut pool = Vec::with_capacity(jobs);
            for i in 0..jobs {
                let directory = directory_.clone();
                let save_statistics = save_statistics_.clone();
                let paths = all_paths.clone();
                let stat_dir = stat_dir_.clone();
                let tokenizer = tokenizer_.clone();
                let ign_case = ign_case_.clone();
                pool.push(
                    thread::Builder::new()
                        .name(format!("Tokenize #{i}"))
                        .spawn_scoped(scope, move || -> HashMap<String, u32> {
                            let mut tokens = HashMap::new();

                            loop {
                                let path = {
                                    let mut lock = paths.lock().unwrap();
                                    if lock.len() == 0 {
                                        break;
                                    }
                                    lock.pop().unwrap()
                                };
                                if let Some(tok) = Self::analyze_file(
                                    &directory.join(&path),
                                    &tokenizer,
                                    *ign_case,
                                ) {
                                    if *save_statistics {
                                        let file_tokens = Self::sort_tokenization(tok);
                                        match File::create(stat_dir.join(path)) {
                                            Ok(mut f) => {
                                                for t in &file_tokens {
                                                    let _ = f.write(
                                                        format!("{} {}\n", t.1, t.0).as_bytes(),
                                                    );
                                                }
                                            }
                                            Err(e) => println!(
                                                "[WARN] Unable to save statistics: {}",
                                                e.to_string()
                                            ),
                                        }
                                        Self::merge_tokens(&mut tokens, file_tokens);
                                    } else {
                                        Self::merge_tokens(&mut tokens, tok);
                                    }
                                }
                            }
                            tokens
                        })
                        .expect("Unable to spawn a thread"),
                );
            }
            let mut a = HashMap::new();
            for thr in pool {
                Self::merge_tokens(&mut a, thr.join().expect("Thread finished with an error"));
            }

            let tokens = Self::sort_tokenization(a);
            if self.cfg.corpora.save_statistics {
                match File::create(stat_dir_.join("overall_stat.txt")) {
                    Ok(mut f) => {
                        for t in &tokens {
                            let _ = f.write(format!("{} {}\n", t.1, t.0).as_bytes());
                        }
                    }
                    Err(e) => println!("[WARN] Unable to save statistics: {}", e.to_string()),
                }
            }
            self.tokens = tokens.into_iter().map(|(k, _)| k).collect();
        });
    }

    pub fn build_markov_chain(&self) -> Option<MarkovChain<String>> {
        let mut builder = MarkovChainBuilder::with_states(self.tokens.clone());
        for fpath in &self.cfg.corpora.texts {
            let mut text = String::new();
            match File::open(self.directory.join(&fpath)) {
                Ok(mut f) => {
                    if let Err(e) = f.read_to_string(&mut text) {
                        println!("Unable to read {fpath}: {}", e.to_string());
                        return None;
                    }
                },
                Err(e) => {
                    println!("[WARN] Unable to open {fpath}: {}", e.to_string());
                    return None;
                }
            }
            if self.cfg.tokenize.ignore_case {
                text = text.to_lowercase();
            }
            let mut token = self.tokenizer.find_iter(&text);
            let mut prev = token.next().unwrap().as_str().to_owned();
            for this in token {
                let s = this.as_str().to_string();
                builder.add_transition(&prev, &s);
                prev = s;
            }
        }
        if self.cfg.corpora.save_statistics {
            builder.dump_matrix(&self.directory.join("statistics").join("OCCURANCE_MATRIX.txt"));
        }
        Some(builder.build())
    }

    pub fn get_config(&self) -> &config::Config {
        &self.cfg
    }
}
