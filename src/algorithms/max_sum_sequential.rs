use crate::color::Lab;
use crate::distance::delta_e;
use crate::prng::Mulberry32;

/// Max Sum Sequential: greedy selection maximizing sum of distances to selected set.
pub(crate) fn run(labs: &[Lab], select_count: usize, rng: &mut Mulberry32) -> Vec<usize> {
    let n = labs.len();
    let mut selected = Vec::with_capacity(select_count);
    let mut is_selected = vec![false; n];

    // Track sum of distances from each point to the selected set
    let mut sum_dist_to_selected = vec![0.0f64; n];

    // Select first point randomly
    let first = rng.next_index(n);
    selected.push(first);
    is_selected[first] = true;

    // Update sum distances after first pick
    for i in 0..n {
        if !is_selected[i] {
            sum_dist_to_selected[i] = delta_e(&labs[i], &labs[first]);
        }
    }

    // Select remaining points
    while selected.len() < select_count {
        let mut best_idx = 0;
        let mut best_sum = f64::NEG_INFINITY;

        for i in 0..n {
            if !is_selected[i] && sum_dist_to_selected[i] > best_sum {
                best_sum = sum_dist_to_selected[i];
                best_idx = i;
            }
        }

        selected.push(best_idx);
        is_selected[best_idx] = true;

        // Incrementally update: add distance to newly selected point
        for i in 0..n {
            if !is_selected[i] {
                sum_dist_to_selected[i] += delta_e(&labs[i], &labs[best_idx]);
            }
        }
    }

    selected
}
