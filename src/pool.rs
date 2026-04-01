use crate::color::Rgb;
use crate::prng::Mulberry32;

/// Generate a pool of random RGB colors using the Mulberry32 PRNG.
///
/// Matches the JS behavior: `Array.from({ length: size }, () => randomColor(prng))`
/// where `randomColor` generates `[Math.floor(prng() * 256), ...]`.
pub fn generate_pool(size: usize, seed: u32) -> Vec<Rgb> {
    let mut rng = Mulberry32::new(seed);
    (0..size)
        .map(|_| Rgb::new(rng.next_u8(), rng.next_u8(), rng.next_u8()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pool_size() {
        let pool = generate_pool(100, 42);
        assert_eq!(pool.len(), 100);
    }

    #[test]
    fn test_generate_pool_deterministic() {
        let pool1 = generate_pool(50, 12345);
        let pool2 = generate_pool(50, 12345);
        assert_eq!(pool1, pool2);
    }

    #[test]
    fn test_generate_pool_different_seeds() {
        let pool1 = generate_pool(50, 1);
        let pool2 = generate_pool(50, 2);
        assert_ne!(pool1, pool2);
    }
}
