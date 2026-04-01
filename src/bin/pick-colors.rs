use clap::Parser;
use pick_distinct_colors::{
    pick_distinct_colors, Algorithm, AlgorithmOptions, Config, Rgb,
};
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "pick-colors")]
#[command(about = "Select maximally distinct colors using various algorithms")]
struct Cli {
    /// Number of colors to select
    #[arg(short, long)]
    count: usize,

    /// Algorithm to use
    #[arg(short, long, default_value = "greedy")]
    algorithm: String,

    /// Seed for deterministic random generation
    #[arg(short, long, default_value_t = 42)]
    seed: u32,

    /// Size of random color pool (if no colors provided)
    #[arg(short, long)]
    pool_size: Option<usize>,

    /// Output format: hex, rgb, or json
    #[arg(short, long, default_value = "hex")]
    format: String,

    // -- Algorithm-specific options --
    /// Simulated Annealing: initial temperature
    #[arg(long)]
    initial_temp: Option<f64>,

    /// Simulated Annealing: cooling rate
    #[arg(long)]
    cooling_rate: Option<f64>,

    /// Simulated Annealing: minimum temperature
    #[arg(long)]
    min_temp: Option<f64>,

    /// Genetic Algorithm: population size
    #[arg(long)]
    population_size: Option<usize>,

    /// Genetic Algorithm: number of generations
    #[arg(long)]
    generations: Option<usize>,

    /// Genetic Algorithm: mutation rate
    #[arg(long)]
    mutation_rate: Option<f64>,

    /// PSO: number of particles
    #[arg(long)]
    num_particles: Option<usize>,

    /// PSO: iterations
    #[arg(long)]
    pso_iterations: Option<usize>,

    /// ACO: number of ants
    #[arg(long)]
    num_ants: Option<usize>,

    /// ACO: iterations
    #[arg(long)]
    aco_iterations: Option<usize>,

    /// Tabu Search: max iterations
    #[arg(long)]
    tabu_iterations: Option<usize>,

    /// Tabu Search: tabu tenure
    #[arg(long)]
    tabu_tenure: Option<usize>,

    /// Exact algorithms: max combinations limit
    #[arg(long)]
    exact_limit: Option<u128>,
}

fn format_rgb(rgb: &Rgb) -> String {
    format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b)
}

fn main() {
    let cli = Cli::parse();

    let algorithm = match Algorithm::from_str(&cli.algorithm) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let options = AlgorithmOptions {
        initial_temp: cli.initial_temp,
        cooling_rate: cli.cooling_rate,
        min_temp: cli.min_temp,
        population_size: cli.population_size,
        generations: cli.generations,
        mutation_rate: cli.mutation_rate,
        num_particles: cli.num_particles,
        pso_iterations: cli.pso_iterations,
        num_ants: cli.num_ants,
        aco_iterations: cli.aco_iterations,
        tabu_iterations: cli.tabu_iterations,
        tabu_tenure: cli.tabu_tenure,
        exact_limit: cli.exact_limit,
        ..Default::default()
    };

    let mut config = Config::new(cli.count)
        .algorithm(algorithm)
        .seed(cli.seed)
        .options(options);

    if let Some(ps) = cli.pool_size {
        config = config.pool_size(ps);
    }

    match pick_distinct_colors(config) {
        Ok(result) => match cli.format.as_str() {
            "hex" => {
                for color in &result.colors {
                    println!("{}", color.to_hex());
                }
                eprintln!(
                    "Selected {} colors in {:.2}ms using {}",
                    result.colors.len(),
                    result.time_ms,
                    algorithm
                );
            }
            "rgb" => {
                for color in &result.colors {
                    println!("{}", format_rgb(color));
                }
                eprintln!(
                    "Selected {} colors in {:.2}ms using {}",
                    result.colors.len(),
                    result.time_ms,
                    algorithm
                );
            }
            "json" => {
                #[derive(serde::Serialize)]
                struct JsonOutput {
                    colors: Vec<JsonColor>,
                    time_ms: f64,
                    algorithm: String,
                }
                #[derive(serde::Serialize)]
                struct JsonColor {
                    hex: String,
                    rgb: [u8; 3],
                }
                let output = JsonOutput {
                    colors: result
                        .colors
                        .iter()
                        .map(|c| JsonColor {
                            hex: c.to_hex(),
                            rgb: [c.r, c.g, c.b],
                        })
                        .collect(),
                    time_ms: result.time_ms,
                    algorithm: algorithm.to_string(),
                };
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            other => {
                eprintln!("Unknown format: {other}. Use hex, rgb, or json.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
