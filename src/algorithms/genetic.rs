use crate::algorithms::AlgorithmOptions;
use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};
use crate::prng::{fisher_yates_shuffle, Mulberry32};

/// Compute minimum pairwise distance for a selection.
fn calculate_fitness(
    selection: &[usize],
    labs: &[Lab],
    dist_matrix: Option<&DistanceMatrix>,
) -> f64 {
    let mut min_dist = f64::INFINITY;
    for i in 0..selection.len() {
        for j in (i + 1)..selection.len() {
            let d = match dist_matrix {
                Some(dm) => dm.get(selection[i], selection[j]),
                None => delta_e(&labs[selection[i]], &labs[selection[j]]),
            };
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

/// Genetic Algorithm for color selection.
///
/// Fixes from JS:
/// - Fisher-Yates shuffle for population initialization (not biased sort)
/// - Vec<bool> for O(1) membership instead of Array.includes()
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    options: &AlgorithmOptions,
    rng: &mut Mulberry32,
    dist_matrix: Option<&DistanceMatrix>,
) -> Vec<usize> {
    let n = labs.len();
    if select_count >= n {
        return (0..n).collect();
    }
    let population_size = options.population_size.unwrap_or(100);
    let generations = options.generations.unwrap_or(100);
    let mutation_rate = options.mutation_rate.unwrap_or(0.1);

    // Generate initial population using Fisher-Yates
    let mut population: Vec<Vec<usize>> = (0..population_size)
        .map(|_| {
            let mut indices: Vec<usize> = (0..n).collect();
            fisher_yates_shuffle(&mut indices, rng);
            indices[..select_count].to_vec()
        })
        .collect();

    let mut best_solution = population[0].clone();
    let mut best_fitness = calculate_fitness(&best_solution, labs, dist_matrix);

    for _ in 0..generations {
        // Calculate fitness for all
        #[cfg(feature = "parallel")]
        let fitnesses: Vec<f64> = {
            use rayon::prelude::*;
            population
                .par_iter()
                .map(|sol| calculate_fitness(sol, labs, dist_matrix))
                .collect()
        };
        #[cfg(not(feature = "parallel"))]
        let fitnesses: Vec<f64> = population
            .iter()
            .map(|sol| calculate_fitness(sol, labs, dist_matrix))
            .collect();

        // Update best
        for (i, &f) in fitnesses.iter().enumerate() {
            if f > best_fitness {
                best_fitness = f;
                best_solution = population[i].clone();
            }
        }

        // Create new population
        let mut new_population = Vec::with_capacity(population_size);

        while new_population.len() < population_size {
            // Tournament selection (size 3)
            let parent1 = tournament_select(&population, &fitnesses, 3, rng);
            let parent2 = tournament_select(&population, &fitnesses, 3, rng);

            // Crossover
            let crossover_point = rng.next_index(select_count);
            let mut child_set = vec![false; n];
            let mut child = Vec::with_capacity(select_count);

            // Take first part from parent1
            for &idx in &parent1[..crossover_point] {
                if !child_set[idx] {
                    child.push(idx);
                    child_set[idx] = true;
                }
            }

            // Fill from parent2
            for &idx in &parent2[crossover_point..] {
                if child.len() >= select_count {
                    break;
                }
                if !child_set[idx] {
                    child.push(idx);
                    child_set[idx] = true;
                }
            }

            // Fill remaining randomly if needed
            if child.len() < select_count {
                let mut unselected: Vec<usize> = (0..n).filter(|&i| !child_set[i]).collect();
                while child.len() < select_count {
                    let idx = rng.next_index(unselected.len());
                    let chosen = unselected.swap_remove(idx);
                    child.push(chosen);
                    child_set[chosen] = true;
                }
            }

            // Mutation
            if rng.next_f64() < mutation_rate {
                let mut_pos = rng.next_index(select_count);
                let old = child[mut_pos];
                child_set[old] = false;

                let target = rng.next_index(n - select_count + 1);
                let new_idx = (0..n)
                    .filter(|&i| !child_set[i])
                    .nth(target)
                    .unwrap();
                child[mut_pos] = new_idx;
                child_set[new_idx] = true;
            }

            new_population.push(child);
        }

        population = new_population;
    }

    best_solution
}

fn tournament_select(
    population: &[Vec<usize>],
    fitnesses: &[f64],
    tournament_size: usize,
    rng: &mut Mulberry32,
) -> Vec<usize> {
    let mut best_idx = rng.next_index(population.len());
    let mut best_fit = fitnesses[best_idx];

    for _ in 1..tournament_size {
        let idx = rng.next_index(population.len());
        if fitnesses[idx] > best_fit {
            best_fit = fitnesses[idx];
            best_idx = idx;
        }
    }

    population[best_idx].clone()
}
