#include "spade_wrapper.h"
#include "spade_ffi.h"
#include <cmath>
#include <memory>
#include <sstream>
#include <stdexcept>

namespace spade {

TriangulationResult triangulate(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& holes,
    const std::vector<std::vector<Point>>& interior_loops,
    double maxh,
    Quality quality,
    bool enforce_constraints
) {
    if (outer.empty()) {
        throw std::invalid_argument("Outer polygon must have at least one point");
    }

    auto convert_loop = [](const std::vector<Point>& loop) {
        std::vector<SpadePoint> converted;
        converted.reserve(loop.size() + 1);
        for (const auto& p : loop) {
            converted.push_back({p.x, p.y, p.z});
        }
        if (!converted.empty()) {
            const auto& first = converted.front();
            const auto& last = converted.back();
            if (std::fabs(first.x - last.x) > 1e-10 || std::fabs(first.y - last.y) > 1e-10) {
                converted.push_back(first);
            }
        }
        return converted;
    };

    // Convert outer points to C format
    std::vector<SpadePoint> outer_c = convert_loop(outer);

    // Convert hole loops
    std::vector<std::vector<SpadePoint>> holes_c;
    holes_c.reserve(holes.size());
    for (const auto& hole : holes) {
        auto converted = convert_loop(hole);
        if (!converted.empty()) {
            holes_c.push_back(std::move(converted));
        }
    }

    std::vector<const SpadePoint*> hole_ptrs;
    std::vector<size_t> hole_counts;
    hole_ptrs.reserve(holes_c.size());
    hole_counts.reserve(holes_c.size());
    for (const auto& hole : holes_c) {
        hole_ptrs.push_back(hole.data());
        hole_counts.push_back(hole.size());
    }

    // Convert building loops
    std::vector<std::vector<SpadePoint>> interior_loops_c;
    interior_loops_c.reserve(interior_loops.size());
    for (const auto& loop : interior_loops) {
        auto converted = convert_loop(loop);
        if (!converted.empty()) {
            interior_loops_c.push_back(std::move(converted));
        }
    }

    std::vector<const SpadePoint*> building_ptrs;
    std::vector<size_t> building_counts;
    building_ptrs.reserve(interior_loops_c.size());
    building_counts.reserve(interior_loops_c.size());
    for (const auto& loop : interior_loops_c) {
        building_ptrs.push_back(loop.data());
        building_counts.push_back(loop.size());
    }

    // Convert quality enum
    SpadeQuality quality_c = (quality == Quality::Moderate) ?
        SPADE_QUALITY_MODERATE : SPADE_QUALITY_DEFAULT;

    // Call FFI function
    SpadeResult* result_ptr = spade_triangulate(
        outer_c.data(),
        outer_c.size(),
        hole_ptrs.empty() ? nullptr : hole_ptrs.data(),
        hole_counts.empty() ? nullptr : hole_counts.data(),
        hole_ptrs.size(),
        building_ptrs.empty() ? nullptr : building_ptrs.data(),
        building_counts.empty() ? nullptr : building_counts.data(),
        building_ptrs.size(),
        maxh,
        quality_c,
        enforce_constraints ? 1 : 0
    );

    if (!result_ptr) {
        SpadeError error{};
        if (spade_last_error(&error)) {
            std::ostringstream msg;
            msg << "Triangulation failed (code=" << error.code
                << ", poly_id=" << error.poly_id
                << ", seg_idx=" << error.seg_idx << ")";
            throw std::runtime_error(msg.str());
        }
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
