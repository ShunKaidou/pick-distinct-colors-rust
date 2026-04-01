use crate::color::Lab;
use crate::distance::delta_e;
use crate::prng::Mulberry32;

/// Greedy selection: pick the color that maximizes minimum distance to selected set.
///
/// Fixes from JS: uses proper Fisher-Yates for first pick (via rng.next_index),
/// avoids O(n) `includes()` by tracking selected state in a bool vec.
pub(crate) fn run(labs: &[Lab], select_count: usize, rng: &mut Mulberry32) -> Vec<usize> {
    let n = labs.len();
    let mut selected = Vec::with_capacity(select_count);
    let mut is_selected = vec![false; n];

    // Track minimum distance from each point to the selected set
    let mut min_dist_to_selected = vec![f64::INFINITY; n];

    // Select first point randomly
    let first = rng.next_index(n);
    selected.push(first);
    is_selected[first] = true;

    // Update min distances after picking the first point
    for i in 0..n {
        if !is_selected[i] {
            min_dist_to_selected[i] = delta_e(&labs[i], &labs[first]);
        }
    }

    // Select remaining points
    while selected.len() < select_count {
        let mut best_idx = 0;
        let mut best_min_dist = f64::NEG_INFINITY;

        // Find the unselected point with the largest min-distance to selected set
        for i in 0..n {
            if !is_selected[i] && min_dist_to_selected[i] > best_min_dist {
                best_min_dist = min_dist_to_selected[i];
                best_idx = i;
            }
        }

        selected.push(best_idx);
        is_selected[best_idx] = true;

        // Update min distances: only need to check distance to newly added point
        for i in 0..n {
            if !is_selected[i] {
                let d = delta_e(&labs[i], &labs[best_idx]);
                if d < min_dist_to_selected[i] {
                    min_dist_to_selected[i] = d;
                }
            }
        }
    }

    selected
}
