//! # pick-distinct-colors
//!
//! Select maximally distinct colors from a pool using various algorithms
//! in CIELAB color space with CIE76 deltaE as the distance metric.

pub mod algorithms;
pub mod api;
pub mod color;
pub mod distance;
pub mod error;
pub mod metrics;
pub mod pool;
pub mod prng;
#[cfg(feature = "wasm")]
pub mod wasm;

/// Cross-platform millisecond timestamp (works on native and WASM).
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now_ms() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now_ms() -> f64 {
    js_sys::Date::now()
}

// Public re-exports
pub use algorithms::{Algorithm, AlgorithmOptions, SelectionResult};
pub use api::{pick_distinct_colors, Config};
pub use color::{rgb2lab, sort_colors, Lab, Rgb};
pub use distance::{delta_e, DistanceMatrix};
pub use error::Error;
pub use metrics::{calculate_metrics, find_closest_pair, ClosestPair, Distribution, Metrics};
pub use pool::generate_pool;
pub use prng::Mulberry32;
