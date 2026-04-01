/// Mulberry32 seedable PRNG — bit-identical to the JS implementation.
///
/// Produces the same sequence as:
/// ```js
/// function mulberry32(seed) {
///   let t = seed >>> 0;
///   return function() {
///     t += 0x6D2B79F5;
///     let r = Math.imul(t ^ t >>> 15, 1 | t);
///     r ^= r + Math.imul(r ^ r >>> 7, 61 | r);
///     return ((r ^ r >>> 14) >>> 0) / 4294967296;
///   };
/// }
/// ```
pub struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Returns next u32 in the sequence.
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6D2B79F5);
        let mut r = (self.state ^ (self.state >> 15)).wrapping_mul(1 | self.state);
        r ^= r.wrapping_add((r ^ (r >> 7)).wrapping_mul(61 | r));
        r ^ (r >> 14)
    }

    /// Returns next f64 in [0, 1), matching JS output exactly.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / 4294967296.0
    }

    /// Random index in [0, n).
    #[inline]
    pub fn next_index(&mut self, n: usize) -> usize {
        (self.next_f64() * n as f64) as usize
    }

    /// Random u8 in [0, 256) — matches JS `Math.floor(prng() * 256)`.
    #[inline]
    pub fn next_u8(&mut self) -> u8 {
        (self.next_f64() * 256.0) as u8
    }
}

/// Fisher-Yates shuffle using Mulberry32 PRNG.
pub fn fisher_yates_shuffle<T>(slice: &mut [T], rng: &mut Mulberry32) {
    let n = slice.len();
    for i in (1..n).rev() {
        let j = rng.next_index(i + 1);
        slice.swap(i, j);
    }
}

/// Generate `k` random unique indices from `[0, n)` using Fisher-Yates partial shuffle.
pub fn random_sample_indices(n: usize, k: usize, rng: &mut Mulberry32) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..n).collect();
    for i in 0..k {
        let j = i + rng.next_index(n - i);
        indices.swap(i, j);
    }
    indices.truncate(k);
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mulberry32_deterministic() {
        let mut rng1 = Mulberry32::new(42);
        let mut rng2 = Mulberry32::new(42);
        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }

    #[test]
    fn test_mulberry32_range() {
        let mut rng = Mulberry32::new(12345);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn test_mulberry32_different_seeds() {
        let mut rng1 = Mulberry32::new(1);
        let mut rng2 = Mulberry32::new(2);
        // Very unlikely to produce same first value
        assert_ne!(rng1.next_u32(), rng2.next_u32());
    }

    #[test]
    fn test_fisher_yates() {
        let mut data: Vec<usize> = (0..10).collect();
        let original = data.clone();
        let mut rng = Mulberry32::new(42);
        fisher_yates_shuffle(&mut data, &mut rng);
        // Should be a permutation (same elements)
        let mut sorted = data.clone();
        sorted.sort();
        assert_eq!(sorted, original);
        // Should (very likely) differ from original order
        assert_ne!(data, original);
    }

    #[test]
    fn test_random_sample_indices() {
        let mut rng = Mulberry32::new(42);
        let sample = random_sample_indices(100, 10, &mut rng);
        assert_eq!(sample.len(), 10);
        // All unique
        let mut unique = sample.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 10);
        // All in range
        for &idx in &sample {
            assert!(idx < 100);
        }
    }

    #[test]
    fn test_fisher_yates_deterministic() {
        let mut data1: Vec<usize> = (0..20).collect();
        let mut data2: Vec<usize> = (0..20).collect();
        fisher_yates_shuffle(&mut data1, &mut Mulberry32::new(99));
        fisher_yates_shuffle(&mut data2, &mut Mulberry32::new(99));
        assert_eq!(data1, data2);
    }
}
