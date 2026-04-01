//! Edge case tests: boundary conditions, degenerate inputs, error paths.

use pick_distinct_colors::{
    pick_distinct_colors, Algorithm, AlgorithmOptions, Config, Rgb,
};

// --- Error paths ---

#[test]
fn count_zero_returns_error() {
    let result = pick_distinct_colors(Config::new(0));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("at least 1"), "Error message: {err}");
}

#[test]
fn empty_pool_returns_error() {
    let result = pick_distinct_colors(Config::new(3).colors(vec![]));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("at least 1 color"), "Error message: {err}");
}

#[test]
fn count_exceeds_pool_returns_error() {
    let result = pick_distinct_colors(Config::new(200).pool_size(10));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("exceeds pool"), "Error message: {err}");
}

#[test]
fn exact_infeasible_returns_error() {
    // C(30, 10) = 30045015 which exceeds default limit of 1_000_000
    let result = pick_distinct_colors(
        Config::new(10)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(30)
            .seed(42),
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("infeasible"), "Error message: {err}");
}

#[test]
fn exact_infeasible_maximum_returns_error() {
    let result = pick_distinct_colors(
        Config::new(10)
            .algorithm(Algorithm::ExactMaximum)
            .pool_size(30)
            .seed(42),
    );
    assert!(result.is_err());
}

#[test]
fn exact_custom_limit_allows_large() {
    let opts = AlgorithmOptions {
        exact_limit: Some(100_000_000),
        ..Default::default()
    };
    // C(15, 3) = 455 which is under the custom limit
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(15)
            .seed(42)
            .options(opts),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap().colors.len(), 3);
}

// --- Boundary counts ---

#[test]
fn pool_size_1_count_1() {
    let pool = vec![Rgb::new(128, 64, 32)];
    let result = pick_distinct_colors(Config::new(1).colors(pool.clone())).unwrap();
    assert_eq!(result.colors.len(), 1);
    assert_eq!(result.colors[0], pool[0]);
}

#[test]
fn count_equals_pool_size_returns_all() {
    let pool = vec![
        Rgb::new(255, 0, 0),
        Rgb::new(0, 255, 0),
        Rgb::new(0, 0, 255),
    ];
    let result = pick_distinct_colors(Config::new(3).colors(pool.clone())).unwrap();
    assert_eq!(result.colors.len(), 3);
    // All pool colors must be present
    for c in &pool {
        assert!(result.colors.contains(c));
    }
}

#[test]
fn count_2_minimum_viable() {
    let pool = vec![Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)];
    let result = pick_distinct_colors(Config::new(2).colors(pool)).unwrap();
    assert_eq!(result.colors.len(), 2);
}

// --- Duplicate colors in pool ---

#[test]
fn pool_all_identical_colors_count_1() {
    let pool = vec![Rgb::new(128, 128, 128); 100];
    let result = pick_distinct_colors(Config::new(1).colors(pool)).unwrap();
    assert_eq!(result.colors.len(), 1);
    assert_eq!(result.colors[0], Rgb::new(128, 128, 128));
}

#[test]
fn pool_all_identical_colors_count_3() {
    let pool = vec![Rgb::new(128, 128, 128); 100];
    let result = pick_distinct_colors(
        Config::new(3).colors(pool).algorithm(Algorithm::Greedy),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 3);
    for c in &result.colors {
        assert_eq!(*c, Rgb::new(128, 128, 128));
    }
}

#[test]
fn pool_all_identical_sa() {
    let pool = vec![Rgb::new(100, 100, 100); 20];
    let result = pick_distinct_colors(
        Config::new(3)
            .colors(pool)
            .algorithm(Algorithm::SimulatedAnnealing)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 3);
}

#[test]
fn pool_all_identical_kmeans() {
    let pool = vec![Rgb::new(100, 100, 100); 20];
    let result = pick_distinct_colors(
        Config::new(3)
            .colors(pool)
            .algorithm(Algorithm::KMeansPP)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 3);
}

// --- Near-identical colors ---

