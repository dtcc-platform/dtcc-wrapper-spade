use serde::{Deserialize, Serialize};
use std::io::{self, Read};

mod core;

#[derive(Deserialize)]
struct Input {
    outer: Vec<[f64; 2]>,
    inner_loops: Vec<Vec<[f64; 2]>>,
    maxh: Option<f64>,
    quality: String,
    enforce_constraints: bool,
    min_angle: Option<f64>,
    exclude_holes: Option<bool>,
}

#[derive(Serialize)]
struct Output {
    points: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    constraint_edges: Vec<[usize; 2]>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read JSON input from stdin
    let mut input_str = String::new();
    io::stdin().read_to_string(&mut input_str)?;
    let input: Input = serde_json::from_str(&input_str)?;

    // Convert input format
    let outer: Vec<(f64, f64)> = input.outer.iter().map(|&[x, y]| (x, y)).collect();
    let inner_loops: Vec<Vec<(f64, f64)>> = input
        .inner_loops
        .iter()
        .map(|loop_pts| loop_pts.iter().map(|&[x, y]| (x, y)).collect())
        .collect();

    // Call core triangulation function
    let result = core::triangulate_polygon(
        outer,
        inner_loops,
        input.maxh,
        input.quality,
        input.enforce_constraints,
        input.min_angle,
        input.exclude_holes.unwrap_or(true),
    )?;

    // Convert output format
    let output = Output {
        points: result.points.iter().map(|&(x, y, z)| [x, y, z]).collect(),
        triangles: result.triangles.iter().map(|&(i, j, k)| [i, j, k]).collect(),
        constraint_edges: result.constraint_edges.iter().map(|&(i, j)| [i, j]).collect(),
    };

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
