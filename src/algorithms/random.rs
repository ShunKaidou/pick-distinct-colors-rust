use crate::prng::{random_sample_indices, Mulberry32};

/// Random selection without replacement.
pub(crate) fn run(n: usize, select_count: usize, rng: &mut Mulberry32) -> Vec<usize> {
    random_sample_indices(n, select_count, rng)
}
