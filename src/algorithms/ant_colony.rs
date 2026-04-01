use crate::algorithms::AlgorithmOptions;
use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};
use crate::prng::Mulberry32;

/// Get distance between two colors, using matrix if available.
#[inline]
fn get_distance(i: usize, j: usize, labs: &[Lab], dm: Option<&DistanceMatrix>) -> f64 {
    match dm {
        Some(matrix) => matrix.get(i, j),
        None => delta_e(&labs[i], &labs[j]),
    }
}

/// Compute minimum pairwise distance for a selection.
fn calculate_fitness(selection: &[usize], labs: &[Lab], dm: Option<&DistanceMatrix>) -> f64 {
    let mut min_dist = f64::INFINITY;
    for i in 0..selection.len() {
        for j in (i + 1)..selection.len() {
            let d = get_distance(selection[i], selection[j], labs, dm);
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

/// Construct a single ant's solution via probabilistic construction.
fn construct_ant_solution(
    n: usize,
    select_count: usize,
    labs: &[Lab],
    dm: Option<&DistanceMatrix>,
    pheromones: &[f64],
    alpha: f64,
    beta: f64,
    rng: &mut Mulberry32,
) -> Vec<usize> {
    let mut solution = Vec::with_capacity(select_count);
    let mut in_solution = vec![false; n];

    // Randomly select first color
    let first = rng.next_index(n);
    solution.push(first);
    in_solution[first] = true;

    // Select remaining colors via probabilistic construction
    while solution.len() < select_count {
        let mut candidates = Vec::with_capacity(n - solution.len());
        let mut cumulative = 0.0;

        for i in 0..n {
            if in_solution[i] {
                continue;
            }

            let pheromone = pheromones[i].powf(alpha);

            // Heuristic: minimum distance to already-selected colors
            let min_dist = solution
                .iter()
                .map(|&j| get_distance(i, j, labs, dm))
                .fold(f64::INFINITY, f64::min);
            let heuristic = min_dist.powf(beta);

            let weight = pheromone * heuristic;
            cumulative += weight;
            candidates.push((i, cumulative));
        }

        if candidates.is_empty() {
            break;
        }

        let target = rng.next_f64() * cumulative;
        let chosen = match candidates.binary_search_by(|&(_, cum)| {
            cum.partial_cmp(&target).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            Ok(pos) => candidates[pos].0,
            Err(pos) => candidates[pos.min(candidates.len() - 1)].0,
        };

        solution.push(chosen);
        in_solution[chosen] = true;
    }

    solution
}

/// Ant Colony Optimization for color selection.
///
/// Fixes from JS:
/// - Uses precomputed distance matrix (JS recomputed deltaE despite having a matrix)
/// - Fitness-proportional pheromone deposit (JS deposited equally regardless of quality)
/// - Fixed roulette wheel with proper clamping (no OOB risk)
/// - Vec<bool> for O(1) membership
/// - Optional parallel ant construction via Rayon
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    options: &AlgorithmOptions,
    rng: &mut Mulberry32,
    dist_matrix: Option<&DistanceMatrix>,
) -> Vec<usize> {
    let n = labs.len();
    let num_ants = options.num_ants.unwrap_or(20);
    let max_iterations = options.aco_iterations.unwrap_or(100);
    let evaporation_rate = options.evaporation_rate.unwrap_or(0.1);
    let alpha = options.pheromone_importance.unwrap_or(1.0);
    let beta = options.heuristic_importance.unwrap_or(2.0);

    // Initialize pheromone trails
    let mut pheromones = vec![1.0f64; n];

    let mut best_solution = Vec::new();
    let mut best_fitness = f64::NEG_INFINITY;

    for _ in 0..max_iterations {
        // Each ant constructs a solution
        #[cfg(feature = "parallel")]
        let (iteration_solutions, iteration_fitnesses) = {
            use rayon::prelude::*;
            // Pre-generate per-ant seeds for deterministic parallel execution
            let ant_seeds: Vec<u32> = (0..num_ants).map(|_| rng.next_u32()).collect();
            let results: Vec<(Vec<usize>, f64)> = ant_seeds
                .into_par_iter()
                .map(|seed| {
                    let mut ant_rng = Mulberry32::new(seed);
                    let solution = construct_ant_solution(
                        n, select_count, labs, dist_matrix, &pheromones, alpha, beta, &mut ant_rng,
                    );
                    let fitness = calculate_fitness(&solution, labs, dist_matrix);
                    (solution, fitness)
                })
                .collect();
            let (sols, fits): (Vec<_>, Vec<_>) = results.into_iter().unzip();
            (sols, fits)
        };
        #[cfg(not(feature = "parallel"))]
        let (iteration_solutions, iteration_fitnesses) = {
            let mut sols = Vec::with_capacity(num_ants);
            let mut fits = Vec::with_capacity(num_ants);
            for _ in 0..num_ants {
                let solution = construct_ant_solution(
                    n, select_count, labs, dist_matrix, &pheromones, alpha, beta, rng,
                );
                let fitness = calculate_fitness(&solution, labs, dist_matrix);
                fits.push(fitness);
                sols.push(solution);
            }
            (sols, fits)
        };

        // Update best
        for (i, &fitness) in iteration_fitnesses.iter().enumerate() {
            if fitness > best_fitness {
                best_fitness = fitness;
                best_solution = iteration_solutions[i].clone();
            }
        }

        // Evaporate pheromones
        for p in &mut pheromones {
            *p *= 1.0 - evaporation_rate;
        }

        // Deposit pheromones proportional to fitness (fix: JS deposited equally)
        let max_fitness = iteration_fitnesses
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        for (solution, &fitness) in iteration_solutions.iter().zip(iteration_fitnesses.iter()) {
            let deposit = if max_fitness > 0.0 {
                fitness / max_fitness
            } else {
                1.0 / solution.len() as f64
            };
            for &color_idx in solution {
                pheromones[color_idx] += deposit;
            }
        }
    }

    best_solution
}
