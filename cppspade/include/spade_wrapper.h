#ifndef SPADE_WRAPPER_H
#define SPADE_WRAPPER_H

#include <cstddef>
#include <vector>

namespace spade {

// Point structure for vertices
struct Point {
    double x;
    double y;
    double z;
};

// Triangle structure (indices into vertex array)
struct Triangle {
    size_t v0;
    size_t v1;
    size_t v2;
};

// Edge/Line structure (indices into vertex array)
struct Edge {
    size_t v0;
    size_t v1;
};

// Quality settings for mesh refinement
enum class Quality {
    Default,
    Moderate
};

// Result structure containing the triangulated mesh
struct TriangulationResult {
    std::vector<Point> points;
    std::vector<Triangle> triangles;
    std::vector<Edge> edges;

    // Statistics
    size_t num_vertices() const { return points.size(); }
    size_t num_triangles() const { return triangles.size(); }
    size_t num_edges() const { return edges.size(); }
};

// Main triangulation function
// Parameters:
//   outer: exterior polygon vertices (must be closed, i.e., first == last)
//   holes: vector of hole polygons (each must be closed, these regions will be excluded)
//   interior_loops: vector of constraint-only polygons (each must be closed; interiors are meshed)
//   maxh: target maximum edge length (converted to area constraint)
//   quality: refinement quality level
//   enforce_constraints: whether to honor PSLG edges as constraints
// Returns: TriangulationResult containing vertices, triangles, and edges
TriangulationResult triangulate(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& holes,
    const std::vector<std::vector<Point>>& interior_loops,
    double maxh,
    Quality quality = Quality::Default,
    bool enforce_constraints = true
);

// Convenience overload when only building loops are provided (no explicit holes).
inline TriangulationResult triangulate(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& interior_loops,
    double maxh,
    Quality quality = Quality::Default,
    bool enforce_constraints = true
) {
    return triangulate(outer, {}, interior_loops, maxh, quality, enforce_constraints);
}

// Convenience overload when holes are provided but no building loops.
inline TriangulationResult triangulate_with_holes(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& holes,
    double maxh,
    Quality quality = Quality::Default,
    bool enforce_constraints = true
) {
    return triangulate(outer, holes, {}, maxh, quality, enforce_constraints);
}

} // namespace spade

#endif // SPADE_WRAPPER_H
