use crate::algorithms::AlgorithmOptions;
use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};
use crate::prng::{fisher_yates_shuffle, Mulberry32};
use std::collections::HashMap;

/// Get distance between two colors.
#[inline]
fn get_distance(i: usize, j: usize, labs: &[Lab], dm: Option<&DistanceMatrix>) -> f64 {
    match dm {
        Some(matrix) => matrix.get(i, j),
        None => delta_e(&labs[i], &labs[j]),
    }
}

/// Compute minimum pairwise distance with one position swapped (avoids cloning).
#[inline]
fn calculate_fitness_with_swap(
    selection: &[usize],
    swap_pos: usize,
    new_val: usize,
    labs: &[Lab],
    dm: Option<&DistanceMatrix>,
) -> f64 {
    let mut min_dist = f64::INFINITY;
    for i in 0..selection.len() {
        let ci = if i == swap_pos { new_val } else { selection[i] };
        for j in (i + 1)..selection.len() {
            let cj = if j == swap_pos { new_val } else { selection[j] };
            let d = get_distance(ci, cj, labs, dm);
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

/// Tabu Search for color selection.
///
/// Fixes from JS:
/// - Randomized initial solution (JS used [0, 1, ..., k-1])
/// - Correct tabu recording: makes the REVERSE of the accepted move tabu
///   (JS recorded moves between current and global-best, which was meaningless)
/// - Uses seed (JS ignored it entirely)
/// - Uses precomputed distance matrix
/// - Vec<bool> for O(1) membership
/// - Swap-based fitness avoids cloning per neighbor
/// - Optional parallel neighborhood evaluation via Rayon
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    options: &AlgorithmOptions,
    rng: &mut Mulberry32,
    dist_matrix: Option<&DistanceMatrix>,
) -> Vec<usize> {
    let n = labs.len();
    let max_iterations = options.tabu_iterations.unwrap_or(1000);
    let tabu_tenure = options.tabu_tenure.unwrap_or(5);

    // Randomized initial solution (fix: JS used [0..k])
    let mut indices: Vec<usize> = (0..n).collect();
    fisher_yates_shuffle(&mut indices, rng);
    let mut current: Vec<usize> = indices[..select_count].to_vec();

    let mut in_current = vec![false; n];
    for &idx in &current {
        in_current[idx] = true;
    }

    let mut best = current.clone();
    let mut best_fitness = {
        let mut min_dist = f64::INFINITY;
        for i in 0..current.len() {
            for j in (i + 1)..current.len() {
                let d = get_distance(current[i], current[j], labs, dist_matrix);
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
        min_dist
    };

    // Tabu list: maps (removed_color, added_color) -> expiration iteration
    let mut tabu_list: HashMap<(usize, usize), usize> = HashMap::new();

    for iteration in 0..max_iterations {
        // Find best neighbor: (fitness, pos, old_color, new_color)
        #[cfg(feature = "parallel")]
        let best_move: Option<(f64, usize, usize, usize)> = {
            use rayon::prelude::*;
            let current_ref = &current;
            let in_current_ref = &in_current;
            let tabu_list_ref = &tabu_list;
            (0..select_count)
                .into_par_iter()
                .flat_map_iter(move |pos| {
                    let old_color = current_ref[pos];
                    (0..n)
                        .filter(move |&nc| !in_current_ref[nc])
                        .filter_map(move |new_color| {
                            let move_key = (new_color, old_color);
                            let is_tabu =
                                tabu_list_ref.get(&move_key).is_some_and(|&exp| exp > iteration);
                            let fitness = calculate_fitness_with_swap(
                                current_ref, pos, new_color, labs, dist_matrix,
                            );
                            if !is_tabu || fitness > best_fitness {
                                Some((fitness, pos, old_color, new_color))
                            } else {
                                None
                            }
                        })
                })
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        };
        #[cfg(not(feature = "parallel"))]
        let best_move: Option<(f64, usize, usize, usize)> = {
            let mut result: Option<(f64, usize, usize, usize)> = None;
            for pos in 0..select_count {
                let old_color = current[pos];
                #[allow(clippy::needless_range_loop)]
                for new_color in 0..n {
                    if in_current[new_color] {
                        continue;
                    }
                    let move_key = (new_color, old_color);
                    let is_tabu =
                        tabu_list.get(&move_key).is_some_and(|&exp| exp > iteration);
                    let fitness =
                        calculate_fitness_with_swap(&current, pos, new_color, labs, dist_matrix);
                    let dominated = match &result {
                        Some((bf, _, _, _)) => fitness > *bf,
                        None => true,
                    };
                    if (dominated && !is_tabu) || fitness > best_fitness {
                        result = Some((fitness, pos, old_color, new_color));
                    }
                }
            }
            result
        };

        match best_move {
            Some((fitness, pos, old_color, new_color)) => {
                // Apply move
                current[pos] = new_color;
                in_current[old_color] = false;
                in_current[new_color] = true;

                // Record the REVERSE move as tabu (fix: JS recorded wrong moves)
                tabu_list.insert((old_color, new_color), iteration + tabu_tenure);

                // Update global best
                if fitness > best_fitness {
                    best = current.clone();
                    best_fitness = fitness;
                }
            }
            None => break, // No valid neighbor found
        }

        // Clean expired entries periodically
        if iteration % 50 == 0 {
            tabu_list.retain(|_, exp| *exp > iteration);
        }
    }

    best
}
