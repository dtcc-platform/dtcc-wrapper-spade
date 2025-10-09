use spade::{ConstrainedDelaunayTriangulation, Point2, RefinementParameters, AngleLimit, Triangulation};
use std::collections::HashMap;

#[repr(C)]
pub struct SpadePoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
pub struct SpadeTriangle {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

#[repr(C)]
pub struct SpadeEdge {
    pub v0: usize,
    pub v1: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum SpadeQuality {
    Default = 0,
    Moderate = 1,
}

pub struct SpadeResult {
    pub points: Vec<SpadePoint>,
    pub triangles: Vec<SpadeTriangle>,
    pub edges: Vec<SpadeEdge>,
}

/// Perform triangulation
/// Returns opaque handle to result, or NULL on failure
#[no_mangle]
pub extern "C" fn spade_triangulate(
    outer_points: *const SpadePoint,
    outer_count: usize,
    inner_loops: *const *const SpadePoint,
    inner_loop_counts: *const usize,
    num_inner_loops: usize,
    maxh: f64,
    quality: SpadeQuality,
    enforce_constraints: i32,
) -> *mut SpadeResult {
    // Safety: convert raw pointers to slices
    if outer_points.is_null() || outer_count == 0 {
        return std::ptr::null_mut();
    }

    let outer_slice = unsafe { std::slice::from_raw_parts(outer_points, outer_count) };

    // Convert outer points
    let mut outer: Vec<(f64, f64)> = outer_slice.iter().map(|p| (p.x, p.y)).collect();

    // Ensure outer loop is closed
    if let (Some(first), Some(last)) = (outer.first(), outer.last()) {
        if (first.0 - last.0).abs() > 1e-10 || (first.1 - last.1).abs() > 1e-10 {
            outer.push(*first);
        }
    }

    // Convert inner loops
    let mut inner_loops_vec: Vec<Vec<(f64, f64)>> = Vec::new();
    if !inner_loops.is_null() && !inner_loop_counts.is_null() && num_inner_loops > 0 {
        let inner_ptrs = unsafe { std::slice::from_raw_parts(inner_loops, num_inner_loops) };
        let inner_counts = unsafe { std::slice::from_raw_parts(inner_loop_counts, num_inner_loops) };

        for i in 0..num_inner_loops {
            if !inner_ptrs[i].is_null() && inner_counts[i] > 0 {
                let inner_slice = unsafe { std::slice::from_raw_parts(inner_ptrs[i], inner_counts[i]) };
                let mut inner: Vec<(f64, f64)> = inner_slice.iter().map(|p| (p.x, p.y)).collect();

                // Ensure inner loop is closed
                if let (Some(first), Some(last)) = (inner.first(), inner.last()) {
                    if (first.0 - last.0).abs() > 1e-10 || (first.1 - last.1).abs() > 1e-10 {
                        inner.push(*first);
                    }
                }

                inner_loops_vec.push(inner);
            }
        }
    }

    // Perform triangulation
    match triangulate_polygon(
        outer,
        inner_loops_vec,
        if maxh > 0.0 { Some(maxh) } else { None },
        match quality {
            SpadeQuality::Moderate => "moderate".to_string(),
            SpadeQuality::Default => "default".to_string(),
        },
        enforce_constraints != 0,
    ) {
        Ok(result) => Box::into_raw(Box::new(result)),
        Err(_) => std::ptr::null_mut(),
    }
}

fn triangulate_polygon(
    outer: Vec<(f64, f64)>,
    inner_loops: Vec<Vec<(f64, f64)>>,
    maxh: Option<f64>,
    quality: String,
    enforce_constraints: bool,
) -> Result<SpadeResult, Box<dyn std::error::Error>> {
    // Build vertex list and edge list for CDT
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut vertex_idx = 0;

    // Add outer loop vertices
    let outer_start = vertex_idx;
    for &(x, y) in &outer {
        vertices.push(Point2::new(x, y));
        vertex_idx += 1;
    }
    let outer_end = vertex_idx;

    // Create edges for outer loop (skip duplicate closing point)
    let outer_vertex_count = if outer.len() > 1 &&
        (outer[0].0 - outer[outer.len()-1].0).abs() < 1e-10 &&
        (outer[0].1 - outer[outer.len()-1].1).abs() < 1e-10 {
        outer.len() - 1
    } else {
        outer.len()
    };

    for i in 0..outer_vertex_count {
        let next = (i + 1) % outer_vertex_count;
        edges.push([outer_start + i, outer_start + next]);
    }

    // Add inner loops
    for inner in &inner_loops {
        let inner_start = vertex_idx;
        for &(x, y) in inner {
            vertices.push(Point2::new(x, y));
            vertex_idx += 1;
        }
        let inner_end = vertex_idx;

        // Create edges for inner loop (skip duplicate closing point)
        let inner_vertex_count = if inner.len() > 1 &&
            (inner[0].0 - inner[inner.len()-1].0).abs() < 1e-10 &&
            (inner[0].1 - inner[inner.len()-1].1).abs() < 1e-10 {
            inner.len() - 1
        } else {
            inner.len()
        };

        for i in 0..inner_vertex_count {
            let next = (i + 1) % inner_vertex_count;
            edges.push([inner_start + i, inner_start + next]);
        }
    }

    // Create CDT
    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::default();
    let mut vertex_handles = Vec::new();

    for vertex in vertices {
        let handle = cdt.insert(vertex)?;
        vertex_handles.push(handle);
    }

    // Add constraint edges if requested
    let has_constraints = enforce_constraints && !edges.is_empty();
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

    // Refinement with hole exclusion
    let excluded_faces = if has_constraints {
        let mut params = RefinementParameters::<f64>::new()
            .exclude_outer_faces(true);

        if let Some(max_edge_len) = maxh {
            let max_area = 0.433 * max_edge_len * max_edge_len;
            params = params.with_max_allowed_area(max_area);
        }

        if quality == "moderate" {
            params = params.with_angle_limit(AngleLimit::from_deg(25.0));
        } else {
            params = params.with_angle_limit(AngleLimit::from_deg(0.0));
        }

        let result = cdt.refine(params);
        result.excluded_faces
    } else {
        if let Some(max_edge_len) = maxh {
            let max_area = 0.433 * max_edge_len * max_edge_len;
            let mut params = RefinementParameters::<f64>::new()
                .with_max_allowed_area(max_area)
                .exclude_outer_faces(false);

            if quality == "moderate" {
                params = params.with_angle_limit(AngleLimit::from_deg(25.0));
            } else {
                params = params.with_angle_limit(AngleLimit::from_deg(0.0));
            }

            cdt.refine(params);
        }
        Vec::new()
    };

    let excluded_set: std::collections::HashSet<_> = excluded_faces.into_iter().collect();

    // Extract points and triangles
    let mut point_map = HashMap::new();
    let mut output_points = Vec::new();

    for (idx, vertex) in cdt.vertices().enumerate() {
        let pos = vertex.position();
        point_map.insert(vertex.fix(), idx);
        output_points.push(SpadePoint { x: pos.x, y: pos.y, z: 0.0 });
    }

    let mut output_triangles = Vec::new();
    for face in cdt.inner_faces() {
        if !excluded_set.contains(&face.fix()) {
            let vertices: [_; 3] = face.vertices().map(|v| point_map[&v.fix()]);
            output_triangles.push(SpadeTriangle {
                v0: vertices[0],
                v1: vertices[1],
                v2: vertices[2],
            });
        }
    }

    // Extract constraint edges
    let mut constraint_edges = Vec::new();
    for edge in cdt.undirected_edges() {
        if edge.is_constraint_edge() {
            let [v0, v1] = edge.vertices().map(|v| point_map[&v.fix()]);
            constraint_edges.push(SpadeEdge { v0, v1 });
        }
    }

    Ok(SpadeResult {
        points: output_points,
        triangles: output_triangles,
        edges: constraint_edges,
    })
}

/// Get number of points in result
#[no_mangle]
pub extern "C" fn spade_result_num_points(result: *const SpadeResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).points.len() }
}

