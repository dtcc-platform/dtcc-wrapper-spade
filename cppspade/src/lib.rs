use spade::{
    AngleLimit, ConstrainedDelaunayTriangulation, Point2, RefinementParameters, Triangulation,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

const ERR_UNKNOWN: i32 = 0;
const ERR_CONSTRAINT_PANIC: i32 = 1;

static CURRENT_EDGE_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);

#[derive(Clone)]
struct EdgeDebugInfo {
    poly_id: i32,
    seg_idx: usize,
    start: usize,
    end: usize,
}

thread_local! {
    static EDGE_DEBUG_INFO: RefCell<Vec<EdgeDebugInfo>> = RefCell::new(Vec::new());
}

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

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SpadeError {
    pub code: i32,
    pub poly_id: i32,
    pub seg_idx: usize,
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
    hole_loops: *const *const SpadePoint,
    hole_loop_counts: *const usize,
    num_hole_loops: usize,
    interior_loops: *const *const SpadePoint,
    interior_loop_counts: *const usize,
    num_interior_loops: usize,
    maxh: f64,
    quality: SpadeQuality,
    enforce_constraints: i32,
) -> *mut SpadeResult {
    if outer_points.is_null() || outer_count == 0 {
        return std::ptr::null_mut();
    }

    let attempt = catch_unwind(AssertUnwindSafe(|| unsafe {
        let outer_slice = std::slice::from_raw_parts(outer_points, outer_count);
        let outer = normalize_loop(
            outer_slice
                .iter()
                .map(|p| Point2::new(p.x, p.y))
                .collect::<Vec<_>>(),
        );

        let holes = convert_loop_group(hole_loops, hole_loop_counts, num_hole_loops);
        let interior_loops =
            convert_loop_group(interior_loops, interior_loop_counts, num_interior_loops);

        triangulate_polygon(
            outer,
            holes,
            interior_loops,
            if maxh > 0.0 { Some(maxh) } else { None },
            match quality {
                SpadeQuality::Moderate => "moderate".to_string(),
                SpadeQuality::Default => "default".to_string(),
            },
            enforce_constraints != 0,
        )
    }));

    let result_ptr = match attempt {
        Ok(Ok(result)) => {
            clear_last_error();
            Box::into_raw(Box::new(result))
        }
        Ok(Err(_)) => {
            store_last_error(SpadeError {
                code: ERR_UNKNOWN,
                poly_id: -1,
                seg_idx: 0,
            });
            std::ptr::null_mut()
        }
        Err(_) => {
            let err = current_constraint_error().unwrap_or(SpadeError {
                code: ERR_UNKNOWN,
                poly_id: -1,
                seg_idx: 0,
            });
            store_last_error(err);
            std::ptr::null_mut()
        }
    };

    EDGE_DEBUG_INFO.with(|info| info.borrow_mut().clear());
    CURRENT_EDGE_INDEX.store(usize::MAX, Ordering::SeqCst);

    result_ptr
}

