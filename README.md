# pick-distinct-colors

A Rust library for selecting maximally distinct colors from a pool using various algorithms in CIELAB color space with CIE76 deltaE as the distance metric.

This is an optimized Rust port of the JavaScript library [pick-distinct-colors](https://github.com/bdamokos/pick-distinct-colors) by Bence Damokos, with some small bug fixes and significant performance improvements.

## Features

- **12 color selection algorithms** ranging from fast heuristics to exact solvers
- **Deterministic output** with seedable PRNG (Mulberry32, bit-identical to the JS version)
- **Zero required runtime dependencies** (only `thiserror` for error derives)
- **Optional parallelism** via Rayon (feature-gated)
- **CLI binary** for quick command-line usage
- **Pre-computed distance matrices** with upper-triangular storage (half memory)
- **Branch-and-bound pruning** for exact algorithms

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
pick-distinct-colors = "0.1"
```

### Optional features

```toml
[dependencies]
pick-distinct-colors = { version = "0.1", features = ["parallel", "serde", "cli"] }
```

| Feature | Description |
|---------|-------------|
| `parallel` | Rayon-based parallel distance matrix computation |
| `serde` | Serialize/Deserialize for all public types |
| `cli` | Builds the `pick-colors` CLI binary |

## Quick Start

### Library usage

```rust
use pick_distinct_colors::{pick_distinct_colors, Config, Algorithm};

// Pick 8 distinct colors using the default greedy algorithm
let result = pick_distinct_colors(Config::new(8)).unwrap();
for color in &result.colors {
    println!("{}", color.to_hex());
}

// Use a specific algorithm with a custom seed
let result = pick_distinct_colors(
    Config::new(10)
        .algorithm(Algorithm::SimulatedAnnealing)
        .seed(12345)
        .pool_size(200)
).unwrap();

println!("Selected {} colors in {:.2}ms", result.colors.len(), result.time_ms);
```

### Custom color pool

```rust
use pick_distinct_colors::{pick_distinct_colors, Config, Algorithm, Rgb};

let my_colors = vec![
    Rgb::new(255, 0, 0),
    Rgb::new(0, 255, 0),
    Rgb::new(0, 0, 255),
    Rgb::new(255, 255, 0),
    Rgb::new(0, 255, 255),
    Rgb::new(255, 0, 255),
];

let result = pick_distinct_colors(
    Config::new(3)
        .colors(my_colors)
        .algorithm(Algorithm::Greedy)
).unwrap();
```

### CLI usage

Build with the `cli` feature:

```bash
cargo build --features cli --release
```

```bash
# Pick 8 colors in hex format (default)
pick-colors --count 8

# Use a specific algorithm with JSON output
pick-colors --count 10 --algorithm simulated_annealing --seed 42 --format json

# RGB format with custom pool size
pick-colors -c 5 -a greedy -s 12345 -p 200 -f rgb
```

**Output formats:** `hex` (default), `rgb`, `json`

## Algorithms

| Algorithm | Key | Time Complexity | Best For |
|-----------|-----|-----------------|----------|
| Greedy | `greedy` | O(n * k) | Fast, good results (default) |
| Max Sum Global | `max_sum_global` | O(n^2) | Colors far from the centroid |
| Max Sum Sequential | `max_sum_sequential` | O(n * k) | Balance of speed/quality |
| K-Means++ | `kmeans_pp` | O(n * k) | Well-distributed colors |
| Simulated Annealing | `simulated_annealing` | Variable | Escaping local optima |
| Genetic Algorithm | `genetic_algorithm` | O(p * g * k^2) | Complex search spaces |
| Particle Swarm | `particle_swarm` | O(p * i * k^2) | Swarm-based exploration |
| Ant Colony | `ant_colony` | O(a * i * k) | Discrete optimization |
| Tabu Search | `tabu_search` | O(i * k * n) | Avoiding revisited solutions |
| Exact Minimum | `exact_minimum` | O(C(n,k)) | Provably optimal (small inputs) |
| Exact Maximum | `exact_maximum` | O(C(n,k)) | Max pairwise distance |
| Random | `random` | O(k) | Baseline comparison |

Where: `n` = pool size, `k` = colors to select, `p` = population/particles, `g` = generations, `i` = iterations, `a` = ants.

## Algorithm-Specific Options

```rust
use pick_distinct_colors::{Config, Algorithm, AlgorithmOptions};

let opts = AlgorithmOptions {
    // Simulated Annealing
    initial_temp: Some(1000.0),
    cooling_rate: Some(0.995),
    min_temp: Some(0.1),

    // Genetic Algorithm
    population_size: Some(100),
    generations: Some(100),
    mutation_rate: Some(0.1),

    // Particle Swarm
    num_particles: Some(30),
    pso_iterations: Some(100),

    // Ant Colony
    num_ants: Some(20),
    aco_iterations: Some(100),

    // Tabu Search
    tabu_iterations: Some(1000),
    tabu_tenure: Some(5),

    // Exact algorithms: max combinations before error
    exact_limit: Some(1_000_000),

    ..Default::default()
};

let result = pick_distinct_colors(
    Config::new(8)
        .algorithm(Algorithm::GeneticAlgorithm)
        .options(opts)
).unwrap();
```

## API Reference

### Core Types

```rust
// An sRGB color
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

// A CIELAB color
pub struct Lab { pub l: f64, pub a: f64, pub b: f64 }

// Algorithm result
pub struct SelectionResult {
    pub colors: Vec<Rgb>,
    pub time_ms: f64,
}
```

### Color Utilities

```rust
use pick_distinct_colors::{Rgb, Lab, rgb2lab, delta_e, sort_colors};

let lab = rgb2lab(Rgb::new(255, 128, 0));
let hex = Rgb::new(255, 128, 0).to_hex(); // "#ff8000"

let red = Rgb::new(255, 0, 0).to_lab();
let green = Rgb::new(0, 255, 0).to_lab();
let distance = delta_e(&red, &green); // ~170.58
```

### Metrics

```rust
use pick_distinct_colors::{calculate_metrics, find_closest_pair, Rgb};

let colors = vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255)];

