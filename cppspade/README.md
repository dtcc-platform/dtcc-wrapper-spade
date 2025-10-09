# Spade C++ Wrapper

A C++ wrapper for the Spade 2D triangulation library (Rust).

## Features

- Clean C++ API with modern C++17 design
- Support for constrained Delaunay triangulation (CDT)
- Mesh refinement with quality controls
- Polygon with holes support
- Easy-to-use interface with STL containers

## Building

### Prerequisites

- CMake 3.15+
- C++17 compatible compiler (GCC 7+, Clang 5+, MSVC 2017+)
- Rust toolchain (for building the FFI layer)

### Build Steps

```bash
# 1. Build the Rust FFI library
cargo build --release

# 2. Build the C++ wrapper
mkdir build
cd build
cmake ..
make

# 3. Run examples
./examples/simple_triangulation
./examples/polygon_with_holes
```

## Usage Example

```cpp
#include "spade_wrapper.h"
#include <vector>

int main() {
    // Define outer polygon (must be closed)
    std::vector<spade::Point> outer = {
        {0.0, 0.0, 0.0},
        {1.0, 0.0, 0.0},
        {1.0, 1.0, 0.0},
        {0.0, 1.0, 0.0},
        {0.0, 0.0, 0.0}
    };

    // Define holes (optional)
    std::vector<std::vector<spade::Point>> holes;

    // Triangulate
    auto result = spade::triangulate(
        outer,
        holes,
        0.5,                      // maxh - target edge length
        spade::Quality::Moderate,  // quality settings
        true                       // enforce constraints
    );

    // Access results
    std::cout << "Vertices: " << result.num_vertices() << "\n";
    std::cout << "Triangles: " << result.num_triangles() << "\n";

    return 0;
}
```

## API Reference

### Structures

- **`Point`**: Represents a 3D point with x, y, z coordinates
- **`Triangle`**: Triangle with three vertex indices (v0, v1, v2)
- **`Edge`**: Edge with two vertex indices (v0, v1)
- **`TriangulationResult`**: Contains the mesh data (points, triangles, edges)

### Quality Enum

- **`Quality::Default`**: No angle constraints (0°)
- **`Quality::Moderate`**: Minimum angle of 25°

### Main Function

```cpp
TriangulationResult triangulate(
    const std::vector<Point>& outer,           // Exterior polygon
    const std::vector<std::vector<Point>>& inner_loops,  // Holes
    double maxh,                                // Target max edge length
    Quality quality = Quality::Default,         // Refinement quality
    bool enforce_constraints = true             // Honor PSLG edges
);
```

## Architecture

The wrapper consists of three layers:

1. **Rust FFI Layer** (`src/lib.rs`): C-compatible exports from Rust Spade library
2. **C FFI Header** (`include/spade_ffi.h`): C interface definition
3. **C++ Wrapper** (`src/spade_wrapper.cpp`): Modern C++ API using the C FFI

This design provides:
- Memory safety through RAII and smart pointers
- Automatic resource cleanup
- Exception-based error handling
- Type-safe interfaces

## License

Same as the Spade library: MIT OR Apache-2.0
