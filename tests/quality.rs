//! Algorithm quality tests: verify smarter algorithms produce better results.

use pick_distinct_colors::{
    calculate_metrics, pick_distinct_colors, Algorithm, Config, Rgb,
};

fn min_delta_e(colors: &[Rgb]) -> f64 {
    calculate_metrics(colors).min
}

#[test]
fn greedy_beats_random_statistically() {
    let mut greedy_wins = 0;
    let total = 20;

    for seed in 1..=(total as u32) {
        let greedy = pick_distinct_colors(
            Config::new(8)
                .algorithm(Algorithm::Greedy)
                .pool_size(100)
                .seed(seed),
        )
        .unwrap();
        let random = pick_distinct_colors(
            Config::new(8)
                .algorithm(Algorithm::Random)
                .pool_size(100)
                .seed(seed),
        )
        .unwrap();

        if min_delta_e(&greedy.colors) > min_delta_e(&random.colors) {
            greedy_wins += 1;
        }
    }

    assert!(
        greedy_wins >= 15,
        "Greedy should beat random in at least 15/20 seeds, got {greedy_wins}/20"
    );
}

#[test]
fn greedy_beats_random_for_fixed_seed() {
    let pool = pick_distinct_colors::generate_pool(100, 42);
    let greedy = pick_distinct_colors(
        Config::new(8)
            .algorithm(Algorithm::Greedy)
            .colors(pool.clone())
            .seed(42),
    )
    .unwrap();
    let random = pick_distinct_colors(
        Config::new(8)
            .algorithm(Algorithm::Random)
            .colors(pool)
            .seed(42),
    )
    .unwrap();

    assert!(
        min_delta_e(&greedy.colors) > min_delta_e(&random.colors),
        "Greedy min-dE ({}) should exceed random min-dE ({})",
        min_delta_e(&greedy.colors),
        min_delta_e(&random.colors)
    );
}

#[test]
fn exact_minimum_is_globally_optimal() {
    // For a small pool, exhaustively verify no better subset exists
    let pool = pick_distinct_colors::generate_pool(8, 42);
    let k = 3;

    let exact = pick_distinct_colors(
        Config::new(k)
            .algorithm(Algorithm::ExactMinimum)
            .colors(pool.clone()),
    )
    .unwrap();
    let exact_min = min_delta_e(&exact.colors);

    // Check all C(8,3) = 56 combinations
    for i in 0..pool.len() {
        for j in (i + 1)..pool.len() {
            for l in (j + 1)..pool.len() {
                let subset = vec![pool[i], pool[j], pool[l]];
                let subset_min = min_delta_e(&subset);
                assert!(
                    exact_min >= subset_min - 1e-10,
                    "Found better subset: {subset:?} with min-dE={subset_min} > exact {exact_min}"
                );
            }
        }
    }
}

#[test]
fn exact_minimum_beats_all_heuristics() {
    let pool = pick_distinct_colors::generate_pool(10, 42);
    let k = 3;

    let exact = pick_distinct_colors(
        Config::new(k)
            .algorithm(Algorithm::ExactMinimum)
            .colors(pool.clone()),
    )
    .unwrap();
    let exact_min = min_delta_e(&exact.colors);

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

    for &algo in &algorithms {
        let result = pick_distinct_colors(
            Config::new(k)
                .algorithm(algo)
                .colors(pool.clone())
                .seed(42),
        )
        .unwrap();
        let heur_min = min_delta_e(&result.colors);
        assert!(
            exact_min >= heur_min - 1e-10,
            "{algo:?} min-dE ({heur_min}) exceeds exact ({exact_min}) — impossible!"
        );
    }
}

#[test]
fn sa_beats_random_statistically() {
    let mut sa_wins = 0;
    let total = 10;

    for seed in 1..=(total as u32) {
        let sa = pick_distinct_colors(
            Config::new(5)
                .algorithm(Algorithm::SimulatedAnnealing)
                .pool_size(50)
                .seed(seed),
        )
        .unwrap();
        let random = pick_distinct_colors(
            Config::new(5)
                .algorithm(Algorithm::Random)
                .pool_size(50)
                .seed(seed),
        )
        .unwrap();

        if min_delta_e(&sa.colors) > min_delta_e(&random.colors) {
            sa_wins += 1;
        }
    }

    assert!(
        sa_wins >= 7,
        "SA should beat random in at least 7/10 seeds, got {sa_wins}/10"
    );
}

#[test]
fn max_sum_global_sum_higher_than_random() {
    let pool = pick_distinct_colors::generate_pool(100, 42);
    let global = pick_distinct_colors(
        Config::new(8)
            .algorithm(Algorithm::MaxSumGlobal)
            .colors(pool.clone())
            .seed(42),
    )
    .unwrap();
    let random = pick_distinct_colors(
        Config::new(8)
            .algorithm(Algorithm::Random)
            .colors(pool)
            .seed(42),
    )
    .unwrap();

    let global_sum = calculate_metrics(&global.colors).sum;
    let random_sum = calculate_metrics(&random.colors).sum;

    assert!(
        global_sum > random_sum,
        "MaxSumGlobal sum ({global_sum}) should exceed random sum ({random_sum})"
    );
}

#[test]
fn greedy_maximizes_spread_with_clear_outliers() {
    // Pool: 48 near-identical grays + black + white
    let mut pool: Vec<Rgb> = (0..48)
        .map(|i| Rgb::new(128 + (i % 3) as u8, 128 + (i % 2) as u8, 128))
        .collect();
    pool.push(Rgb::new(0, 0, 0));
    pool.push(Rgb::new(255, 255, 255));

    // With k=3, greedy must include both black and white since they're
    // the most distant from each other and from the grays
    let result = pick_distinct_colors(
        Config::new(3)
            .algorithm(Algorithm::Greedy)
            .colors(pool)
            .seed(42),
    )
    .unwrap();

    let has_black = result.colors.iter().any(|c| c.r == 0 && c.g == 0 && c.b == 0);
    let has_white = result.colors.iter().any(|c| c.r == 255 && c.g == 255 && c.b == 255);
    assert!(
        has_black && has_white,
        "Greedy k=3 should include both black and white, got {:?}",
        result.colors
    );
}
