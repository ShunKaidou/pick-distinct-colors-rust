//! Cross-validation: verify Rust Mulberry32 PRNG matches JS implementation exactly.

use pick_distinct_colors::{generate_pool, Mulberry32, Rgb};

#[test]
fn mulberry32_seed0_first10_match_js() {
    let mut rng = Mulberry32::new(0);
    let expected: [u32; 10] = [
        1144304738, 1416247, 958946056, 627933444, 2007157716,
        2340967985, 2642484575, 2787370982, 1958536065, 2496316458,
    ];
    for (i, &exp) in expected.iter().enumerate() {
        assert_eq!(rng.next_u32(), exp, "seed=0, index={i} mismatch");
    }
}

#[test]
fn mulberry32_seed42_first10_match_js() {
    let mut rng = Mulberry32::new(42);
    let expected: [u32; 10] = [
        2581720956, 1925393290, 3661312704, 2876485805, 750819978,
        2261697747, 1173505300, 2683257857, 3717185310, 2028586305,
    ];
    for (i, &exp) in expected.iter().enumerate() {
        assert_eq!(rng.next_u32(), exp, "seed=42, index={i} mismatch");
    }
}

#[test]
fn mulberry32_seed_deadbeef_first10_match_js() {
    let mut rng = Mulberry32::new(0xDEADBEEF);
    let expected: [u32; 10] = [
        4043151706, 1147597007, 3315858022, 1538288752, 2042435954,
        3600176436, 484360372, 1362401224, 379893202, 1051950098,
    ];
    for (i, &exp) in expected.iter().enumerate() {
        assert_eq!(rng.next_u32(), exp, "seed=0xDEADBEEF, index={i} mismatch");
    }
}

#[test]
fn next_f64_equals_u32_divided_by_2pow32() {
    let mut rng1 = Mulberry32::new(42);
    let mut rng2 = Mulberry32::new(42);
    for _ in 0..100 {
        let u = rng1.next_u32();
        let f = rng2.next_f64();
        assert_eq!(f, u as f64 / 4294967296.0);
    }
}

#[test]
fn next_u8_matches_js_floor_prng_times_256() {
    let mut rng1 = Mulberry32::new(42);
    let mut rng2 = Mulberry32::new(42);
    for _ in 0..100 {
        let u8_val = rng1.next_u8();
        let f64_val = rng2.next_f64();
        assert_eq!(u8_val, (f64_val * 256.0) as u8);
    }
}

#[test]
fn generate_pool_seed42_size5_matches_js() {
    let pool = generate_pool(5, 42);
    let expected = vec![
        Rgb::new(153, 114, 218),
        Rgb::new(171, 44, 134),
        Rgb::new(69, 159, 221),
        Rgb::new(120, 63, 225),
        Rgb::new(190, 78, 50),
    ];
    assert_eq!(pool, expected, "Pool generation must match JS output");
}

#[test]
fn generate_pool_seed42_size10_matches_js() {
    let pool = generate_pool(10, 42);
    let expected = vec![
        Rgb::new(153, 114, 218),
        Rgb::new(171, 44, 134),
        Rgb::new(69, 159, 221),
        Rgb::new(120, 63, 225),
        Rgb::new(190, 78, 50),
        Rgb::new(128, 175, 156),
        Rgb::new(0, 120, 214),
        Rgb::new(13, 151, 8),
        Rgb::new(68, 15, 47),
        Rgb::new(200, 135, 6),
    ];
    assert_eq!(pool, expected, "Pool generation must match JS output");
}

#[test]
fn mulberry32_state_wrapping_no_panic() {
    // Exercise wrapping arithmetic with edge-case seeds
    for seed in [0, 1, u32::MAX, u32::MAX - 1, 0x6D2B79F5] {
        let mut rng = Mulberry32::new(seed);
        for _ in 0..1000 {
            let _ = rng.next_u32();
        }
    }
}

#[test]
fn next_index_never_reaches_n() {
    let mut rng = Mulberry32::new(42);
    for n in [1, 2, 3, 7, 100, 256] {
        for _ in 0..10000 {
            let idx = rng.next_index(n);
            assert!(idx < n, "next_index({n}) returned {idx}");
        }
    }
}

#[test]
fn next_u8_range() {
    let mut rng = Mulberry32::new(42);
    for _ in 0..10000 {
        let v = rng.next_u8();
        let _ = v; // exercises the PRNG path without panic
    }
}
