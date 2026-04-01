//! Property-based invariant tests: universal properties that must hold for ALL algorithms.

use pick_distinct_colors::{
    pick_distinct_colors, Algorithm, Config, Rgb,
};

const ALL_ALGORITHMS: [Algorithm; 12] = [
    Algorithm::Greedy,
    Algorithm::MaxSumGlobal,
    Algorithm::MaxSumSequential,
    Algorithm::KMeansPP,
    Algorithm::SimulatedAnnealing,
    Algorithm::GeneticAlgorithm,
    Algorithm::ParticleSwarm,
    Algorithm::AntColony,
    Algorithm::TabuSearch,
    Algorithm::ExactMinimum,
    Algorithm::ExactMaximum,
    Algorithm::Random,
];

/// Algorithms that don't blow up on larger pools (exclude exact for large pools).
const HEURISTIC_ALGORITHMS: [Algorithm; 10] = [
    Algorithm::Greedy,
    Algorithm::MaxSumGlobal,
    Algorithm::MaxSumSequential,
    Algorithm::KMeansPP,
    Algorithm::SimulatedAnnealing,
    Algorithm::GeneticAlgorithm,
    Algorithm::ParticleSwarm,
    Algorithm::AntColony,
    Algorithm::TabuSearch,
    Algorithm::Random,
];

fn make_pool(size: usize) -> Vec<Rgb> {
    pick_distinct_colors::generate_pool(size, 42)
}

#[test]
fn prop_output_length_equals_count() {
    for &algo in &ALL_ALGORITHMS {
        for count in [1, 2, 5, 10] {
            let result = pick_distinct_colors(
                Config::new(count)
                    .algorithm(algo)
                    .pool_size(20)
                    .seed(42),
            )
            .unwrap();
            assert_eq!(
                result.colors.len(),
                count,
                "{algo:?} count={count}: wrong output length"
            );
        }
    }
}

#[test]
fn prop_all_output_colors_exist_in_pool() {
    let pool = make_pool(50);
    for &algo in &HEURISTIC_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(42),
        )
        .unwrap();
        for color in &result.colors {
            assert!(
                pool.contains(color),
                "{algo:?}: output color {color:?} not in pool"
            );
        }
    }
}

#[test]
fn prop_no_duplicate_colors_in_output() {
    let pool = make_pool(100);
    for &algo in &HEURISTIC_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(10)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(42),
        )
        .unwrap();
        let mut sorted = result.colors.clone();
        sorted.sort_by_key(|c| (c.r, c.g, c.b));
        let before_len = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            before_len,
            "{algo:?}: output contains duplicates"
        );
    }
}

#[test]
fn prop_output_sorted_by_lab_l_desc() {
    let pool = make_pool(100);
    for &algo in &HEURISTIC_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(8)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(42),
        )
        .unwrap();
        for i in 0..result.colors.len() - 1 {
            let l_curr = result.colors[i].to_lab().l;
            let l_next = result.colors[i + 1].to_lab().l;
            assert!(
                l_curr >= l_next - 1e-10,
                "{algo:?}: output not sorted by L desc at index {i}: {l_curr} < {l_next}"
            );
        }
    }
}

#[test]
fn prop_deterministic_same_seed() {
    let pool = make_pool(50);
    for &algo in &HEURISTIC_ALGORITHMS {
        let r1 = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(99),
        )
        .unwrap();
        let r2 = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(99),
        )
        .unwrap();
        assert_eq!(
            r1.colors, r2.colors,
            "{algo:?}: not deterministic with same seed"
        );
    }
}

#[test]
fn prop_count_1_works() {
    for &algo in &ALL_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(1)
                .algorithm(algo)
                .pool_size(20)
                .seed(42),
        )
        .unwrap();
        assert_eq!(result.colors.len(), 1, "{algo:?}: count=1 failed");
    }
}

#[test]
fn prop_count_equals_pool_size() {
    let pool = make_pool(10);
    for &algo in &ALL_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(10)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(42),
        )
        .unwrap();
        assert_eq!(
            result.colors.len(),
            10,
            "{algo:?}: count==pool_size failed"
        );
        // All pool colors should be present (sorted differently)
        for color in &pool {
            assert!(
                result.colors.contains(color),
                "{algo:?}: pool color {color:?} missing from output"
            );
        }
    }
}

#[test]
fn prop_time_ms_non_negative() {
    for &algo in &HEURISTIC_ALGORITHMS {
        let result = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .pool_size(30)
                .seed(42),
        )
        .unwrap();
        assert!(
            result.time_ms >= 0.0,
            "{algo:?}: negative time_ms"
        );
    }
}

#[test]
fn prop_different_seeds_generally_differ() {
    for &algo in &HEURISTIC_ALGORITHMS {
        // Skip max_sum_global — it's deterministic regardless of seed (ranks by total distance)
        if algo == Algorithm::MaxSumGlobal {
            continue;
        }
        let r1 = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .pool_size(50)
                .seed(1),
        )
        .unwrap();
        let r2 = pick_distinct_colors(
            Config::new(5)
                .algorithm(algo)
                .pool_size(50)
                .seed(2),
        )
        .unwrap();
        // Different seeds with different pools should produce different results
        // (seeds affect pool generation too)
        assert_ne!(
            r1.colors, r2.colors,
            "{algo:?}: same output with different seeds"
        );
    }
}
