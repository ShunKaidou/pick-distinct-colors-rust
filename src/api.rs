use crate::algorithms::{run_algorithm, Algorithm, AlgorithmOptions, SelectionResult};
use crate::color::{Lab, Rgb};
use crate::distance::DistanceMatrix;
use crate::error::{self, Error};
use crate::pool::generate_pool;

/// Configuration for color selection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// Number of colors to select (required).
    pub count: usize,
    /// Algorithm to use. Default: Greedy.
    pub algorithm: Algorithm,
    /// Pool of RGB colors to select from. If None, a random pool is generated.
    pub colors: Option<Vec<Rgb>>,
    /// Size of random pool if `colors` is None. Default: max(count * 16, 128).
    pub pool_size: Option<usize>,
    /// Seed for the Mulberry32 PRNG. Default: 42.
    pub seed: u32,
    /// Algorithm-specific options.
    pub options: AlgorithmOptions,
}

impl Config {
    /// Create a new Config with defaults (greedy algorithm, seed 42).
    pub fn new(count: usize) -> Self {
        Self {
            count,
            algorithm: Algorithm::Greedy,
            colors: None,
            pool_size: None,
            seed: 42,
            options: AlgorithmOptions::default(),
        }
    }

    pub fn algorithm(mut self, algo: Algorithm) -> Self {
        self.algorithm = algo;
        self
    }

    pub fn colors(mut self, colors: Vec<Rgb>) -> Self {
        self.colors = Some(colors);
        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = Some(size);
        self
    }

    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    pub fn options(mut self, opts: AlgorithmOptions) -> Self {
        self.options = opts;
        self
    }
}

/// Pick maximally distinct colors from a pool.
///
/// If no color pool is provided, generates a random pool using the configured seed.
/// Pre-converts all colors to Lab space once, and builds a distance matrix when
/// beneficial for the chosen algorithm.
pub fn pick_distinct_colors(config: Config) -> error::Result<SelectionResult> {
    if config.count == 0 {
        return Err(Error::ZeroCount);
    }

    // Get or generate pool
    let pool = match config.colors {
        Some(colors) => {
            if colors.is_empty() {
                return Err(Error::EmptyPool);
            }
            colors
        }
        None => {
            let size = config
                .pool_size
                .unwrap_or_else(|| config.count.saturating_mul(16).max(128));
            generate_pool(size, config.seed)
        }
    };

    if config.count > pool.len() {
        return Err(Error::CountExceedsPool {
            count: config.count,
            pool_size: pool.len(),
        });
    }

    // Short-circuit: if selecting all colors, just sort and return them
    if config.count == pool.len() {
        let start = crate::now_ms();
        let labs: Vec<Lab> = pool.iter().map(|c| c.to_lab()).collect();
        let mut colors = pool;
        crate::color::sort_colors_by_lab(&mut colors, &labs);
        return Ok(SelectionResult {
            colors,
            time_ms: crate::now_ms() - start,
        });
    }

    // Pre-compute Lab colors once for all algorithms
    let labs: Vec<Lab> = pool.iter().map(|c| c.to_lab()).collect();

    // Build distance matrix for algorithms that benefit from it
    let needs_matrix = matches!(
        config.algorithm,
        Algorithm::MaxSumGlobal
            | Algorithm::SimulatedAnnealing
            | Algorithm::GeneticAlgorithm
            | Algorithm::ParticleSwarm
            | Algorithm::AntColony
            | Algorithm::TabuSearch
            | Algorithm::ExactMinimum
            | Algorithm::ExactMaximum
    );

    let dist_matrix = if needs_matrix && pool.len() <= 4096 {
        #[cfg(feature = "parallel")]
        {
            Some(DistanceMatrix::from_labs_parallel(&labs))
        }
        #[cfg(not(feature = "parallel"))]
        {
            Some(DistanceMatrix::from_labs(&labs))
        }
    } else {
        None
    };

    run_algorithm(
        config.algorithm,
        &pool,
        &labs,
        config.count,
        dist_matrix.as_ref(),
        &config.options,
        config.seed,
    )
}
