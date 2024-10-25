use serde::Deserialize;

fn serde_false_workaround() -> bool {
    false
}

#[derive(Deserialize)]
pub struct Tokenize {
    #[serde(default)]
    pub stop: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default = "serde_false_workaround")]
    pub ignore_case: bool,
}

impl Default for Tokenize {
    fn default() -> Self {
        Self {
            stop: vec![" ".to_string()],
            ignore: vec![],
            ignore_case: false,
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tokenize: Tokenize,
}