let metrics = calculate_metrics(&colors);
println!("Min deltaE: {:.2}", metrics.min);
println!("Avg deltaE: {:.2}", metrics.avg);

let pair = find_closest_pair(&colors);
println!("Closest pair distance: {:.2}", pair.distance);
```

## Determinism

By default, all operations are deterministic with seed `42`. The PRNG (Mulberry32) is bit-identical to the JavaScript implementation, so the same seed produces the same colors in both Rust and JS.

```rust
// These two calls always produce identical output
let r1 = pick_distinct_colors(Config::new(8).seed(42)).unwrap();
let r2 = pick_distinct_colors(Config::new(8).seed(42)).unwrap();
assert_eq!(r1.colors, r2.colors);
```

## Bug Fixes Over the JS Version

This Rust port fixes 13 bugs from the original JavaScript implementation:

- **Tabu Search:** randomized initial solution (JS used `[0,1,...,k-1]`), correct tabu recording (JS recorded wrong moves), seed actually used (JS ignored it)
- **Shuffling:** Fisher-Yates replacing biased `sort(() => rng()-0.5)` in SA, GA, PSO
- **K-Means++/ACO:** roulette wheel OOB prevention with binary search + clamping
- **ACO:** fitness-proportional pheromone deposit (JS deposited equally regardless of quality), uses precomputed distance matrix (JS recomputed deltaE despite having one)
- **PSO:** proper discrete adaptation with probability-based position updates (JS velocity model was broken)
- **All metaheuristics:** `Vec<bool>` for O(1) membership instead of O(n) `Array.includes()`
- **Exact Minimum:** branch-and-bound pruning (10-1000x faster than brute force)
- **Parameter names:** typed `AlgorithmOptions` struct prevents silent config mismatches

## Testing

```bash
# Run all 124 tests
cargo test

# Run specific test category
cargo test --test cross_validation   # PRNG parity with JS
cargo test --test property           # Universal invariants (all 12 algorithms)
cargo test --test quality            # Algorithm quality comparisons
cargo test --test regression         # Pinned output regression tests
cargo test --test edge_cases         # Boundary conditions and error paths
```

## License

MIT

## Credits

- Original JavaScript library by [Bence Damokos](https://github.com/bdamokos/pick-distinct-colors)
- Rust port generated with the help of [Claude Code](https://claude.ai/code) (Opus)
