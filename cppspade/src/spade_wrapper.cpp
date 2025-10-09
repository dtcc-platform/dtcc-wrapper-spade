#include "spade_wrapper.h"
#include "spade_ffi.h"
#include <stdexcept>
#include <memory>

namespace spade {

TriangulationResult triangulate(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& inner_loops,
    double maxh,
    Quality quality,
    bool enforce_constraints
) {
    if (outer.empty()) {
        throw std::invalid_argument("Outer polygon must have at least one point");
    }

    // Convert outer points to C format
    std::vector<SpadePoint> outer_c(outer.size());
    for (size_t i = 0; i < outer.size(); ++i) {
        outer_c[i] = {outer[i].x, outer[i].y, outer[i].z};
    }

    // Convert inner loops to C format
    std::vector<std::vector<SpadePoint>> inner_loops_c;
    std::vector<const SpadePoint*> inner_loops_ptrs;
    std::vector<size_t> inner_loop_counts;

    for (const auto& inner : inner_loops) {
        if (!inner.empty()) {
            std::vector<SpadePoint> inner_c(inner.size());
            for (size_t i = 0; i < inner.size(); ++i) {
                inner_c[i] = {inner[i].x, inner[i].y, inner[i].z};
            }
            inner_loops_c.push_back(std::move(inner_c));
        }
    }

    // Create pointer array for inner loops
    for (const auto& inner : inner_loops_c) {
        inner_loops_ptrs.push_back(inner.data());
        inner_loop_counts.push_back(inner.size());
    }

    // Convert quality enum
    SpadeQuality quality_c = (quality == Quality::Moderate) ?
        SPADE_QUALITY_MODERATE : SPADE_QUALITY_DEFAULT;

    // Call FFI function
    SpadeResult* result_ptr = spade_triangulate(
        outer_c.data(),
        outer_c.size(),
        inner_loops_ptrs.empty() ? nullptr : inner_loops_ptrs.data(),
        inner_loop_counts.empty() ? nullptr : inner_loop_counts.data(),
        inner_loops_ptrs.size(),
        maxh,
        quality_c,
        enforce_constraints ? 1 : 0
    );

    if (!result_ptr) {
        throw std::runtime_error("Triangulation failed");
    }

    // Use unique_ptr with custom deleter for automatic cleanup
    std::unique_ptr<SpadeResult, decltype(&spade_result_free)> result(
        result_ptr,
        &spade_result_free
    );

    // Get sizes
    size_t num_points = spade_result_num_points(result.get());
    size_t num_triangles = spade_result_num_triangles(result.get());
    size_t num_edges = spade_result_num_edges(result.get());

    // Allocate output vectors
    TriangulationResult output;
    output.points.resize(num_points);
    output.triangles.resize(num_triangles);
    output.edges.resize(num_edges);

    // Copy data from C structs
    if (num_points > 0) {
        std::vector<SpadePoint> points_c(num_points);
        spade_result_get_points(result.get(), points_c.data());
        for (size_t i = 0; i < num_points; ++i) {
            output.points[i] = {points_c[i].x, points_c[i].y, points_c[i].z};
        }
    }

    if (num_triangles > 0) {
        std::vector<SpadeTriangle> triangles_c(num_triangles);
        spade_result_get_triangles(result.get(), triangles_c.data());
        for (size_t i = 0; i < num_triangles; ++i) {
            output.triangles[i] = {triangles_c[i].v0, triangles_c[i].v1, triangles_c[i].v2};
        }
    }

    if (num_edges > 0) {
        std::vector<SpadeEdge> edges_c(num_edges);
        spade_result_get_edges(result.get(), edges_c.data());
        for (size_t i = 0; i < num_edges; ++i) {
            output.edges[i] = {edges_c[i].v0, edges_c[i].v1};
        }
    }

    return output;
}

} // namespace spade
