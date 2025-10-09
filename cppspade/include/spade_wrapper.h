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
//   inner_loops: vector of hole/island polygons (each must be closed)
//   maxh: target maximum edge length (converted to area constraint)
//   quality: refinement quality level
//   enforce_constraints: whether to honor PSLG edges as constraints
// Returns: TriangulationResult containing vertices, triangles, and edges
TriangulationResult triangulate(
    const std::vector<Point>& outer,
    const std::vector<std::vector<Point>>& inner_loops,
    double maxh,
    Quality quality = Quality::Default,
    bool enforce_constraints = true
);

} // namespace spade

#endif // SPADE_WRAPPER_H
