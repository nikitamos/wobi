use std::hash::{Hash, Hasher};

pub struct MarkovChain<T: Hash> {
    matrix: Vec<Vec<f32>>,
    data: Vec<(usize, T)>,
    current_state: usize
}
impl<T: Hash> MarkovChain<T> {
    pub fn state_count() -> usize {
      todo!()
    }
    pub fn reset(state: T) {

    }
    pub fn next_state() -> Option<T> {

      Some(1)
    }
    fn hash() {
    }
}
