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

/// Exact Minimum: brute-force search for the combination that maximizes the
/// minimum pairwise distance (the true "most distinct" objective).
///
/// Improvements over JS:
/// - Branch-and-bound pruning: if any pair in partial selection has distance <= best known,
///   skip all extensions of that combination
/// - Index-based combination generation (no array allocations)
/// - Feasibility check with configurable limit
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

    // Compute initial best from first combination
    let mut best_min_distance = {
        let mut init_min = f64::INFINITY;
        for i in 0..select_count {
            for j in (i + 1)..select_count {
                let d = get_distance(i, j, labs, dist_matrix);
                if d < init_min {
                    init_min = d;
                }
            }
        }
        init_min
    };

    // Branch-and-bound recursive search
    let mut current = Vec::with_capacity(select_count);
    branch_and_bound(
        labs,
        dist_matrix,
        n,
        select_count,
        0,
        &mut current,
        f64::INFINITY,
        &mut best_selection,
        &mut best_min_distance,
    );

    Ok(best_selection)
}

#[allow(clippy::too_many_arguments)]
fn branch_and_bound(
    labs: &[Lab],
    dm: Option<&DistanceMatrix>,
    n: usize,
    k: usize,
    start: usize,
    current: &mut Vec<usize>,
    current_min: f64,
    best_selection: &mut Vec<usize>,
    best_min_distance: &mut f64,
) {
    if current.len() == k {
        if current_min > *best_min_distance {
            *best_min_distance = current_min;
            best_selection.clear();
            best_selection.extend_from_slice(current);
        }
        return;
    }

    let remaining = k - current.len();

    for i in start..=(n - remaining) {
        // Pruning: check distance from candidate i to all already-selected
        let mut min_with_new = current_min;
        let mut pruned = false;

        for &selected in current.iter() {
            let d = get_distance(i, selected, labs, dm);
            if d < min_with_new {
                min_with_new = d;
            }
            // Prune if this partial solution already can't beat the best
            if min_with_new <= *best_min_distance {
                pruned = true;
                break;
            }
        }

        if pruned {
            continue;
        }

        current.push(i);
        branch_and_bound(labs, dm, n, k, i + 1, current, min_with_new, best_selection, best_min_distance);
        current.pop();
    }
}
