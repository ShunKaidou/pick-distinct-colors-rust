use crate::color::Lab;

/// CIE76 deltaE: Euclidean distance in CIELAB color space.
#[inline]
pub fn delta_e(a: &Lab, b: &Lab) -> f64 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}

/// Squared deltaE (avoids sqrt, use when only comparing distances).
#[inline]
pub fn delta_e_squared(a: &Lab, b: &Lab) -> f64 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    dl * dl + da * da + db * db
}

/// Pre-computed symmetric distance matrix stored as upper-triangular.
///
/// For N colors, stores N*(N-1)/2 entries instead of N*N.
/// Index mapping: for i < j, index = i * (2*n - i - 1) / 2 + (j - i - 1)
pub struct DistanceMatrix {
    n: usize,
    distances: Vec<f64>,
}

impl DistanceMatrix {
    /// Build a distance matrix from pre-computed Lab colors.
    pub fn from_labs(labs: &[Lab]) -> Self {
        let n = labs.len();
        if n < 2 {
            return Self { n, distances: Vec::new() };
        }
        let size = n * (n - 1) / 2;
        let mut distances = vec![0.0; size];

        for i in 0..n {
            for j in (i + 1)..n {
                let idx = Self::index_of(n, i, j);
                distances[idx] = delta_e(&labs[i], &labs[j]);
            }
        }

        Self { n, distances }
    }

    /// Build a distance matrix using Rayon for parallel computation.
    #[cfg(feature = "parallel")]
    pub fn from_labs_parallel(labs: &[Lab]) -> Self {
        use rayon::prelude::*;

        let n = labs.len();
        if n < 2 {
            return Self { n, distances: Vec::new() };
        }
        let size = n * (n - 1) / 2;
        let mut distances = vec![0.0; size];

        // Parallelize over rows
        let row_results: Vec<Vec<(usize, f64)>> = (0..n)
            .into_par_iter()
            .map(|i| {
                let mut row = Vec::with_capacity(n - i - 1);
                for j in (i + 1)..n {
                    let idx = Self::index_of(n, i, j);
                    row.push((idx, delta_e(&labs[i], &labs[j])));
                }
                row
            })
            .collect();

        for row in row_results {
            for (idx, dist) in row {
                distances[idx] = dist;
            }
        }

        Self { n, distances }
    }

    /// Get the distance between colors i and j.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> f64 {
        if i == j {
            return 0.0;
        }
        debug_assert!(
            i < self.n && j < self.n,
            "index out of range: i={i}, j={j}, n={}",
            self.n
        );
        let (lo, hi) = if i < j { (i, j) } else { (j, i) };
        self.distances[Self::index_of(self.n, lo, hi)]
    }

    /// Number of colors in the matrix.
    #[inline]
    pub fn n(&self) -> usize {
        self.n
    }

    /// Compute the flat index for the upper-triangular entry (i, j) where i < j.
    #[inline]
    fn index_of(n: usize, i: usize, j: usize) -> usize {
        debug_assert!(i < j);
        i * (2 * n - i - 1) / 2 + (j - i - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;

    #[test]
    fn test_delta_e_same_color() {
        let lab = Rgb::new(128, 64, 32).to_lab();
        assert!((delta_e(&lab, &lab)).abs() < 1e-10);
    }

    #[test]
    fn test_delta_e_known_value() {
        let red = Rgb::new(255, 0, 0).to_lab();
        let green = Rgb::new(0, 255, 0).to_lab();
        let d = delta_e(&red, &green);
        assert!((d - 170.58).abs() < 1.0);
    }

    #[test]
    fn test_delta_e_symmetric() {
        let a = Rgb::new(100, 50, 200).to_lab();
        let b = Rgb::new(50, 200, 100).to_lab();
        assert!((delta_e(&a, &b) - delta_e(&b, &a)).abs() < 1e-10);
    }

    #[test]
    fn test_distance_matrix() {
        let colors = vec![
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
        ];
        let labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();
        let matrix = DistanceMatrix::from_labs(&labs);

        assert_eq!(matrix.n(), 3);
        assert!((matrix.get(0, 0)).abs() < 1e-10);
        assert!((matrix.get(0, 1) - delta_e(&labs[0], &labs[1])).abs() < 1e-10);
        assert!((matrix.get(1, 0) - matrix.get(0, 1)).abs() < 1e-10); // symmetric
        assert!((matrix.get(0, 2) - delta_e(&labs[0], &labs[2])).abs() < 1e-10);
    }

    #[test]
    fn test_delta_e_squared() {
        let a = Rgb::new(100, 50, 200).to_lab();
        let b = Rgb::new(50, 200, 100).to_lab();
        let d = delta_e(&a, &b);
        let d2 = delta_e_squared(&a, &b);
        assert!((d2 - d * d).abs() < 1e-10);
    }
}
