use crate::color::{Lab, Rgb};
use crate::distance::delta_e;

/// Pairwise distance metrics for a set of colors.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metrics {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub sum: f64,
}

/// The closest pair of colors and their distance.
#[derive(Clone, Debug)]
pub struct ClosestPair {
    pub colors: [Rgb; 2],
    pub distance: f64,
}

/// Distribution coverage in Lab color space.
#[derive(Clone, Debug)]
pub struct Distribution {
    /// Coverage percentage for L* (lightness), 0-100.
    pub l_coverage: f64,
    /// Coverage percentage for a* (green-red), 0-100.
    pub a_coverage: f64,
    /// Coverage percentage for b* (blue-yellow), 0-100.
    pub b_coverage: f64,
}

/// Calculate pairwise distance metrics for a set of colors.
pub fn calculate_metrics(colors: &[Rgb]) -> Metrics {
    let labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();
    calculate_metrics_from_labs(&labs)
}

/// Calculate pairwise distance metrics from pre-computed Lab values.
pub fn calculate_metrics_from_labs(labs: &[Lab]) -> Metrics {
    if labs.len() < 2 {
        return Metrics { min: 0.0, max: 0.0, avg: 0.0, sum: 0.0 };
    }
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sum = 0.0;
    let mut count = 0u64;

    for i in 0..labs.len() {
        for j in (i + 1)..labs.len() {
            let dist = delta_e(&labs[i], &labs[j]);
            if dist < min {
                min = dist;
            }
            if dist > max {
                max = dist;
            }
            sum += dist;
            count += 1;
        }
    }

    Metrics {
        min,
        max,
        avg: if count > 0 { sum / count as f64 } else { 0.0 },
        sum,
    }
}

/// Find the closest pair of colors (smallest deltaE).
///
/// Requires at least 2 colors. Returns the first color paired with itself
/// (distance 0) if only 1 color is provided.
pub fn find_closest_pair(colors: &[Rgb]) -> ClosestPair {
    assert!(!colors.is_empty(), "find_closest_pair requires at least 1 color");

    if colors.len() == 1 {
        return ClosestPair {
            colors: [colors[0], colors[0]],
            distance: 0.0,
        };
    }

    let labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();
    let mut min_dist = f64::INFINITY;
    let mut pair = [0usize, 1];

    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            let dist = delta_e(&labs[i], &labs[j]);
            if dist < min_dist {
                min_dist = dist;
                pair = [i, j];
            }
        }
    }

    ClosestPair {
        colors: [colors[pair[0]], colors[pair[1]]],
        distance: min_dist,
    }
}

/// Analyze color distribution coverage in Lab space.
pub fn analyze_distribution(colors: &[Rgb]) -> Distribution {
    if colors.is_empty() {
        return Distribution {
            l_coverage: 0.0,
            a_coverage: 0.0,
            b_coverage: 0.0,
        };
    }

    let labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();

    let mut l_min = labs[0].l;
    let mut l_max = labs[0].l;
    let mut a_min = labs[0].a;
    let mut a_max = labs[0].a;
    let mut b_min = labs[0].b;
    let mut b_max = labs[0].b;

    for lab in &labs[1..] {
        l_min = l_min.min(lab.l);
        l_max = l_max.max(lab.l);
        a_min = a_min.min(lab.a);
        a_max = a_max.max(lab.a);
        b_min = b_min.min(lab.b);
        b_max = b_max.max(lab.b);
    }

    Distribution {
        l_coverage: (l_max - l_min) / 100.0 * 100.0, // L range [0, 100]
        a_coverage: (a_max - a_min) / 255.0 * 100.0,  // a range [-128, 127]
        b_coverage: (b_max - b_min) / 255.0 * 100.0,  // b range [-128, 127]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics() {
        let colors = vec![
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
        ];
        let metrics = calculate_metrics(&colors);
        assert!(metrics.min > 0.0);
        assert!(metrics.max >= metrics.min);
        assert!(metrics.avg >= metrics.min);
        assert!(metrics.avg <= metrics.max);
        assert!(metrics.sum > 0.0);
    }

    #[test]
    fn test_find_closest_pair() {
        let colors = vec![
            Rgb::new(255, 0, 0),
            Rgb::new(254, 0, 0), // Very close to red
            Rgb::new(0, 0, 255),
        ];
        let pair = find_closest_pair(&colors);
        assert!(pair.distance < 5.0); // The two reds should be very close
        assert!(
            (pair.colors[0] == Rgb::new(255, 0, 0) && pair.colors[1] == Rgb::new(254, 0, 0))
                || (pair.colors[0] == Rgb::new(254, 0, 0)
                    && pair.colors[1] == Rgb::new(255, 0, 0))
        );
    }

    #[test]
    fn test_analyze_distribution() {
        let colors = vec![
            Rgb::new(0, 0, 0),
            Rgb::new(255, 255, 255),
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
        ];
        let dist = analyze_distribution(&colors);
        assert!(dist.l_coverage > 80.0);
        assert!(dist.a_coverage > 40.0);
        assert!(dist.b_coverage > 40.0);
    }

    #[test]
    fn test_find_closest_pair_single_color() {
        let pair = find_closest_pair(&[Rgb::new(128, 64, 32)]);
        assert_eq!(pair.distance, 0.0);
        assert_eq!(pair.colors[0], Rgb::new(128, 64, 32));
    }

    #[test]
    fn test_find_closest_pair_identical_colors() {
        let colors = vec![Rgb::new(100, 100, 100); 5];
        let pair = find_closest_pair(&colors);
        assert!(pair.distance < 1e-10);
    }

    #[test]
    fn test_find_closest_pair_two_colors() {
        let colors = vec![Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)];
        let pair = find_closest_pair(&colors);
        assert!(pair.distance > 90.0); // black/white are very distant
    }

    #[test]
    fn test_calculate_metrics_two_colors() {
        let colors = vec![Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)];
        let m = calculate_metrics(&colors);
        // Only 1 pair, so min == max == avg
        assert!((m.min - m.max).abs() < 1e-10);
        assert!((m.min - m.avg).abs() < 1e-10);
        assert!((m.min - m.sum).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_metrics_three_known() {
        let colors = vec![
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
        ];
        let m = calculate_metrics(&colors);
        // 3 pairs, all distances > 0
        assert!(m.min > 50.0);
        assert!(m.sum > 0.0);
        assert_eq!(m.avg, m.sum / 3.0);
    }

    #[test]
    fn test_analyze_distribution_single_color() {
        let dist = analyze_distribution(&[Rgb::new(128, 128, 128)]);
        assert!((dist.l_coverage).abs() < 1e-10);
        assert!((dist.a_coverage).abs() < 1e-10);
        assert!((dist.b_coverage).abs() < 1e-10);
    }

    #[test]
    fn test_analyze_distribution_empty() {
        let dist = analyze_distribution(&[]);
        assert!((dist.l_coverage).abs() < 1e-10);
        assert!((dist.a_coverage).abs() < 1e-10);
        assert!((dist.b_coverage).abs() < 1e-10);
    }
}