/// Get number of triangles in result
#[no_mangle]
pub extern "C" fn spade_result_num_triangles(result: *const SpadeResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).triangles.len() }
}

/// Get number of edges in result
#[no_mangle]
pub extern "C" fn spade_result_num_edges(result: *const SpadeResult) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { (*result).edges.len() }
}

/// Get points from result (copies into user-provided buffer)
#[no_mangle]
pub extern "C" fn spade_result_get_points(result: *const SpadeResult, buffer: *mut SpadePoint) {
    if result.is_null() || buffer.is_null() {
        return;
    }
    unsafe {
        let points = &(*result).points;
        std::ptr::copy_nonoverlapping(points.as_ptr(), buffer, points.len());
    }
}

/// Get triangles from result (copies into user-provided buffer)
#[no_mangle]
pub extern "C" fn spade_result_get_triangles(result: *const SpadeResult, buffer: *mut SpadeTriangle) {
    if result.is_null() || buffer.is_null() {
        return;
    }
    unsafe {
        let triangles = &(*result).triangles;
        std::ptr::copy_nonoverlapping(triangles.as_ptr(), buffer, triangles.len());
    }
}

/// Get edges from result (copies into user-provided buffer)
#[no_mangle]
pub extern "C" fn spade_result_get_edges(result: *const SpadeResult, buffer: *mut SpadeEdge) {
    if result.is_null() || buffer.is_null() {
        return;
    }
    unsafe {
        let edges = &(*result).edges;
        std::ptr::copy_nonoverlapping(edges.as_ptr(), buffer, edges.len());
    }
}

/// Free the result
#[no_mangle]
pub extern "C" fn spade_result_free(result: *mut SpadeResult) {
    if !result.is_null() {
        unsafe {
            let _ = Box::from_raw(result);
        }
    }
}
