#include "spade_wrapper.h"
#include <iostream>
#include <iomanip>

int main() {
    try {
        // Create a simple square polygon
        std::vector<spade::Point> outer = {
            {0.0, 0.0, 0.0},
            {1.0, 0.0, 0.0},
            {1.0, 1.0, 0.0},
            {0.0, 1.0, 0.0},
            {0.0, 0.0, 0.0}  // Close the polygon
        };

        // No inner loops (no holes)
        std::vector<std::vector<spade::Point>> inner_loops;

        // Triangulate with default settings
        std::cout << "Triangulating a simple unit square...\n";
        auto result = spade::triangulate(
            outer,
            inner_loops,
            0.5,  // maxh = 0.5
            spade::Quality::Default,
            true  // enforce constraints
        );

        // Print results
        std::cout << "\nTriangulation results:\n";
        std::cout << "  Vertices: " << result.num_vertices() << "\n";
        std::cout << "  Triangles: " << result.num_triangles() << "\n";
        std::cout << "  Edges: " << result.num_edges() << "\n";

        // Print vertices
        std::cout << "\nVertices:\n";
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

        // Print triangles
        std::cout << "\nTriangles:\n";
        for (size_t i = 0; i < result.triangles.size() && i < 10; ++i) {
            std::cout << "  t" << i << ": ("
                      << result.triangles[i].v0 << ", "
                      << result.triangles[i].v1 << ", "
                      << result.triangles[i].v2 << ")\n";
        }
        if (result.triangles.size() > 10) {
            std::cout << "  ... and " << (result.triangles.size() - 10) << " more\n";
        }

        std::cout << "\nSuccess!\n";
        return 0;

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}
