/// Errors that can occur during color selection.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("count ({count}) exceeds pool size ({pool_size})")]
    CountExceedsPool { count: usize, pool_size: usize },

    #[error("count must be at least 1")]
    ZeroCount,

    #[error("pool must contain at least 1 color")]
    EmptyPool,

    #[error("exact algorithm infeasible: C({n}, {k}) exceeds limit {limit}")]
    ExactInfeasible { n: usize, k: usize, limit: u128 },

    #[error("unknown algorithm: {0}")]
    UnknownAlgorithm(String),
}

/// Convenience Result type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Compute C(n, k) as u128, returning None on overflow.
pub fn combinations_count(n: usize, k: usize) -> Option<u128> {
    if k > n {
        return Some(0);
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    for i in 0..k {
        result = result.checked_mul((n - i) as u128)?;
        result /= (i + 1) as u128;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinations_count_base_cases() {
        assert_eq!(combinations_count(0, 0), Some(1));
        assert_eq!(combinations_count(1, 0), Some(1));
        assert_eq!(combinations_count(5, 0), Some(1));
        assert_eq!(combinations_count(5, 5), Some(1));
        assert_eq!(combinations_count(1, 1), Some(1));
    }

    #[test]
    fn combinations_count_known_values() {
        assert_eq!(combinations_count(10, 3), Some(120));
        assert_eq!(combinations_count(20, 6), Some(38760));
        assert_eq!(combinations_count(52, 5), Some(2598960));
        assert_eq!(combinations_count(10, 1), Some(10));
        assert_eq!(combinations_count(10, 9), Some(10));
    }

    #[test]
    fn combinations_count_k_greater_than_n() {
        assert_eq!(combinations_count(3, 5), Some(0));
        assert_eq!(combinations_count(0, 1), Some(0));
    }

    #[test]
    fn combinations_count_symmetry() {
        for n in 0..20 {
            for k in 0..=n {
                assert_eq!(
                    combinations_count(n, k),
                    combinations_count(n, n - k),
                    "C({n},{k}) != C({n},{})",
                    n - k
                );
            }
        }
    }

    #[test]
    fn combinations_count_large_feasible() {
        assert_eq!(combinations_count(100, 2), Some(4950));
        assert_eq!(combinations_count(30, 10), Some(30045015));
    }

    #[test]
    fn combinations_count_overflow_returns_none() {
        // C(200, 100) is astronomically large — should overflow u128
        assert_eq!(combinations_count(200, 100), None);
    }
}
