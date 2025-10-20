#include "spade_wrapper.h"
#include <exception>
#include <iostream>
#include <vector>

int main() {
    using spade::Point;

    // Simple square outer boundary
    std::vector<Point> outer = {
        {0.0, 0.0, 0.0},
        {1.0, 0.0, 0.0},
        {1.0, 1.0, 0.0},
        {0.0, 1.0, 0.0},
        {0.0, 0.0, 0.0},
    };

    // Two building loops that share a full side. The overlapping constraint segments
    // force SPADE to detect the intersection when enforcing PSLG constraints.
    std::vector<std::vector<Point>> building_loops = {
        {
            {0.25, 0.25, 0.0},
            {0.55, 0.25, 0.0},
            {0.55, 0.75, 0.0},
            {0.25, 0.75, 0.0},
            {0.25, 0.25, 0.0},
        },
        {
            {0.55, 0.25, 0.0},
            {0.85, 0.25, 0.0},
            {0.85, 0.75, 0.0},
            {0.55, 0.75, 0.0},
            {0.55, 0.25, 0.0},
        },
    };

    try {
        auto result = spade::triangulate(
            outer,
            {},              // holes
            building_loops,  // building loops
            /*maxh*/ 0.0,
            spade::Quality::Default,
            /*enforce_constraints*/ true);

        std::cout << "Unexpected success: generated "
                  << result.num_triangles() << " triangles\n";
    } catch (const std::exception &ex) {
        std::cerr << "Caught exception: " << ex.what() << '\n';
        std::cerr << "This demonstrates that intersecting constraint edges "
                     "trigger SPADE's safety checks.\n";
    }

    return 0;
}
