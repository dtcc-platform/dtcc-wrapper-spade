use pyo3::prelude::*;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation, RefinementParameters, AngleLimit};
use std::collections::{HashMap, HashSet};

/// Triangulate a polygon with optional holes using constrained Delaunay triangulation.
///
/// Args:
///     outer (list): Exterior boundary vertices as list of (x, y) tuples
///     holes (list, optional): List of hole polygons, each as list of (x, y) tuples. Default: []
///     max_edge_length (float, optional): Target maximum edge length for mesh refinement
///     min_angle (float, optional): Minimum angle constraint in degrees (0-33.9°)
///     triangulate_holes (bool, optional): If True, mesh inside holes; if False, exclude them. Default: False
///
/// Returns:
///     dict: Dictionary with keys:
///         - 'vertices': List of (x, y, z) vertex coordinates (z=0.0)
///         - 'triangles': List of (i, j, k) triangle vertex indices (0-based)
///         - 'edges': List of (i, j) constrained edge indices
///
/// Example:
///     >>> import pyspade
///     >>> result = pyspade.triangulate(
///     ...     outer=[(0, 0), (10, 0), (10, 10), (0, 10)],
///     ...     holes=[[(2, 2), (4, 2), (4, 4), (2, 4)]],
///     ...     max_edge_length=1.0,
///     ...     min_angle=20.0
///     ... )
///     >>> print(f"Generated {len(result['triangles'])} triangles")
#[pyfunction]
#[pyo3(signature = (outer, holes=None, max_edge_length=None, min_angle=None, triangulate_holes=false))]
fn triangulate(
    outer: Vec<(f64, f64)>,
    holes: Option<Vec<Vec<(f64, f64)>>>,
    max_edge_length: Option<f64>,
    min_angle: Option<f64>,
    triangulate_holes: bool,
) -> PyResult<HashMap<String, PyObject>> {
    Python::with_gil(|py| {
        let result = triangulate_impl(outer, holes, max_edge_length, min_angle, triangulate_holes)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

        let mut output = HashMap::new();
        output.insert("vertices".to_string(), result.vertices.into_py(py));
        output.insert("triangles".to_string(), result.triangles.into_py(py));
        output.insert("edges".to_string(), result.edges.into_py(py));

        Ok(output)
    })
}

struct TriangulationResult {
    vertices: Vec<(f64, f64, f64)>,
    triangles: Vec<(usize, usize, usize)>,
    edges: Vec<(usize, usize)>,
}

fn triangulate_impl(
    outer: Vec<(f64, f64)>,
    holes: Option<Vec<Vec<(f64, f64)>>>,
    max_edge_length: Option<f64>,
    min_angle: Option<f64>,
    triangulate_holes: bool,
) -> Result<TriangulationResult, Box<dyn std::error::Error>> {
    let holes = holes.unwrap_or_default();

    // Build vertex list and constraint edges
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut vertex_idx = 0;

    // Add outer boundary
    let outer_start = vertex_idx;
    for &(x, y) in &outer {
        vertices.push(Point2::new(x, y));
        vertex_idx += 1;
    }
    let outer_end = vertex_idx;

    // Create constraint edges for outer boundary
    for i in outer_start..outer_end {
        let next = if i + 1 < outer_end { i + 1 } else { outer_start };
        edges.push([i, next]);
    }

    // Add holes
    for hole in &holes {
        let hole_start = vertex_idx;
        for &(x, y) in hole {
            vertices.push(Point2::new(x, y));
            vertex_idx += 1;
        }
        let hole_end = vertex_idx;

        // Create constraint edges for hole
        for i in hole_start..hole_end {
            let next = if i + 1 < hole_end { i + 1 } else { hole_start };
            edges.push([i, next]);
        }
    }

    // Create CDT using incremental insertion
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::default();
    let mut vertex_handles = Vec::new();

    for vertex in vertices {
        let handle = cdt.insert(vertex)?;
        vertex_handles.push(handle);
    }

    // Add constraint edges
    let has_constraints = !edges.is_empty();
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

    // Apply refinement if needed
    let should_exclude_holes = !triangulate_holes && !holes.is_empty();
    let excluded_faces = if has_constraints && (max_edge_length.is_some() || min_angle.is_some() || should_exclude_holes) {
        let mut params = RefinementParameters::<f64>::new()
            .exclude_outer_faces(should_exclude_holes);

        if let Some(max_edge) = max_edge_length {
            // Convert edge length to area: area ≈ 0.433 * edge²
            let max_area = 0.433 * max_edge * max_edge;
            params = params.with_max_allowed_area(max_area);
        }

        if let Some(angle) = min_angle {
            params = params.with_angle_limit(AngleLimit::from_deg(angle));
        }

        let result = cdt.refine(params);
        result.excluded_faces
    } else {
        Vec::new()
    };

    let excluded_set: HashSet<_> = excluded_faces.into_iter().collect();

    // Extract output vertices
    let mut point_map = HashMap::new();
    let mut output_vertices = Vec::new();

    for (idx, vertex) in cdt.vertices().enumerate() {
        let pos = vertex.position();
        point_map.insert(vertex.fix(), idx);
        output_vertices.push((pos.x, pos.y, 0.0));
    }

    // Extract triangles (exclude holes if requested)
    let mut output_triangles = Vec::new();
    for face in cdt.inner_faces() {
        if !excluded_set.contains(&face.fix()) {
            let vertices: [_; 3] = face.vertices().map(|v| point_map[&v.fix()]);
            output_triangles.push((vertices[0], vertices[1], vertices[2]));
        }
    }

    // Extract constraint edges
    let mut output_edges = Vec::new();
    for edge in cdt.undirected_edges() {
        if edge.is_constraint_edge() {
            let [v0, v1] = edge.vertices().map(|v| point_map[&v.fix()]);
            output_edges.push((v0, v1));
        }
    }

    Ok(TriangulationResult {
        vertices: output_vertices,
        triangles: output_triangles,
        edges: output_edges,
    })
}

/// pyspade - Fast 2D Delaunay triangulation for Python
///
/// This module provides Python bindings for the Spade library, a robust
/// implementation of constrained Delaunay triangulation with mesh refinement.
///
/// Example:
///     >>> import pyspade
///     >>> result = pyspade.triangulate(
///     ...     outer=[(0, 0), (100, 0), (100, 100), (0, 100)],
///     ...     max_edge_length=10.0
///     ... )
#[pymodule]
fn pyspade(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(triangulate, m)?)?;
    Ok(())
}
