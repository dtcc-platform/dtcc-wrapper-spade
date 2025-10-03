use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation, RefinementParameters, AngleLimit};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Deserialize)]
struct Input {
    outer: Vec<[f64; 2]>,
    inner_loops: Vec<Vec<[f64; 2]>>,
    maxh: Option<f64>,
    quality: String,
    enforce_constraints: bool,
    min_angle: Option<f64>,  // Minimum angle in degrees
    exclude_holes: Option<bool>,  // If true, exclude inner loops as holes (default: true)
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

    // Build vertex list and edge list for CDT
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut vertex_idx = 0;

    // Add outer loop vertices
    let outer_start = vertex_idx;
    for &[x, y] in &input.outer {
        vertices.push(Point2::new(x, y));
        vertex_idx += 1;
    }
    let outer_end = vertex_idx;

    // Create edges for outer loop
    for i in outer_start..outer_end {
        let next = if i + 1 < outer_end { i + 1 } else { outer_start };
        edges.push([i, next]);
    }

    // Add inner loops
    for inner in &input.inner_loops {
        let inner_start = vertex_idx;
        for &[x, y] in inner {
            vertices.push(Point2::new(x, y));
            vertex_idx += 1;
        }
        let inner_end = vertex_idx;

        // Create edges for inner loop
        for i in inner_start..inner_end {
            let next = if i + 1 < inner_end { i + 1 } else { inner_start };
            edges.push([i, next]);
        }
    }

    // Create CDT - use incremental insertion to avoid bulk_load deduplication issues
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::default();
    let mut vertex_handles = Vec::new();

    for vertex in vertices {
        let handle = cdt.insert(vertex)?;
        vertex_handles.push(handle);
    }

    // Add constraint edges if requested
    let has_constraints = input.enforce_constraints && !edges.is_empty();
    if has_constraints {
        for [i, j] in &edges {
            if *i != *j && *i < vertex_handles.len() && *j < vertex_handles.len() {
                let vi = vertex_handles[*i];
                let vj = vertex_handles[*j];
                if vi != vj {
                    cdt.add_constraint(vi, vj);
                }
            }
        }
    }

    // Use refinement to properly identify and exclude holes (only if we have constraint edges)
    let should_exclude_holes = input.exclude_holes.unwrap_or(true);  // Default: exclude holes
    let excluded_faces = if has_constraints && should_exclude_holes {
        // Set up refinement parameters
        let mut params = RefinementParameters::<f64>::new()
            .exclude_outer_faces(true);  // Exclude outer boundary and holes

        // Add maxh constraint if specified
        if let Some(max_edge_len) = input.maxh {
            // Convert edge length to approximate triangle area
            // For an equilateral triangle with edge length h:
            // area = (sqrt(3)/4) * h^2 ≈ 0.433 * h^2
            let max_area = 0.433 * max_edge_len * max_edge_len;
            params = params.with_max_allowed_area(max_area);
        }

        // Set angle limit - priority: min_angle param > quality setting > none
        if let Some(min_angle) = input.min_angle {
            params = params.with_angle_limit(AngleLimit::from_deg(min_angle));
        } else if input.quality == "moderate" {
            params = params.with_angle_limit(AngleLimit::from_deg(25.0));
        } else {
            // Default: no angle constraint
            params = params.with_angle_limit(AngleLimit::from_deg(0.0));
        }

        // Perform refinement and get excluded faces (holes + outer boundary)
        let result = cdt.refine(params);
        result.excluded_faces
    } else if has_constraints && !should_exclude_holes {
        // Have constraints but want to include holes - refine everything
        let mut params = RefinementParameters::<f64>::new()
            .exclude_outer_faces(false);  // Include everything

        // Add maxh constraint if specified
        if let Some(max_edge_len) = input.maxh {
            let max_area = 0.433 * max_edge_len * max_edge_len;
            params = params.with_max_allowed_area(max_area);
        }

        // Set angle limit
        if let Some(min_angle) = input.min_angle {
            params = params.with_angle_limit(AngleLimit::from_deg(min_angle));
        } else if input.quality == "moderate" {
            params = params.with_angle_limit(AngleLimit::from_deg(25.0));
        } else {
            params = params.with_angle_limit(AngleLimit::from_deg(0.0));
        }

        cdt.refine(params);
        Vec::new()
    } else {
        // No constraint edges: simple refinement without exclusions
        if let Some(max_edge_len) = input.maxh {
            let max_area = 0.433 * max_edge_len * max_edge_len;
            let mut params = RefinementParameters::<f64>::new()
                .with_max_allowed_area(max_area)
                .exclude_outer_faces(false);

            // Set angle limit
            if let Some(min_angle) = input.min_angle {
                params = params.with_angle_limit(AngleLimit::from_deg(min_angle));
            } else if input.quality == "moderate" {
                params = params.with_angle_limit(AngleLimit::from_deg(25.0));
            } else {
                params = params.with_angle_limit(AngleLimit::from_deg(0.0));
            }

            cdt.refine(params);
        }
        Vec::new()
    };

    // Convert excluded faces to a HashSet for fast lookup
    let excluded_set: std::collections::HashSet<_> = excluded_faces.into_iter().collect();

    // Extract points and triangles
    let mut point_map = std::collections::HashMap::new();
    let mut output_points = Vec::new();

    for (idx, vertex) in cdt.vertices().enumerate() {
        let pos = vertex.position();
        point_map.insert(vertex.fix(), idx);
        output_points.push([pos.x, pos.y, 0.0]);
    }

    let mut output_triangles = Vec::new();
    for face in cdt.inner_faces() {
        // Skip excluded faces (holes and outer boundary)
        if !excluded_set.contains(&face.fix()) {
            let vertices: [_; 3] = face.vertices().map(|v| point_map[&v.fix()]);
            output_triangles.push([vertices[0], vertices[1], vertices[2]]);
        }
    }

    // Extract constraint edges
    let mut constraint_edges = Vec::new();
    for edge in cdt.undirected_edges() {
        if edge.is_constraint_edge() {
            let [v0, v1] = edge.vertices().map(|v| point_map[&v.fix()]);
            constraint_edges.push([v0, v1]);
        }
    }

    // Output JSON result
    let output = Output {
        points: output_points,
        triangles: output_triangles,
        constraint_edges,
    };

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
