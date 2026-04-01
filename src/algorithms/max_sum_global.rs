use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};

/// Max Sum Global: rank each color by total distance to all others, pick top-k.
///
/// This is O(n^2), not O(n choose k) as the JS docs claimed.
/// Uses pre-computed distance matrix when available.
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    dist_matrix: Option<&DistanceMatrix>,
) -> Vec<usize> {
    let n = labs.len();

    // Compute total distance for each color
    let mut total_distances: Vec<(usize, f64)> = (0..n)
        .map(|i| {
            let sum: f64 = match dist_matrix {
                Some(dm) => (0..n).filter(|&j| j != i).map(|j| dm.get(i, j)).sum(),
                None => (0..n)
                    .filter(|&j| j != i)
                    .map(|j| delta_e(&labs[i], &labs[j]))
                    .sum(),
            };
            (i, sum)
        })
        .collect();

    // Sort by total distance descending
    total_distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top-k
    total_distances
        .iter()
        .take(select_count)
        .map(|&(idx, _)| idx)
        .collect()
}
