#include "spade_wrapper.h"
#include <iostream>
#include <iomanip>

int main() {
    try {
        // Create an outer square polygon
        std::vector<spade::Point> outer = {
            {0.0, 0.0, 0.0},
            {10.0, 0.0, 0.0},
            {10.0, 10.0, 0.0},
            {0.0, 10.0, 0.0},
            {0.0, 0.0, 0.0}  // Close the polygon
        };

        // Create inner loops (holes)
        std::vector<std::vector<spade::Point>> inner_loops;

        // First hole - a square in the center
        std::vector<spade::Point> hole1 = {
            {4.0, 4.0, 0.0},
            {6.0, 4.0, 0.0},
            {6.0, 6.0, 0.0},
            {4.0, 6.0, 0.0},
            {4.0, 4.0, 0.0}  // Close the polygon
        };
        inner_loops.push_back(hole1);

        // Second hole - a triangle in the corner
        std::vector<spade::Point> hole2 = {
            {1.0, 1.0, 0.0},
            {2.5, 1.0, 0.0},
            {1.0, 2.5, 0.0},
            {1.0, 1.0, 0.0}  // Close the polygon
        };
        inner_loops.push_back(hole2);

        // Triangulate with moderate quality
        std::cout << "Triangulating a 10x10 square with two holes...\n";
        std::cout << "  Outer: square (0,0) to (10,10)\n";
        std::cout << "  Hole 1: square (4,4) to (6,6)\n";
        std::cout << "  Hole 2: triangle at (1,1)\n\n";

        auto result = spade::triangulate(
            outer,
            inner_loops,
            1.0,  // maxh = 1.0
            spade::Quality::Moderate,
            true  // enforce constraints
        );

        // Print results
        std::cout << "Triangulation results:\n";
        std::cout << "  Vertices: " << result.num_vertices() << "\n";
        std::cout << "  Triangles: " << result.num_triangles() << "\n";
        std::cout << "  Constraint edges: " << result.num_edges() << "\n";

        // Print some vertices
        std::cout << "\nFirst 10 vertices:\n";
        for (size_t i = 0; i < result.points.size() && i < 10; ++i) {
            std::cout << "  v" << i << ": ("
                      << std::fixed << std::setprecision(3)
                      << result.points[i].x << ", "
                      << result.points[i].y << ", "
                      << result.points[i].z << ")\n";
        }
        if (result.points.size() > 10) {
            std::cout << "  ... and " << (result.points.size() - 10) << " more\n";
        }

        // Print some triangles
        std::cout << "\nFirst 10 triangles:\n";
        for (size_t i = 0; i < result.triangles.size() && i < 10; ++i) {
            std::cout << "  t" << i << ": ("
                      << result.triangles[i].v0 << ", "
                      << result.triangles[i].v1 << ", "
                      << result.triangles[i].v2 << ")\n";
        }
        if (result.triangles.size() > 10) {
            std::cout << "  ... and " << (result.triangles.size() - 10) << " more\n";
        }

        // Print constraint edges
        std::cout << "\nConstraint edges (boundary and holes):\n";
        for (size_t i = 0; i < result.edges.size() && i < 15; ++i) {
            std::cout << "  e" << i << ": ("
                      << result.edges[i].v0 << " -> "
                      << result.edges[i].v1 << ")\n";
        }
        if (result.edges.size() > 15) {
            std::cout << "  ... and " << (result.edges.size() - 15) << " more\n";
        }

        std::cout << "\nSuccess! Note: triangles inside holes are excluded.\n";
        return 0;

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
