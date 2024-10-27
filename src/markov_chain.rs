use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
    fs::File,
    hash::Hash,
    io::Write,
    path::Path,
    sync::Arc,
    thread,
};

pub struct MarkovChain<T: Hash> {
    matrix: Vec<Vec<f32>>,
    state_ids: HashMap<T, usize>,
    states: Vec<T>,
    current_state: usize,
}

impl<T: Hash + Eq + Clone> MarkovChain<T> {
    pub fn reset(&mut self, state: &T) -> Result<(), ()> {
        if self.state_ids.contains_key(&state) {
            self.current_state = self.state_ids[&state];
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn next_state(&mut self) -> Option<T> {
        let next: f32 = rand::random();
        let mut sum = 0f32;
        for i in 0..self.matrix.len() {
            sum += self.matrix[self.current_state][i];
            if sum >= next {
                self.current_state = i;
                return Some(self.states[i].clone());
            }
        }
        self.current_state = 0;
        None
    }
}

enum BuilderState {
    Stating,
    Recording(Vec<Vec<u32>>),
}

use BuilderState::*;

pub struct MarkovChainBuilder<T: Hash + Eq> {
    state: BuilderState,
    mapped: HashMap<T, usize>,
    vectored: Vec<T>,
}

impl<T: Hash + Eq + Default + Clone + Display> MarkovChainBuilder<T> {
    /// Computes the lines [beg..end) of the chain matrix
    /// from the corresponding lines of occurance matrix
    fn build_worker(state: &Vec<Vec<u32>>, beg: usize, end: usize) -> Vec<Vec<f32>> {
        let mut matrix = Vec::new();
        for i in beg..end {
            matrix.push(Vec::<f32>::new());
            let mut sum = 0f32;
            for j in 0..state.len() {
                matrix[i - beg].push(state[i][j] as f32);
                sum += state[i][j] as f32;
            }
            for j in 0..state.len() {
                matrix[i - beg][j] /= sum;
            }
        }
        matrix
    }
    pub fn build(self, mut jobs: usize) -> MarkovChain<T> {
        if let Recording(state) = self.state {
            let state_ = Arc::new(state);
            jobs = jobs.min(state_.len());
            let mut pool = VecDeque::with_capacity(jobs);
            let rem = state_.len() % jobs;
            let mut used = 0;
            let div = state_.len() / jobs;

            for i in 0..jobs {
                let beg = used + i * div;
                let mut end = beg + div;
                if used < rem {
                    used += 1;
                    end += 1;
                }
                let state = state_.clone();

                pool.push_back(thread::spawn(move || {
                    Self::build_worker(&state, beg, end).into_iter()
                }));
            }
            let mut a: Box<dyn Iterator<Item = Vec<f32>>> =
                Box::new(pool.pop_front().unwrap().join().unwrap());
            for _ in 1..jobs {
                a = Box::new(a.chain(pool.pop_front().unwrap().join().unwrap()));
            }
            let matrix: Vec<Vec<f32>> = a.collect();

            MarkovChain {
                matrix,
                current_state: 0,
                state_ids: self.mapped,
                states: self.vectored,
            }
        } else {
            panic!("Attempt to build map while stating not finalized");
        }
    }
    pub fn new() -> Self {
        MarkovChainBuilder::<T> {
            state: Stating,
            mapped: Default::default(),
            vectored: Default::default(),
        }
    }
    pub fn with_states(state: Vec<T>) -> Self {
        let mut r = MarkovChainBuilder::<T> {
            state: Stating,
            mapped: state
                .clone()
                .into_iter()
                .enumerate()
                .map(|(x, y)| (y, x))
                .collect(),
            vectored: state,
        };
        r.finalize_stating();
        r
    }
    pub fn add_state(&mut self, state: T) {
        if let Stating = self.state {
            self.vectored.push(state.clone());
            self.mapped.insert(state, self.mapped.len());
        }
    }
    pub fn finalize_stating(&mut self) {
        let mut matrix: Vec<Vec<u32>>;
        if let Stating = &mut self.state {
            let cap = self.mapped.len();
            matrix = Vec::with_capacity(cap);
            let mut v = Vec::with_capacity(cap);
            for _ in 0..cap {
                v.push(0);
            }
            for _ in 0..cap {
                matrix.push(v.clone());
            }
        } else {
            panic!("finalize_stating called twice");
        }
        self.state = Recording(matrix);
    }
    pub fn add_transition(&mut self, state1: &T, state2: &T) {
        if let Recording(mat) = &mut self.state {
            let s1 = self.mapped[state1];
            let s2 = self.mapped[state2];
            mat[s1][s2] += 1;
        } else {
            panic!("Attempting to add a transition to unfinalized builder");
        }
    }
    pub fn add_transition_by_id(&mut self, state1: usize, state2: usize) {
        if let Recording(mat) = &mut self.state {
            mat[state1][state2] += 1;
        } else {
            panic!("Attempting to add a transition to unfinalized builder");
        }
    }
    pub fn dump_matrix(&self, path: &Path) {
        if let Ok(mut f) = File::create(path) {
            if let Recording(mat) = &self.state {
                for i in 0..mat.len() {
                    for j in 0..mat.len() {
                        if let Err(e) = f.write(format!("{:5} ", mat[i][j]).as_bytes()) {
                            println!("[WARN] Error writing {}: {}", path.to_str().unwrap(), e);
                        }
                    }
                    if let Err(e) = f.write("\n".as_bytes()) {
                        println!("[WARN] Error writing {}: {}", path.to_str().unwrap(), e);
                    }
                }
            }
        } else {
            println!("[WARN] Unable to open file {}", path.to_str().unwrap());
        }
    }
}
