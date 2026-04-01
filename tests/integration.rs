use pick_distinct_colors::{pick_distinct_colors, Algorithm, Config};

#[test]
fn test_greedy_basic() {
    let result = pick_distinct_colors(Config::new(5).algorithm(Algorithm::Greedy).seed(42)).unwrap();
    assert_eq!(result.colors.len(), 5);
    assert!(result.time_ms >= 0.0);
}

#[test]
fn test_all_algorithms_produce_correct_count() {
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
        Algorithm::Random,
    ];

    for algo in &algorithms {
        let result = pick_distinct_colors(
            Config::new(5).algorithm(*algo).seed(42).pool_size(50),
        )
        .unwrap();
        assert_eq!(
            result.colors.len(),
            5,
            "Algorithm {:?} returned wrong count",
            algo
        );
    }
}

#[test]
fn test_deterministic_with_same_seed() {
    let r1 = pick_distinct_colors(Config::new(8).seed(12345)).unwrap();
    let r2 = pick_distinct_colors(Config::new(8).seed(12345)).unwrap();
    assert_eq!(r1.colors, r2.colors);
}

#[test]
fn test_different_seeds_different_results() {
    let r1 = pick_distinct_colors(Config::new(8).seed(1)).unwrap();
    let r2 = pick_distinct_colors(Config::new(8).seed(2)).unwrap();
    assert_ne!(r1.colors, r2.colors);
}

#[test]
fn test_exact_minimum_small() {
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(10)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 3);
}

#[test]
fn test_exact_maximum_small() {
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::ExactMaximum)
            .pool_size(10)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 3);
}

#[test]
fn test_error_count_exceeds_pool() {
    let result = pick_distinct_colors(Config::new(200).pool_size(10));
    assert!(result.is_err());
}

#[test]
fn test_error_zero_count() {
    let result = pick_distinct_colors(Config::new(0));
    assert!(result.is_err());
}

#[test]
fn test_custom_color_pool() {
    use pick_distinct_colors::Rgb;
    let pool = vec![
        Rgb::new(255, 0, 0),
        Rgb::new(0, 255, 0),
        Rgb::new(0, 0, 255),
        Rgb::new(255, 255, 0),
        Rgb::new(0, 255, 255),
        Rgb::new(255, 0, 255),
    ];
    let result = pick_distinct_colors(Config::new(3).colors(pool)).unwrap();
    assert_eq!(result.colors.len(), 3);
}

#[test]
fn test_algorithm_from_str() {
    use std::str::FromStr;
    assert_eq!(Algorithm::from_str("greedy").unwrap(), Algorithm::Greedy);
    assert_eq!(
        Algorithm::from_str("simulatedAnnealing").unwrap(),
        Algorithm::SimulatedAnnealing
    );
    assert_eq!(
        Algorithm::from_str("kmeansppSelection").unwrap(),
        Algorithm::KMeansPP
    );
    assert!(Algorithm::from_str("nonexistent").is_err());
}
