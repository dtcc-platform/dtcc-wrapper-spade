use pyo3::prelude::*;
use pyo3::types::PyList;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation, RefinementParameters, AngleLimit};
use std::collections::HashMap;

pub mod core;

/// Triangulate a polygon with optional holes using Spade CDT
///
/// Args:
///     outer: List of (x, y) tuples for exterior boundary
///     inner_loops: List of inner loops (holes), each as list of (x, y) tuples
///     maxh: Optional maximum edge length for refinement
///     quality: "default" or "moderate" - sets angle limit (25° for moderate)
///     enforce_constraints: Whether to add constraint edges
///     min_angle: Optional minimum angle in degrees (overrides quality)
///     exclude_holes: Whether to exclude inner loops as holes (default: True)
///
/// Returns:
///     Tuple of (points, triangles, constraint_edges)
///     - points: List of (x, y, z) coordinates (z=0.0)
///     - triangles: List of (i, j, k) vertex indices
///     - constraint_edges: List of (i, j) edge indices
#[pyfunction]
#[pyo3(signature = (outer, inner_loops, maxh=None, quality="default", enforce_constraints=false, min_angle=None, exclude_holes=true))]
fn triangulate(
    py: Python,
    outer: Vec<(f64, f64)>,
    inner_loops: Vec<Vec<(f64, f64)>>,
    maxh: Option<f64>,
    quality: &str,
    enforce_constraints: bool,
    min_angle: Option<f64>,
    exclude_holes: bool,
) -> PyResult<(Vec<(f64, f64, f64)>, Vec<(usize, usize, usize)>, Vec<(usize, usize)>)> {
    // Call core triangulation function
    let result = core::triangulate_polygon(
        outer,
        inner_loops,
        maxh,
        quality.to_string(),
        enforce_constraints,
        min_angle,
        exclude_holes,
    ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Ok((result.points, result.triangles, result.constraint_edges))
}

/// Spade 2D triangulation module
#[pymodule]
fn spade_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(triangulate, m)?)?;
    Ok(())
}
