pub mod random;
pub mod greedy;
pub mod max_sum_global;
pub mod max_sum_sequential;
pub mod kmeans_pp;
pub mod simulated_annealing;
pub mod genetic;
pub mod particle_swarm;
pub mod ant_colony;
pub mod tabu_search;
pub mod exact_minimum;
pub mod exact_maximum;

use std::fmt;
use std::str::FromStr;
use std::time::Instant;

use crate::color::{sort_colors_by_lab, Lab, Rgb};
use crate::distance::DistanceMatrix;
use crate::error::Error;
use crate::prng::Mulberry32;

/// Available color selection algorithms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Algorithm {
    Greedy,
    MaxSumGlobal,
    MaxSumSequential,
    KMeansPP,
    SimulatedAnnealing,
    GeneticAlgorithm,
    ParticleSwarm,
    AntColony,
    TabuSearch,
    ExactMinimum,
    ExactMaximum,
    Random,
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algorithm::Greedy => write!(f, "greedy"),
            Algorithm::MaxSumGlobal => write!(f, "max_sum_global"),
            Algorithm::MaxSumSequential => write!(f, "max_sum_sequential"),
            Algorithm::KMeansPP => write!(f, "kmeans_pp"),
            Algorithm::SimulatedAnnealing => write!(f, "simulated_annealing"),
            Algorithm::GeneticAlgorithm => write!(f, "genetic_algorithm"),
            Algorithm::ParticleSwarm => write!(f, "particle_swarm"),
            Algorithm::AntColony => write!(f, "ant_colony"),
            Algorithm::TabuSearch => write!(f, "tabu_search"),
            Algorithm::ExactMinimum => write!(f, "exact_minimum"),
            Algorithm::ExactMaximum => write!(f, "exact_maximum"),
            Algorithm::Random => write!(f, "random"),
        }
    }
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // snake_case (Rust/Python convention)
            "greedy" => Ok(Algorithm::Greedy),
            "max_sum_global" | "maxSumDistancesGlobal" | "maxSumGlobal" => {
                Ok(Algorithm::MaxSumGlobal)
            }
            "max_sum_sequential" | "maxSumDistancesSequential" | "maxSumSequential" => {
                Ok(Algorithm::MaxSumSequential)
            }
            "kmeans_pp" | "kmeanspp" | "kmeansppSelection" => Ok(Algorithm::KMeansPP),
            "simulated_annealing" | "simulatedAnnealing" => Ok(Algorithm::SimulatedAnnealing),
            "genetic_algorithm" | "geneticAlgorithm" | "genetic" => {
                Ok(Algorithm::GeneticAlgorithm)
            }
            "particle_swarm" | "particleSwarmOptimization" | "particleSwarm" => {
                Ok(Algorithm::ParticleSwarm)
            }
            "ant_colony" | "antColonyOptimization" | "antColony" => Ok(Algorithm::AntColony),
            "tabu_search" | "tabuSearch" | "tabu" => Ok(Algorithm::TabuSearch),
            "exact_minimum" | "exactMinimum" | "exactMinDistance" => Ok(Algorithm::ExactMinimum),
            "exact_maximum" | "exactMaximum" | "exactMaxDistance" => Ok(Algorithm::ExactMaximum),
            "random" | "randomSelection" => Ok(Algorithm::Random),
            _ => Err(Error::UnknownAlgorithm(s.to_string())),
        }
    }
}

/// Algorithm-specific tuning parameters.
/// Only the fields relevant to the chosen algorithm are read; others are ignored.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlgorithmOptions {
    // Simulated Annealing
    pub initial_temp: Option<f64>,
    pub cooling_rate: Option<f64>,
    pub min_temp: Option<f64>,

    // Genetic Algorithm
    pub population_size: Option<usize>,
    pub generations: Option<usize>,
    pub mutation_rate: Option<f64>,

    // Particle Swarm
    pub num_particles: Option<usize>,
    pub pso_iterations: Option<usize>,
    pub inertia_weight: Option<f64>,
    pub cognitive_weight: Option<f64>,
    pub social_weight: Option<f64>,

    // Ant Colony
    pub num_ants: Option<usize>,
    pub aco_iterations: Option<usize>,
    pub evaporation_rate: Option<f64>,
    pub pheromone_importance: Option<f64>,
    pub heuristic_importance: Option<f64>,

    // Tabu Search
    pub tabu_iterations: Option<usize>,
    pub tabu_tenure: Option<usize>,

    // Exact algorithms: max combinations before returning error
    pub exact_limit: Option<u128>,
}

/// The result of running a color-selection algorithm.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionResult {
    /// The selected colors, sorted by Lab (L desc, a desc, b desc).
    pub colors: Vec<Rgb>,
    /// Execution time in milliseconds.
    pub time_ms: f64,
}

/// Run an algorithm and return a sorted SelectionResult.
pub(crate) fn run_algorithm(
    algorithm: Algorithm,
    pool: &[Rgb],
    labs: &[Lab],
    select_count: usize,
    dist_matrix: Option<&DistanceMatrix>,
    options: &AlgorithmOptions,
    seed: u32,
) -> crate::error::Result<SelectionResult> {
    let start = Instant::now();
    let mut rng = Mulberry32::new(seed);

    let selected_indices = match algorithm {
        Algorithm::Random => random::run(pool.len(), select_count, &mut rng),
        Algorithm::Greedy => greedy::run(labs, select_count, &mut rng),
        Algorithm::MaxSumGlobal => max_sum_global::run(labs, select_count, dist_matrix),
        Algorithm::MaxSumSequential => max_sum_sequential::run(labs, select_count, &mut rng),
        Algorithm::KMeansPP => kmeans_pp::run(labs, select_count, &mut rng),
        Algorithm::SimulatedAnnealing => {
            simulated_annealing::run(labs, select_count, options, &mut rng, dist_matrix)
        }
        Algorithm::GeneticAlgorithm => {
            genetic::run(labs, select_count, options, &mut rng, dist_matrix)
        }
        Algorithm::ParticleSwarm => {
            particle_swarm::run(labs, select_count, options, &mut rng, dist_matrix)
        }
        Algorithm::AntColony => {
            ant_colony::run(labs, select_count, options, &mut rng, dist_matrix)
        }
        Algorithm::TabuSearch => {
            tabu_search::run(labs, select_count, options, &mut rng, dist_matrix)
        }
        Algorithm::ExactMinimum => {
            exact_minimum::run(labs, select_count, options, dist_matrix)?
        }
        Algorithm::ExactMaximum => {
            exact_maximum::run(labs, select_count, options, dist_matrix)?
        }
    };

    let mut colors: Vec<Rgb> = selected_indices.iter().map(|&i| pool[i]).collect();
    let selected_labs: Vec<Lab> = selected_indices.iter().map(|&i| labs[i]).collect();
    sort_colors_by_lab(&mut colors, &selected_labs);

    let elapsed = start.elapsed();
    Ok(SelectionResult {
        colors,
        time_ms: elapsed.as_secs_f64() * 1000.0,
    })
}