#[test]
fn pool_near_identical_greedy() {
    // 50 near-identical grays + black + white
    let mut pool: Vec<Rgb> = (0..50)
        .map(|i| Rgb::new(128 + (i % 2) as u8, 128 + (i % 3) as u8, 128))
        .collect();
    pool.push(Rgb::new(0, 0, 0));
    pool.push(Rgb::new(255, 255, 255));

    let greedy = pick_distinct_colors(
        Config::new(2)
            .colors(pool.clone())
            .algorithm(Algorithm::Greedy)
            .seed(42),
    )
    .unwrap();

    // Greedy's min-distance should be much higher than picking 2 grays
    let gray_pair = vec![Rgb::new(128, 128, 128), Rgb::new(129, 129, 128)];
    let greedy_min = pick_distinct_colors::calculate_metrics(&greedy.colors).min;
    let gray_min = pick_distinct_colors::calculate_metrics(&gray_pair).min;
    assert!(
        greedy_min > gray_min * 10.0,
        "Greedy min-dE ({greedy_min}) should far exceed gray pair ({gray_min})"
    );
}

// --- Large pool sizes ---

#[test]
fn large_pool_1000_greedy_k20() {
    let result = pick_distinct_colors(
        Config::new(20)
            .algorithm(Algorithm::Greedy)
            .pool_size(1000)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 20);
}

#[test]
fn large_pool_greedy_k1() {
    let result = pick_distinct_colors(
        Config::new(1)
            .algorithm(Algorithm::Greedy)
            .pool_size(5000)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 1);
}

// --- Default pool size formula ---

#[test]
fn default_pool_size_count_1() {
    // count=1 → pool_size = max(16, 128) = 128
    let result = pick_distinct_colors(Config::new(1).seed(42)).unwrap();
    assert_eq!(result.colors.len(), 1);
}

#[test]
fn default_pool_size_count_10() {
    // count=10 → pool_size = max(160, 128) = 160
    let result = pick_distinct_colors(Config::new(10).seed(42)).unwrap();
    assert_eq!(result.colors.len(), 10);
}

// --- Exact algorithms at boundary ---

#[test]
fn exact_minimum_k_equals_1() {
    let result = pick_distinct_colors(
        Config::new(1)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(10)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 1);
}

#[test]
fn exact_minimum_k_equals_2() {
    let result = pick_distinct_colors(
        Config::new(2)
            .algorithm(Algorithm::ExactMinimum)
            .pool_size(10)
            .seed(42),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 2);
}

#[test]
fn exact_minimum_k_equals_pool_size() {
    let pool = pick_distinct_colors::generate_pool(5, 42);
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::ExactMinimum)
            .colors(pool.clone()),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
    for c in &pool {
        assert!(result.colors.contains(c));
    }
}

// --- Algorithm with custom options ---

#[test]
fn sa_with_custom_options() {
    let opts = AlgorithmOptions {
        initial_temp: Some(500.0),
        cooling_rate: Some(0.99),
        min_temp: Some(0.5),
        ..Default::default()
    };
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::SimulatedAnnealing)
            .pool_size(30)
            .seed(42)
            .options(opts),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
}

#[test]
fn genetic_with_custom_options() {
    let opts = AlgorithmOptions {
        population_size: Some(20),
        generations: Some(10),
        mutation_rate: Some(0.2),
        ..Default::default()
    };
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::GeneticAlgorithm)
            .pool_size(30)
            .seed(42)
            .options(opts),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
}

#[test]
fn tabu_with_custom_options() {
    let opts = AlgorithmOptions {
        tabu_iterations: Some(100),
        tabu_tenure: Some(3),
        ..Default::default()
    };
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::TabuSearch)
            .pool_size(30)
            .seed(42)
            .options(opts),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
}

#[test]
fn pso_with_custom_options() {
    let opts = AlgorithmOptions {
        num_particles: Some(10),
        pso_iterations: Some(20),
        ..Default::default()
    };
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::ParticleSwarm)
            .pool_size(30)
            .seed(42)
            .options(opts),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
}

#[test]
fn aco_with_custom_options() {
    let opts = AlgorithmOptions {
        num_ants: Some(10),
        aco_iterations: Some(20),
        ..Default::default()
    };
    let result = pick_distinct_colors(
        Config::new(5)
            .algorithm(Algorithm::AntColony)
            .pool_size(30)
            .seed(42)
            .options(opts),
    )
    .unwrap();
    assert_eq!(result.colors.len(), 5);
}
