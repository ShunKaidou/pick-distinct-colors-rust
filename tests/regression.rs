//! Regression tests: pinned outputs for specific configurations.
//! If any of these fail after a code change, it means behavior changed.

use pick_distinct_colors::{
    pick_distinct_colors, Algorithm, Config, Rgb,
};
use std::str::FromStr;

#[test]
fn regression_greedy_seed42_count5_default_pool() {
    let result = pick_distinct_colors(Config::new(5).algorithm(Algorithm::Greedy).seed(42)).unwrap();
    let expected = vec![
        Rgb::new(37, 241, 17),
        Rgb::new(222, 177, 38),
        Rgb::new(74, 112, 145),
        Rgb::new(144, 24, 247),
        Rgb::new(190, 13, 17),
    ];
    assert_eq!(result.colors, expected, "greedy seed=42 count=5 regression");
}

#[test]
fn regression_random_seed42_count5_default_pool() {
    let result = pick_distinct_colors(Config::new(5).algorithm(Algorithm::Random).seed(42)).unwrap();
    let expected = vec![
        Rgb::new(254, 144, 234),
        Rgb::new(25, 210, 14),
        Rgb::new(97, 182, 233),
        Rgb::new(70, 161, 197),
        Rgb::new(74, 112, 145),
    ];
    assert_eq!(result.colors, expected, "random seed=42 count=5 regression");
}

#[test]
fn regression_exact_min_seed42_count3_pool10() {
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(10)
            .seed(42),
    )
    .unwrap();
    let expected = vec![
        Rgb::new(13, 151, 8),
        Rgb::new(190, 78, 50),
        Rgb::new(120, 63, 225),
    ];
    assert_eq!(result.colors, expected, "exact_min seed=42 count=3 pool=10 regression");
}

#[test]
fn regression_greedy_seed1_count3_pool20() {
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::Greedy)
            .pool_size(20)
            .seed(1),
    )
    .unwrap();
    let expected = vec![
        Rgb::new(116, 197, 254),
        Rgb::new(61, 193, 39),
        Rgb::new(194, 49, 184),
    ];
    assert_eq!(result.colors, expected, "greedy seed=1 count=3 pool=20 regression");
}

#[test]
fn regression_pool_generation_seed42_size10() {
    let pool = pick_distinct_colors::generate_pool(10, 42);
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
    assert_eq!(pool, expected);
}

#[test]
fn regression_algorithm_display_roundtrip() {
    let algorithms = [
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
    for algo in &algorithms {
        let s = algo.to_string();
        let parsed = Algorithm::from_str(&s).unwrap();
        assert_eq!(*algo, parsed, "Display→FromStr roundtrip failed for {s}");
    }
}

#[test]
fn regression_fromstr_all_aliases() {
    // snake_case
    assert_eq!(Algorithm::from_str("greedy").unwrap(), Algorithm::Greedy);
    assert_eq!(Algorithm::from_str("max_sum_global").unwrap(), Algorithm::MaxSumGlobal);
    assert_eq!(Algorithm::from_str("max_sum_sequential").unwrap(), Algorithm::MaxSumSequential);
    assert_eq!(Algorithm::from_str("kmeans_pp").unwrap(), Algorithm::KMeansPP);
    assert_eq!(Algorithm::from_str("simulated_annealing").unwrap(), Algorithm::SimulatedAnnealing);
    assert_eq!(Algorithm::from_str("genetic_algorithm").unwrap(), Algorithm::GeneticAlgorithm);
    assert_eq!(Algorithm::from_str("particle_swarm").unwrap(), Algorithm::ParticleSwarm);
    assert_eq!(Algorithm::from_str("ant_colony").unwrap(), Algorithm::AntColony);
    assert_eq!(Algorithm::from_str("tabu_search").unwrap(), Algorithm::TabuSearch);
    assert_eq!(Algorithm::from_str("exact_minimum").unwrap(), Algorithm::ExactMinimum);
    assert_eq!(Algorithm::from_str("exact_maximum").unwrap(), Algorithm::ExactMaximum);
    assert_eq!(Algorithm::from_str("random").unwrap(), Algorithm::Random);

    // camelCase (JS compat)
    assert_eq!(Algorithm::from_str("maxSumDistancesGlobal").unwrap(), Algorithm::MaxSumGlobal);
    assert_eq!(Algorithm::from_str("maxSumDistancesSequential").unwrap(), Algorithm::MaxSumSequential);
    assert_eq!(Algorithm::from_str("kmeansppSelection").unwrap(), Algorithm::KMeansPP);
    assert_eq!(Algorithm::from_str("simulatedAnnealing").unwrap(), Algorithm::SimulatedAnnealing);
    assert_eq!(Algorithm::from_str("geneticAlgorithm").unwrap(), Algorithm::GeneticAlgorithm);
    assert_eq!(Algorithm::from_str("particleSwarmOptimization").unwrap(), Algorithm::ParticleSwarm);
    assert_eq!(Algorithm::from_str("antColonyOptimization").unwrap(), Algorithm::AntColony);
    assert_eq!(Algorithm::from_str("tabuSearch").unwrap(), Algorithm::TabuSearch);
    assert_eq!(Algorithm::from_str("exactMinimum").unwrap(), Algorithm::ExactMinimum);
    assert_eq!(Algorithm::from_str("exactMaximum").unwrap(), Algorithm::ExactMaximum);
    assert_eq!(Algorithm::from_str("randomSelection").unwrap(), Algorithm::Random);

    // Short aliases
    assert_eq!(Algorithm::from_str("kmeanspp").unwrap(), Algorithm::KMeansPP);
    assert_eq!(Algorithm::from_str("genetic").unwrap(), Algorithm::GeneticAlgorithm);
    assert_eq!(Algorithm::from_str("tabu").unwrap(), Algorithm::TabuSearch);

    // Invalid
    assert!(Algorithm::from_str("nonexistent").is_err());
    assert!(Algorithm::from_str("").is_err());
}
