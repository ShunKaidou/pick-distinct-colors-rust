use crate::algorithms::AlgorithmOptions;
use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};
use crate::error::{self, combinations_count, Error};

/// Get distance between two colors.
#[inline]
fn get_distance(i: usize, j: usize, labs: &[Lab], dm: Option<&DistanceMatrix>) -> f64 {
    match dm {
        Some(matrix) => matrix.get(i, j),
        None => delta_e(&labs[i], &labs[j]),
    }
}

/// Exact Maximum: brute-force search for the combination that maximizes the
/// maximum pairwise distance.
///
/// Note: This objective is somewhat degenerate — for k >= 2, the result is
/// dominated by the two most distant colors in the pool. The remaining k-2
/// colors are arbitrary. Kept for API compatibility with the JS version.
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    options: &AlgorithmOptions,
    dist_matrix: Option<&DistanceMatrix>,
) -> error::Result<Vec<usize>> {
    let n = labs.len();
    let limit = options.exact_limit.unwrap_or(1_000_000);

    // Check feasibility
    if let Some(count) = combinations_count(n, select_count) {
        if count > limit {
            return Err(Error::ExactInfeasible {
                n,
                k: select_count,
                limit,
            });
        }
    } else {
        return Err(Error::ExactInfeasible {
            n,
            k: select_count,
            limit,
        });
    }

    let mut best_selection: Vec<usize> = (0..select_count).collect();
    let mut best_max_distance = f64::NEG_INFINITY;

    // Enumerate all combinations
    let mut current = Vec::with_capacity(select_count);
    enumerate_combinations(
        labs,
        dist_matrix,
        n,
        select_count,
        0,
        &mut current,
        &mut best_selection,
        &mut best_max_distance,
    );

    Ok(best_selection)
}

#[allow(clippy::too_many_arguments)]
fn enumerate_combinations(
    labs: &[Lab],
    dm: Option<&DistanceMatrix>,
    n: usize,
    k: usize,
    start: usize,
    current: &mut Vec<usize>,
    best_selection: &mut Vec<usize>,
    best_max_distance: &mut f64,
) {
    if current.len() == k {
        // Compute max pairwise distance
        let mut max_dist = f64::NEG_INFINITY;
        for i in 0..k {
            for j in (i + 1)..k {
                let d = get_distance(current[i], current[j], labs, dm);
                if d > max_dist {
                    max_dist = d;
                }
            }
        }
        if max_dist > *best_max_distance {
            *best_max_distance = max_dist;
            best_selection.clear();
            best_selection.extend_from_slice(current);
        }
        return;
    }

    let remaining = k - current.len();
    for i in start..=(n - remaining) {
        current.push(i);
        enumerate_combinations(labs, dm, n, k, i + 1, current, best_selection, best_max_distance);
        current.pop();
    }
}
