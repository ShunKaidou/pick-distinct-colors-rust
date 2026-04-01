use crate::algorithms::AlgorithmOptions;
use crate::color::Lab;
use crate::distance::{delta_e, DistanceMatrix};
use crate::prng::{fisher_yates_shuffle, Mulberry32};

/// Compute minimum pairwise distance for a selection.
fn calculate_fitness(
    selection: &[usize],
    labs: &[Lab],
    dist_matrix: Option<&DistanceMatrix>,
) -> f64 {
    let mut min_dist = f64::INFINITY;
    for i in 0..selection.len() {
        for j in (i + 1)..selection.len() {
            let d = match dist_matrix {
                Some(dm) => dm.get(selection[i], selection[j]),
                None => delta_e(&labs[selection[i]], &labs[selection[j]]),
            };
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

struct Particle {
    position: Vec<usize>,
    in_position: Vec<bool>,
    best_position: Vec<usize>,
    best_fitness: f64,
}

/// Particle Swarm Optimization for color selection.
///
/// Fixes from JS:
/// - Proper discrete PSO: probability-based position update toward pBest/gBest
///   instead of the broken continuous velocity model that degenerates to random swaps
/// - Fisher-Yates for initialization
/// - Vec<bool> for O(1) membership
pub(crate) fn run(
    labs: &[Lab],
    select_count: usize,
    options: &AlgorithmOptions,
    rng: &mut Mulberry32,
    dist_matrix: Option<&DistanceMatrix>,
) -> Vec<usize> {
    let n = labs.len();
    let num_particles = options.num_particles.unwrap_or(30);
    let max_iterations = options.pso_iterations.unwrap_or(100);
    let w = options.inertia_weight.unwrap_or(0.7);
    let c1 = options.cognitive_weight.unwrap_or(1.5);
    let c2 = options.social_weight.unwrap_or(1.5);

    // Initialize particles
    let mut particles: Vec<Particle> = (0..num_particles)
        .map(|_| {
            let mut indices: Vec<usize> = (0..n).collect();
            fisher_yates_shuffle(&mut indices, rng);
            let position = indices[..select_count].to_vec();
            let mut in_position = vec![false; n];
            for &idx in &position {
                in_position[idx] = true;
            }
            let fitness = calculate_fitness(&position, labs, dist_matrix);
            Particle {
                best_position: position.clone(),
                best_fitness: fitness,
                position,
                in_position,
            }
        })
        .collect();

    let mut global_best_position = particles[0].best_position.clone();
    let mut global_best_fitness = particles[0].best_fitness;

    for p in &particles {
        if p.best_fitness > global_best_fitness {
            global_best_fitness = p.best_fitness;
            global_best_position = p.best_position.clone();
        }
    }

    // Main loop
    for _ in 0..max_iterations {
        for particle in &mut particles {
            // Probability-based discrete PSO update:
            // For each position in the selection, with probability proportional to
            // cognitive/social weights, adopt from pBest or gBest
            #[allow(clippy::needless_range_loop)]
            for i in 0..select_count {
                let r1 = rng.next_f64();
                let r2 = rng.next_f64();
                let r_inertia = rng.next_f64();

                // Probability of keeping current (inertia)
                if r_inertia < w {
                    continue; // Keep current position[i]
                }

                let cognitive_prob = c1 * r1 / (c1 * r1 + c2 * r2 + 0.001);

                let target = if rng.next_f64() < cognitive_prob {
                    particle.best_position[i] // Move toward personal best
                } else {
                    global_best_position[i] // Move toward global best
                };

                // If target is different and not already in position, swap
                if target != particle.position[i] && !particle.in_position[target] {
                    let old = particle.position[i];
                    particle.in_position[old] = false;
                    particle.position[i] = target;
                    particle.in_position[target] = true;
                }
            }

            // Evaluate fitness
            let fitness = calculate_fitness(&particle.position, labs, dist_matrix);

            if fitness > particle.best_fitness {
                particle.best_position = particle.position.clone();
                particle.best_fitness = fitness;

                if fitness > global_best_fitness {
                    global_best_fitness = fitness;
                    global_best_position = particle.position.clone();
                }
            }
        }
    }

    global_best_position
}
