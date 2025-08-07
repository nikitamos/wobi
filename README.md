# What does it do?
The program generates incoherent texts in a manner similar to T9. It analyzes given corpora of texts and for each pair of words determines probability that one will follow the other. When a word is known then the next one is chosen randomly, based on pre-calculated probabilities.
# How to use?
To set up the corpora, create a `tokenize.toml` file in `assets` directory and put your texts there.
## Sample config
```toml
# Corpora options
[corpora]
# Corpora Name (required)
name="Universal Declaration of Human Rights"
# List of plain text files to be analyzed (required)
texts=["udhr.txt"]
# Whether to dump the following statistics (optional, defaults to `false`):
# - Count of occurrences of each token
# - Markov chain matrix
# Note: this impacts performance severely
save_statistics=false

# Tokenization options
[tokenize]
# Regex that defines a token(required)
# Every match of the regex in the
# corpora will be converted to token.
# The following regex matches latin letters and points.
# Note single quotes for raw string.
token='([a-z]+|\.)'
# Whether to ignore case (i.e. consider 'A' and 'a' the same) (defaults to false)
ignore_case=true
# Number of jobs to run (defaults to 1)
jobs=4
```
## Running
To run the project, simply type
```shell
cargo run
```
Upon the startup, the process of Markov chain building begins. After it's finished, corresponding message is displayed. Then you
can type a token and press Enter. The program will generate 6 more tokens (by default).

The following commands are supported:

`=exit` Exit from the program;

`=set <n>` Sets length of generated output to `<n>` tokens.