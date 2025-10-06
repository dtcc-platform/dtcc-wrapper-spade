# pyspade

[![PyPI](https://img.shields.io/pypi/v/pyspade.svg)](https://pypi.org/project/pyspade/)
[![Python](https://img.shields.io/pypi/pyversions/pyspade.svg)](https://pypi.org/project/pyspade/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/dtcc-platform/pyspade)

Fast 2D Delaunay triangulation library for Python with constrained edges and mesh refinement.

**pyspade** provides Python bindings to the [Spade](https://github.com/Stoeoef/spade) Rust library, offering high-performance constrained Delaunay triangulation (CDT) with quality mesh generation capabilities.

## Features

✨ **Fast** - Native Rust implementation with zero-copy data transfer
🎯 **Constrained** - Honor polygon boundaries and holes
📐 **Quality Refinement** - Control triangle size and angles
🔧 **Simple API** - Clean Pythonic interface
🌍 **Cross-platform** - Works on Linux, macOS, and Windows
🐍 **Python 3.10+** - Modern Python support

## Installation

```bash
pip install pyspade
```

Pre-built wheels are available for:
- **Python:** 3.10, 3.11, 3.12, 3.13
- **Platforms:** Linux (x86_64, aarch64), macOS (x86_64, arm64), Windows (x86_64)

## Quick Start

```python
import pyspade

# Define a simple square
outer = [(0, 0), (10, 0), (10, 10), (0, 10)]

# Triangulate it
result = pyspade.triangulate(outer)

print(f"Generated {len(result['triangles'])} triangles")
print(f"Vertices: {result['vertices'][:3]}")  # First 3 vertices
```

## Usage Examples

### Basic Triangulation

```python
import pyspade

# Simple polygon
outer = [(0, 0), (100, 0), (100, 100), (0, 100)]
result = pyspade.triangulate(outer)

# Access results
vertices = result['vertices']      # List of (x, y, z) tuples
triangles = result['triangles']    # List of (i, j, k) index tuples
edges = result['edges']            # List of (i, j) constraint edge indices
```

### Triangulation with Holes

```python
import pyspade

# Outer boundary
outer = [(0, 0), (100, 0), (100, 100), (0, 100)]

# Define holes (will be excluded from mesh)
holes = [
    [(20, 20), (30, 20), (30, 30), (20, 30)],  # Small square hole
    [(50, 50), (60, 50), (60, 60), (50, 60)],  # Another hole
]

result = pyspade.triangulate(outer, holes=holes)
```

### Mesh Refinement with Size Control

```python
import pyspade

result = pyspade.triangulate(
    outer=[(0, 0), (100, 0), (100, 100), (0, 100)],
    max_edge_length=5.0  # Target maximum edge length
)

# Result will have smaller, more uniform triangles
```

### Quality Mesh with Angle Constraints

```python
import pyspade

result = pyspade.triangulate(
    outer=[(0, 0), (100, 0), (100, 100), (0, 100)],
    max_edge_length=10.0,
    min_angle=20.0  # Minimum angle in degrees (avoids skinny triangles)
)

# Most triangles will have angles ≥ 20°
```

### Triangulating Holes (Not Excluding Them)

```python
import pyspade

# Sometimes you want to mesh the "holes" too
result = pyspade.triangulate(
    outer=[(0, 0), (100, 0), (100, 100), (0, 100)],
    holes=[[(20, 20), (30, 20), (30, 30), (20, 30)]],
    triangulate_holes=True,  # Mesh inside holes
    max_edge_length=5.0
)

# The hole area will now be triangulated
```

### Integration with NumPy and Matplotlib

```python
import pyspade
import numpy as np
import matplotlib.pyplot as plt
from matplotlib.tri import Triangulation

# Create mesh
outer = [(0, 0), (10, 0), (10, 10), (0, 10)]
result = pyspade.triangulate(outer, max_edge_length=1.0)

# Convert to numpy arrays
vertices = np.array(result['vertices'])
triangles = np.array(result['triangles'])

# Plot with matplotlib
triang = Triangulation(vertices[:, 0], vertices[:, 1], triangles)
plt.triplot(triang, 'b-', lw=0.5)
plt.plot(vertices[:, 0], vertices[:, 1], 'ro', ms=3)
plt.axis('equal')
plt.show()
```

### Real-World Example: City Buildings

```python
import pyspade

# City block (500m × 500m)
city_boundary = [(0, 0), (500, 0), (500, 500), (0, 500)]

# Building footprints as holes
buildings = [
    [(100, 100), (150, 100), (150, 150), (100, 150)],
    [(200, 200), (250, 200), (250, 250), (200, 250)],
    [(300, 50), (350, 50), (350, 100), (300, 100)],
    # ... more buildings
]

# Generate street-level mesh (excludes buildings)
result = pyspade.triangulate(
    outer=city_boundary,
    holes=buildings,
    max_edge_length=10.0,  # 10m triangles
    min_angle=20.0         # Good quality triangles
)

print(f"Street mesh: {len(result['triangles'])} triangles")
```

## API Reference

### `pyspade.triangulate()`

```python
def triangulate(
    outer: List[Tuple[float, float]],
    holes: Optional[List[List[Tuple[float, float]]]] = None,
    max_edge_length: Optional[float] = None,
    min_angle: Optional[float] = None,
    triangulate_holes: bool = False
) -> Dict[str, List]
```

**Parameters:**

- **`outer`** *(required)*: List of (x, y) tuples defining the exterior boundary
- **`holes`** *(optional)*: List of hole polygons, each as list of (x, y) tuples
- **`max_edge_length`** *(optional)*: Target maximum edge length for refinement
- **`min_angle`** *(optional)*: Minimum angle constraint in degrees (typically 20-30°)
- **`triangulate_holes`** *(optional)*: If True, mesh inside holes; if False, exclude them

**Returns:**

Dictionary with keys:
- **`vertices`**: List of (x, y, z) vertex coordinates (z is always 0.0)
- **`triangles`**: List of (i, j, k) triangle vertex indices (0-based)
- **`edges`**: List of (i, j) constrained edge indices

**Raises:**

- `RuntimeError`: If triangulation fails or parameters are invalid

## Performance

pyspade is **fast** because it:
- Uses Rust's native performance (no Python overhead for core algorithm)
- Implements efficient DCEL (Doubly Connected Edge List) data structure
- Avoids unnecessary memory allocations
- Provides zero-copy data transfer where possible

**Benchmark** (15,501 triangles, city geometry):
- **Triangulation time:** ~11ms
- **Throughput:** ~1.4M triangles/sec
- (Tested on Apple M1)

## Comparison with Other Libraries

| Feature | pyspade | Triangle | scipy.spatial.Delaunay | meshpy |
|---------|---------|----------|------------------------|--------|
| Constrained edges | ✅ | ✅ | ❌ | ✅ |
| Mesh refinement | ✅ | ✅ | ❌ | ✅ |
| Angle constraints | ✅ | ✅ | ❌ | ✅ |
| Pure Python wheels | ✅ | ❌ | ✅ | ❌ |
| No GPL dependencies | ✅ | ❌ | ✅ | ❌ |
| Modern Rust backend | ✅ | ❌ | ❌ | ❌ |

## Building from Source

Requires Rust toolchain (1.70+):

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/dtcc-platform/pyspade
cd pyspade
pip install maturin
maturin develop --release
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Built on top of the excellent [Spade](https://github.com/Stoeoef/spade) library by Stoeoef.

## Citation

If you use pyspade in your research, please cite:

```bibtex
@software{pyspade,
  title = {pyspade: Fast 2D Delaunay triangulation for Python},
  author = {Vasilis Naserentin},
  year = {2025},
  url = {https://github.com/dtcc-platform/pyspade}
}
```