fn triangulate_polygon(
    outer: Vec<Point2<f64>>,
    holes: Vec<Vec<Point2<f64>>>,
    interior_loops: Vec<Vec<Point2<f64>>>,
    maxh: Option<f64>,
    quality: String,
    enforce_constraints: bool,
) -> Result<SpadeResult, Box<dyn std::error::Error>> {
    // Build vertex list and edge list for CDT
    let mut vertices = Vec::new();
    let mut edges = Vec::new();
    let mut edge_debug = Vec::new();
    let mut vertex_idx = 0;

    let outer_len = outer.len();
    if outer_len < 3 {
        return Err("Outer boundary must contain at least three points".into());
    }

    let mut push_loop = |loop_points: &Vec<Point2<f64>>, poly_id: i32| {
        let start_index = vertex_idx;
        for point in loop_points {
            vertices.push(*point);
            vertex_idx += 1;
        }
        let loop_len = loop_points.len();
        if loop_len >= 2 {
            for i in 0..loop_len {
                let next = (i + 1) % loop_len;
                edges.push([start_index + i, start_index + next]);
                edge_debug.push(EdgeDebugInfo {
                    poly_id,
                    seg_idx: i,
                    start: start_index + i,
                    end: start_index + next,
                });
            }
        }
    };

    let mut hole_poly_id = 1;
    let mut building_poly_id = -1;

    push_loop(&outer, 0);

    for loop_points in &holes {
        if loop_points.is_empty() {
            continue;
        }
        push_loop(loop_points, hole_poly_id);
        hole_poly_id += 1;
    }

    for loop_points in &interior_loops {
        if loop_points.is_empty() {
            continue;
        }
        push_loop(loop_points, building_poly_id);
        building_poly_id -= 1;
    }

    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::default();
    let mut vertex_handles = Vec::with_capacity(vertices.len());

    for vertex in vertices {
        let handle = cdt.insert(vertex)?;
        vertex_handles.push(handle);
    }

    if enforce_constraints && !edges.is_empty() {
        EDGE_DEBUG_INFO.with(|storage| {
            *storage.borrow_mut() = edge_debug.clone();
        });

        for (edge_idx, ([i, j], meta)) in edges.iter().zip(edge_debug.iter()).enumerate() {
            if *i != *j && *i < vertex_handles.len() && *j < vertex_handles.len() {
                let vi = vertex_handles[*i];
                let vj = vertex_handles[*j];
                if vi != vj {
                    CURRENT_EDGE_INDEX.store(edge_idx, Ordering::SeqCst);
                    // eprintln!(
                    //     "[spade] inserting constraint poly_id={} seg_idx={} ({} -> {})",
                    //     meta.poly_id, meta.seg_idx, meta.start, meta.end
                    // );
                    cdt.add_constraint(vi, vj);
                }
            }
        }
        CURRENT_EDGE_INDEX.store(usize::MAX, Ordering::SeqCst);
    }

    let mut params = RefinementParameters::<f64>::new().exclude_outer_faces(false);
    let mut should_refine = enforce_constraints && !edges.is_empty();

    if let Some(max_edge_len) = maxh {
        let max_area = 0.433 * max_edge_len * max_edge_len;
        params = params.with_max_allowed_area(max_area);
        should_refine = true;
    }

    if quality == "moderate" {
        params = params.with_angle_limit(AngleLimit::from_deg(25.0));
        should_refine = true;
    } else {
        params = params.with_angle_limit(AngleLimit::from_deg(0.0));
    }

    if should_refine {
        cdt.refine(params);
    }

    let mut point_map = HashMap::new();
    let mut output_points = Vec::new();

    for (idx, vertex) in cdt.vertices().enumerate() {
        let pos = vertex.position();
        point_map.insert(vertex.fix(), idx);
        output_points.push(SpadePoint {
            x: pos.x,
            y: pos.y,
            z: 0.0,
        });
    }

    let mut output_triangles = Vec::new();
    for face in cdt.inner_faces() {
        let vertices_with_pos: [_; 3] =
            face.vertices().map(|v| (point_map[&v.fix()], v.position()));

        let centroid = Point2::new(
            (vertices_with_pos[0].1.x + vertices_with_pos[1].1.x + vertices_with_pos[2].1.x) / 3.0,
            (vertices_with_pos[0].1.y + vertices_with_pos[1].1.y + vertices_with_pos[2].1.y) / 3.0,
        );

        if point_in_polygon(&centroid, &outer)
            && holes.iter().all(|hole| !point_in_polygon(&centroid, hole))
        {
            output_triangles.push(SpadeTriangle {
                v0: vertices_with_pos[0].0,
                v1: vertices_with_pos[1].0,
                v2: vertices_with_pos[2].0,
            });
        }
    }

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

fn normalize_loop(mut points: Vec<Point2<f64>>) -> Vec<Point2<f64>> {
    if points.len() >= 2 && points.first() == points.last() {
        points.pop();
    }
    points
}

unsafe fn convert_loop_group(
    loops_ptr: *const *const SpadePoint,
    counts_ptr: *const usize,
    num_loops: usize,
) -> Vec<Vec<Point2<f64>>> {
    if loops_ptr.is_null() || counts_ptr.is_null() || num_loops == 0 {
        return Vec::new();
    }

    let ptrs = std::slice::from_raw_parts(loops_ptr, num_loops);
    let counts = std::slice::from_raw_parts(counts_ptr, num_loops);

    let mut result = Vec::with_capacity(num_loops);
    for i in 0..num_loops {
        if ptrs[i].is_null() || counts[i] == 0 {
            continue;
        }
        let slice = std::slice::from_raw_parts(ptrs[i], counts[i]);
        let loop_points = slice
            .iter()
            .map(|p| Point2::new(p.x, p.y))
            .collect::<Vec<_>>();
        result.push(normalize_loop(loop_points));
    }

    result
}

fn point_in_polygon(point: &Point2<f64>, polygon: &[Point2<f64>]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;

    for (i, pi) in polygon.iter().enumerate() {
        let pj = &polygon[j];

        if point_on_segment(point, pi, pj) {
            return true;
        }

        let intersects = ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x);

        if intersects {
            inside = !inside;
        }

        j = i;
    }

    inside
}

fn point_on_segment(point: &Point2<f64>, a: &Point2<f64>, b: &Point2<f64>) -> bool {
    const EPS: f64 = 1e-9;

    let ab_x = b.x - a.x;
    let ab_y = b.y - a.y;
    let ap_x = point.x - a.x;
    let ap_y = point.y - a.y;

    let cross = ab_x * ap_y - ab_y * ap_x;
    if cross.abs() > EPS {
        return false;
    }

    let dot = ap_x * ab_x + ap_y * ab_y;
    if dot < -EPS {
        return false;
    }

    let squared_len = ab_x * ab_x + ab_y * ab_y;
    if dot - squared_len > EPS {
        return false;
    }

    true
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
pub extern "C" fn spade_result_get_triangles(
    result: *const SpadeResult,
    buffer: *mut SpadeTriangle,
) {
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

#[no_mangle]
pub extern "C" fn spade_last_error(out: *mut SpadeError) -> bool {
    if out.is_null() {
        return false;
    }
    if let Some(err) = take_last_error() {
        unsafe {
            *out = err;
        }
        true
    } else {
        false
    }
}

fn last_error_storage() -> &'static Mutex<Option<SpadeError>> {
    static LAST_ERROR: OnceLock<Mutex<Option<SpadeError>>> = OnceLock::new();
    LAST_ERROR.get_or_init(|| Mutex::new(None))
}

fn store_last_error(err: SpadeError) {
    let storage = last_error_storage();
    if let Ok(mut slot) = storage.lock() {
        *slot = Some(err);
    }
}

fn clear_last_error() {
    let storage = last_error_storage();
    if let Ok(mut slot) = storage.lock() {
        slot.take();
    }
}

fn take_last_error() -> Option<SpadeError> {
    let storage = last_error_storage();
    storage.lock().ok().and_then(|mut guard| guard.take())
}

fn current_constraint_error() -> Option<SpadeError> {
    let idx = CURRENT_EDGE_INDEX.load(Ordering::SeqCst);
    if idx == usize::MAX {
        return None;
    }
    EDGE_DEBUG_INFO
        .with(|info| info.borrow().get(idx).cloned())
        .map(|meta| SpadeError {
            code: ERR_CONSTRAINT_PANIC,
            poly_id: meta.poly_id,
            seg_idx: meta.seg_idx,
        })
}
