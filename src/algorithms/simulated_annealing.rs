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

/// Simulated Annealing for color selection.
///
/// Fixes from JS:
/// - Uses Fisher-Yates shuffle instead of biased sort-based shuffle
/// - Uses Vec<bool> instead of Array.includes() for O(1) membership
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
    let max_iterations = 10_000;
    let initial_temp = options.initial_temp.unwrap_or(1000.0);
    let cooling_rate = options.cooling_rate.unwrap_or(0.995);
    let min_temp = options.min_temp.unwrap_or(0.1);

    // Generate initial solution using Fisher-Yates partial shuffle
    let mut indices: Vec<usize> = (0..n).collect();
    fisher_yates_shuffle(&mut indices, rng);
    let mut current: Vec<usize> = indices[..select_count].to_vec();

    // Build membership set
    let mut in_current = vec![false; n];
    for &idx in &current {
        in_current[idx] = true;
    }

    let mut current_fitness = calculate_fitness(&current, labs, dist_matrix);
    let mut best = current.clone();
    let mut best_fitness = current_fitness;

    let mut temperature = initial_temp;

    for _ in 0..max_iterations {
        if temperature <= min_temp {
            break;
        }

        // Generate neighbor: swap one selected with one unselected
        let swap_pos = rng.next_index(select_count);
        let old_idx = current[swap_pos];

        // Pick a random unselected color (O(n) scan, guaranteed termination)
        let target = rng.next_index(n - select_count);
        let new_idx = (0..n)
            .filter(|&i| !in_current[i])
            .nth(target)
            .unwrap();

        // Apply swap
        current[swap_pos] = new_idx;
        in_current[old_idx] = false;
        in_current[new_idx] = true;

        let neighbor_fitness = calculate_fitness(&current, labs, dist_matrix);

        let delta = neighbor_fitness - current_fitness;
        if delta > 0.0 || rng.next_f64() < (delta / temperature).exp() {
            // Accept
            current_fitness = neighbor_fitness;
            if current_fitness > best_fitness {
                best = current.clone();
                best_fitness = current_fitness;
            }
        } else {
            // Reject: revert swap
            current[swap_pos] = old_idx;
            in_current[new_idx] = false;
            in_current[old_idx] = true;
        }

        temperature *= cooling_rate;
    }

    best
}
