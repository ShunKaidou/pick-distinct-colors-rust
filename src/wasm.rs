use wasm_bindgen::prelude::*;

use crate::api::{pick_distinct_colors, Config};
use crate::algorithms::{Algorithm, AlgorithmOptions};
use crate::color::{rgb2lab, Rgb};
use crate::distance::delta_e;
use crate::metrics::{calculate_metrics, find_closest_pair, analyze_distribution};
use crate::pool::generate_pool;

use std::str::FromStr;

/// Pick maximally distinct colors. Accepts and returns JSON.
///
/// Input JSON: `{ "count": 8, "algorithm": "greedy", "seed": 42, "pool_size": 200, "options": { ... } }`
/// Output JSON: `{ "colors": [{ "hex": "#ff0000", "rgb": [255, 0, 0] }, ...], "time_ms": 0.12 }`
#[wasm_bindgen]
pub fn pick_colors(config_json: &str) -> Result<String, JsError> {
    let input: serde_json::Value =
        serde_json::from_str(config_json).map_err(|e| JsError::new(&e.to_string()))?;

    let count = input["count"]
        .as_u64()
        .ok_or_else(|| JsError::new("missing or invalid 'count'"))? as usize;

    let algorithm = match input.get("algorithm").and_then(|v| v.as_str()) {
        Some(s) => Algorithm::from_str(s).map_err(|e| JsError::new(&e.to_string()))?,
        None => Algorithm::Greedy,
    };

    let seed = input
        .get("seed")
        .and_then(|v| v.as_u64())
        .unwrap_or(42) as u32;

    let mut config = Config::new(count).algorithm(algorithm).seed(seed);

    if let Some(ps) = input.get("pool_size").and_then(|v| v.as_u64()) {
        config = config.pool_size(ps as usize);
    }

    if let Some(colors_arr) = input.get("colors").and_then(|v| v.as_array()) {
        let colors: Result<Vec<Rgb>, JsError> = colors_arr
            .iter()
            .map(|c| {
                let arr = c
                    .as_array()
                    .ok_or_else(|| JsError::new("each color must be [r, g, b]"))?;
                if arr.len() != 3 {
                    return Err(JsError::new("each color must be [r, g, b]"));
                }
                Ok(Rgb::new(
                    arr[0].as_u64().unwrap_or(0) as u8,
                    arr[1].as_u64().unwrap_or(0) as u8,
                    arr[2].as_u64().unwrap_or(0) as u8,
                ))
            })
            .collect();
        config = config.colors(colors?);
    }

    if let Some(opts) = input.get("options") {
        let mut algo_opts = AlgorithmOptions::default();
        if let Some(v) = opts.get("initial_temp").and_then(|v| v.as_f64()) {
            algo_opts.initial_temp = Some(v);
        }
        if let Some(v) = opts.get("cooling_rate").and_then(|v| v.as_f64()) {
            algo_opts.cooling_rate = Some(v);
        }
        if let Some(v) = opts.get("min_temp").and_then(|v| v.as_f64()) {
            algo_opts.min_temp = Some(v);
        }
        if let Some(v) = opts.get("population_size").and_then(|v| v.as_u64()) {
            algo_opts.population_size = Some(v as usize);
        }
        if let Some(v) = opts.get("generations").and_then(|v| v.as_u64()) {
            algo_opts.generations = Some(v as usize);
        }
        if let Some(v) = opts.get("mutation_rate").and_then(|v| v.as_f64()) {
            algo_opts.mutation_rate = Some(v);
        }
        if let Some(v) = opts.get("num_particles").and_then(|v| v.as_u64()) {
            algo_opts.num_particles = Some(v as usize);
        }
        if let Some(v) = opts.get("pso_iterations").and_then(|v| v.as_u64()) {
            algo_opts.pso_iterations = Some(v as usize);
        }
        if let Some(v) = opts.get("num_ants").and_then(|v| v.as_u64()) {
            algo_opts.num_ants = Some(v as usize);
        }
        if let Some(v) = opts.get("aco_iterations").and_then(|v| v.as_u64()) {
            algo_opts.aco_iterations = Some(v as usize);
        }
        if let Some(v) = opts.get("tabu_iterations").and_then(|v| v.as_u64()) {
            algo_opts.tabu_iterations = Some(v as usize);
        }
        if let Some(v) = opts.get("tabu_tenure").and_then(|v| v.as_u64()) {
            algo_opts.tabu_tenure = Some(v as usize);
        }
        if let Some(v) = opts.get("exact_limit").and_then(|v| v.as_u64()) {
            algo_opts.exact_limit = Some(v as u128);
        }
        config = config.options(algo_opts);
    }

    let result = pick_distinct_colors(config).map_err(|e| JsError::new(&e.to_string()))?;

    let output = serde_json::json!({
        "colors": result.colors.iter().map(|c| {
            serde_json::json!({
                "hex": c.to_hex(),
                "rgb": [c.r, c.g, c.b]
            })
        }).collect::<Vec<_>>(),
        "time_ms": result.time_ms,
    });

    serde_json::to_string(&output).map_err(|e| JsError::new(&e.to_string()))
}

