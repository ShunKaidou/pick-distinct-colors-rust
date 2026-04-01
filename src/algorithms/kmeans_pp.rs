use crate::color::Lab;
use crate::distance::delta_e;
use crate::prng::Mulberry32;

/// K-means++ initialization for color selection.
///
/// Fixes from JS:
/// - Roulette wheel only iterates non-selected candidates (no skip logic needed)
/// - Index properly clamped to prevent OOB
pub(crate) fn run(labs: &[Lab], select_count: usize, rng: &mut Mulberry32) -> Vec<usize> {
    let n = labs.len();
    let mut selected = Vec::with_capacity(select_count);
    let mut is_selected = vec![false; n];

    // Track minimum distance to nearest selected center
    let mut min_dist_sq = vec![f64::INFINITY; n];

    // Select first center randomly
    let first = rng.next_index(n);
    selected.push(first);
    is_selected[first] = true;

    // Update squared distances from first center
    for i in 0..n {
        if !is_selected[i] {
            let d = delta_e(&labs[i], &labs[first]);
            min_dist_sq[i] = d * d;
        }
    }

    // Select remaining centers
    while selected.len() < select_count {
        // Build candidate list with cumulative weights (only non-selected)
        let mut candidates = Vec::with_capacity(n - selected.len());
        let mut cumulative = 0.0;

        for i in 0..n {
            if !is_selected[i] {
                cumulative += min_dist_sq[i];
                candidates.push((i, cumulative));
            }
        }

        // Roulette wheel selection on the filtered candidates
        let target = rng.next_f64() * cumulative;
        let chosen_idx = match candidates.binary_search_by(|&(_, cum)| {
            cum.partial_cmp(&target).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            Ok(pos) => candidates[pos].0,
            Err(pos) => {
                // Clamp to valid range
                let clamped = pos.min(candidates.len() - 1);
                candidates[clamped].0
            }
        };

        selected.push(chosen_idx);
        is_selected[chosen_idx] = true;

        // Update min squared distances with new center
        for i in 0..n {
            if !is_selected[i] {
                let d = delta_e(&labs[i], &labs[chosen_idx]);
                let d_sq = d * d;
                if d_sq < min_dist_sq[i] {
                    min_dist_sq[i] = d_sq;
                }
            }
        }
    }

    selected
}
