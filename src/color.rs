/// An sRGB color with 8-bit channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A color in CIELAB color space (D65 illuminant).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to CIELAB color space.
    /// Uses sRGB gamma, D65 illuminant (same as the JS implementation).
    pub fn to_lab(self) -> Lab {
        rgb2lab(self)
    }

    /// Convert to hex string (e.g. "#ff8000").
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl From<[u8; 3]> for Rgb {
    fn from(arr: [u8; 3]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
        }
    }
}

impl From<Rgb> for [u8; 3] {
    fn from(rgb: Rgb) -> Self {
        [rgb.r, rgb.g, rgb.b]
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

/// Convert sRGB to CIELAB (D65 illuminant).
///
/// Pipeline: sRGB → linear RGB → XYZ (D65) → Lab
/// Matches the JS `rgb2lab` function exactly.
pub fn rgb2lab(rgb: Rgb) -> Lab {
    // sRGB gamma decode
    let r = linearize(rgb.r as f64 / 255.0);
    let g = linearize(rgb.g as f64 / 255.0);
    let b = linearize(rgb.b as f64 / 255.0);

    // Linear RGB to XYZ (sRGB matrix, D65)
    let x = (r * 0.4124 + g * 0.3576 + b * 0.1805) * 100.0;
    let y = (r * 0.2126 + g * 0.7152 + b * 0.0722) * 100.0;
    let z = (r * 0.0193 + g * 0.1192 + b * 0.9505) * 100.0;

    // D65 reference white
    let x = xyz_to_lab_f(x / 95.047);
    let y = xyz_to_lab_f(y / 100.0);
    let z = xyz_to_lab_f(z / 108.883);

    Lab {
        l: (116.0 * y) - 16.0,
        a: 500.0 * (x - y),
        b: 200.0 * (y - z),
    }
}

/// sRGB gamma linearization.
#[inline]
fn linearize(c: f64) -> f64 {
    if c > 0.04045 {
        ((c + 0.055) / 1.055).powf(2.4)
    } else {
        c / 12.92
    }
}

/// XYZ-to-Lab nonlinear transform.
#[inline]
fn xyz_to_lab_f(t: f64) -> f64 {
    if t > 0.008856 {
        t.powf(1.0 / 3.0)
    } else {
        (7.787 * t) + 16.0 / 116.0
    }
}

/// Sort colors by Lab values: L descending, then a descending, then b descending.
pub fn sort_colors(colors: &mut [Rgb]) {
    colors.sort_by(|a, b| {
        let la = a.to_lab();
        let lb = b.to_lab();
        lb.l.partial_cmp(&la.l)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(lb.a.partial_cmp(&la.a).unwrap_or(std::cmp::Ordering::Equal))
            .then(lb.b.partial_cmp(&la.b).unwrap_or(std::cmp::Ordering::Equal))
    });
}

/// Sort colors by pre-computed Lab values (avoids re-conversion).
///
/// # Panics
/// Panics if `colors.len() != labs.len()`.
pub fn sort_colors_by_lab(colors: &mut [Rgb], labs: &[Lab]) {
    debug_assert_eq!(colors.len(), labs.len(), "colors and labs must have same length");
    let mut indices: Vec<usize> = (0..colors.len()).collect();
    indices.sort_by(|&i, &j| {
        labs[j]
            .l
            .partial_cmp(&labs[i].l)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                labs[j]
                    .a
                    .partial_cmp(&labs[i].a)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
            .then(
                labs[j]
                    .b
                    .partial_cmp(&labs[i].b)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    let sorted: Vec<Rgb> = indices.iter().map(|&i| colors[i]).collect();
    colors.copy_from_slice(&sorted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb2lab_black() {
        let lab = rgb2lab(Rgb::new(0, 0, 0));
        assert!((lab.l - 0.0).abs() < 0.01);
        assert!((lab.a - 0.0).abs() < 0.01);
        assert!((lab.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb2lab_white() {
        let lab = rgb2lab(Rgb::new(255, 255, 255));
        assert!((lab.l - 100.0).abs() < 0.01);
        assert!(lab.a.abs() < 0.5);
        assert!(lab.b.abs() < 0.5);
    }

    #[test]
    fn test_rgb2lab_red() {
        // Known values for pure red
        let lab = rgb2lab(Rgb::new(255, 0, 0));
        assert!((lab.l - 53.23).abs() < 0.5);
        assert!((lab.a - 80.11).abs() < 0.5);
        assert!((lab.b - 67.22).abs() < 0.5);
    }

    #[test]
    fn test_rgb2lab_orange() {
        let lab = rgb2lab(Rgb::new(255, 128, 0));
        assert!((lab.l - 67.05).abs() < 0.5);
        assert!((lab.a - 42.83).abs() < 0.5);
        assert!((lab.b - 74.03).abs() < 0.5);
    }

    #[test]
    fn test_hex_conversion() {
        assert_eq!(Rgb::new(255, 128, 0).to_hex(), "#ff8000");
        assert_eq!(Rgb::new(0, 0, 0).to_hex(), "#000000");
        assert_eq!(Rgb::new(255, 255, 255).to_hex(), "#ffffff");
    }

    #[test]
    fn test_sort_colors() {
        let mut colors = vec![
            Rgb::new(0, 0, 0),       // L ≈ 0
            Rgb::new(255, 255, 255),  // L ≈ 100
            Rgb::new(128, 128, 128),  // L ≈ 53
        ];
        sort_colors(&mut colors);
        // Should be L descending: white, gray, black
        assert_eq!(colors[0], Rgb::new(255, 255, 255));
        assert_eq!(colors[2], Rgb::new(0, 0, 0));
    }

    #[test]
    fn test_from_array() {
        let rgb = Rgb::from([10, 20, 30]);
        assert_eq!(rgb, Rgb::new(10, 20, 30));
        let arr: [u8; 3] = rgb.into();
        assert_eq!(arr, [10, 20, 30]);
    }

    #[test]
    fn test_from_tuple() {
        let rgb = Rgb::from((10u8, 20u8, 30u8));
        assert_eq!(rgb, Rgb::new(10, 20, 30));
    }

    #[test]
    fn test_rgb2lab_green() {
        let lab = rgb2lab(Rgb::new(0, 255, 0));
        assert!((lab.l - 87.74).abs() < 0.5);
        assert!((lab.a - (-86.18)).abs() < 0.5);
        assert!((lab.b - 83.18).abs() < 0.5);
    }

    #[test]
    fn test_rgb2lab_blue() {
        let lab = rgb2lab(Rgb::new(0, 0, 255));
        assert!((lab.l - 32.30).abs() < 0.5);
        assert!((lab.a - 79.19).abs() < 1.0);
        assert!((lab.b - (-107.86)).abs() < 1.0);
    }

    #[test]
    fn test_rgb2lab_l_range() {
        // L should always be in [0, 100] for any valid RGB
        for r in (0..=255).step_by(51) {
            for g in (0..=255).step_by(51) {
                for b in (0..=255).step_by(51) {
                    let lab = rgb2lab(Rgb::new(r as u8, g as u8, b as u8));
                    assert!(lab.l >= -0.01 && lab.l <= 100.01,
                        "L out of range for ({r},{g},{b}): {}", lab.l);
                }
            }
        }
    }

    #[test]
    fn test_sort_colors_by_lab_matches_sort_colors() {
        let mut colors1 = vec![
            Rgb::new(255, 0, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
            Rgb::new(128, 128, 0),
            Rgb::new(0, 128, 128),
        ];
        let mut colors2 = colors1.clone();
        let labs: Vec<Lab> = colors2.iter().map(|c| c.to_lab()).collect();

        sort_colors(&mut colors1);
        sort_colors_by_lab(&mut colors2, &labs);
        assert_eq!(colors1, colors2, "sort_colors and sort_colors_by_lab must match");
    }

    #[test]
    fn test_sort_colors_by_lab_single_element() {
        let mut colors = vec![Rgb::new(42, 42, 42)];
        let labs = vec![colors[0].to_lab()];
        sort_colors_by_lab(&mut colors, &labs);
        assert_eq!(colors[0], Rgb::new(42, 42, 42));
    }

    #[test]
    fn test_sort_colors_by_lab_already_sorted() {
        let mut colors = vec![
            Rgb::new(255, 255, 255), // L ≈ 100
            Rgb::new(128, 128, 128), // L ≈ 53
            Rgb::new(0, 0, 0),       // L ≈ 0
        ];
        let labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();
        let original = colors.clone();
        sort_colors_by_lab(&mut colors, &labs);
        assert_eq!(colors, original, "Already-sorted should not change");
    }

    #[test]
    fn test_sort_colors_by_lab_tie_breaks() {
        // Two colors with very similar L but different a values
        let c1 = Rgb::new(128, 0, 0);   // reddish
        let c2 = Rgb::new(0, 128, 0);   // greenish
        let lab1 = c1.to_lab();
        let lab2 = c2.to_lab();

        // Find which has higher a
        let mut colors = vec![c1, c2];
        let labs = vec![lab1, lab2];
        sort_colors_by_lab(&mut colors, &labs);

        // Verify sorting is consistent
        let sorted_labs: Vec<Lab> = colors.iter().map(|c| c.to_lab()).collect();
        for i in 0..sorted_labs.len() - 1 {
            let curr = &sorted_labs[i];
            let next = &sorted_labs[i + 1];
            assert!(
                curr.l > next.l
                    || (curr.l == next.l && curr.a >= next.a)
                    || (curr.l == next.l && curr.a == next.a && curr.b >= next.b),
                "Sort order violated at index {i}"
            );
        }
    }
}