/// Convert an RGB color to CIELAB. Returns [L, a, b].
#[wasm_bindgen]
pub fn rgb_to_lab(r: u8, g: u8, b: u8) -> Vec<f64> {
    let lab = rgb2lab(Rgb::new(r, g, b));
    vec![lab.l, lab.a, lab.b]
}

/// CIE76 deltaE between two RGB colors.
#[wasm_bindgen]
pub fn color_distance(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> f64 {
    let a = rgb2lab(Rgb::new(r1, g1, b1));
    let b = rgb2lab(Rgb::new(r2, g2, b2));
    delta_e(&a, &b)
}

/// Generate a pool of random colors as JSON array of [r, g, b].
#[wasm_bindgen]
pub fn generate_color_pool(size: usize, seed: u32) -> String {
    let pool = generate_pool(size, seed);
    let arr: Vec<[u8; 3]> = pool.iter().map(|c| [c.r, c.g, c.b]).collect();
    serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
}

/// Calculate pairwise distance metrics for colors. Input: JSON array of [r,g,b].
/// Returns JSON: `{ "min": f64, "max": f64, "avg": f64, "sum": f64 }`
#[wasm_bindgen]
pub fn metrics(colors_json: &str) -> Result<String, JsError> {
    let colors = parse_color_array(colors_json)?;
    let m = calculate_metrics(&colors);
    let output = serde_json::json!({
        "min": m.min,
        "max": m.max,
        "avg": m.avg,
        "sum": m.sum,
    });
    serde_json::to_string(&output).map_err(|e| JsError::new(&e.to_string()))
}

/// Find the closest pair of colors. Input: JSON array of [r,g,b].
/// Returns JSON: `{ "colors": [[r,g,b], [r,g,b]], "distance": f64 }`
#[wasm_bindgen]
pub fn closest_pair(colors_json: &str) -> Result<String, JsError> {
    let colors = parse_color_array(colors_json)?;
    if colors.is_empty() {
        return Err(JsError::new("need at least 1 color"));
    }
    let pair = find_closest_pair(&colors);
    let output = serde_json::json!({
        "colors": [
            [pair.colors[0].r, pair.colors[0].g, pair.colors[0].b],
            [pair.colors[1].r, pair.colors[1].g, pair.colors[1].b],
        ],
        "distance": pair.distance,
    });
    serde_json::to_string(&output).map_err(|e| JsError::new(&e.to_string()))
}

/// Analyze color distribution coverage. Input: JSON array of [r,g,b].
/// Returns JSON: `{ "l_coverage": f64, "a_coverage": f64, "b_coverage": f64 }`
#[wasm_bindgen]
pub fn distribution(colors_json: &str) -> Result<String, JsError> {
    let colors = parse_color_array(colors_json)?;
    let d = analyze_distribution(&colors);
    let output = serde_json::json!({
        "l_coverage": d.l_coverage,
        "a_coverage": d.a_coverage,
        "b_coverage": d.b_coverage,
    });
    serde_json::to_string(&output).map_err(|e| JsError::new(&e.to_string()))
}

/// List all available algorithm names as JSON array.
#[wasm_bindgen]
pub fn available_algorithms() -> String {
    serde_json::json!([
        "greedy", "max_sum_global", "max_sum_sequential", "kmeans_pp",
        "simulated_annealing", "genetic_algorithm", "particle_swarm",
        "ant_colony", "tabu_search", "exact_minimum", "exact_maximum", "random"
    ])
    .to_string()
}

fn parse_color_array(json: &str) -> Result<Vec<Rgb>, JsError> {
    let arr: Vec<[u8; 3]> =
        serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(arr.into_iter().map(|[r, g, b]| Rgb::new(r, g, b)).collect())
}
