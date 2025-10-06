# Spade Python Bindings

This directory contains both a CLI tool and native Python bindings for the Spade 2D triangulation library.

## Architecture

The project uses PyO3 and maturin to expose Rust functions directly to Python:

```
spade-cli/
├── src/
│   ├── lib.rs          # PyO3 bindings for Python
│   ├── core.rs         # Shared triangulation logic
│   └── main.rs         # CLI binary (uses core.rs)
├── Cargo.toml          # Rust package config with PyO3
└── pyproject.toml      # Python package config for maturin
```

### Two interfaces to the same code:

1. **CLI binary** (`spade-cli`) - JSON input/output via subprocess
2. **Python module** (`spade_python`) - Direct function calls via PyO3

## Building

### Requirements

- Rust toolchain (cargo)
- Python 3.8+ with development headers
- maturin: `pip install maturin`

### Build the CLI binary

```bash
cargo build --release --bin spade-cli
```

Output: `target/release/spade-cli`

### Build the Python module

```bash
# Build wheel
maturin build --release --interpreter python3.11

# Or install in development mode (if no broken venv)
maturin develop --release
```

Output: `target/wheels/spade_python-*.whl`

### Install the Python module

```bash
pip install target/wheels/spade_python-0.1.0-cp311-cp311-*.whl
```

## Usage

### Python (native bindings)

```python
import spade_python

outer = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
inner_loops = [
    [(20.0, 20.0), (30.0, 20.0), (30.0, 30.0), (20.0, 30.0)]
]

points, triangles, edges = spade_python.triangulate(
    outer,
    inner_loops,
    maxh=5.0,                    # Max edge length
    quality="moderate",          # Angle limit (25°)
    enforce_constraints=True,    # Honor polygon edges
    min_angle=20.0,             # Override quality setting
    exclude_holes=True          # Treat inner loops as holes
)

print(f"Generated {len(triangles)} triangles")
```

### CLI (subprocess)

```bash
echo '{
  "outer": [[0,0], [100,0], [100,100], [0,100]],
  "inner_loops": [[[20,20], [30,20], [30,30], [20,30]]],
  "maxh": 5.0,
  "quality": "moderate",
  "enforce_constraints": true,
  "min_angle": 20.0,
  "exclude_holes": true
}' | ./target/release/spade-cli
```

## Performance Comparison

Benchmarked on city geometry (15,501 triangles):

| Method | Time | Throughput | Notes |
|--------|------|------------|-------|
| **Subprocess** | 28.78ms | 539K tri/sec | JSON + process spawn overhead |
| **PyO3 native** | 11.10ms | 1.4M tri/sec | Direct function call |
| **Speedup** | **2.59x** | | Removes 61.4% overhead |

### Overhead breakdown (subprocess):
- Process spawn: ~5-8ms
- JSON serialization/deserialization: ~5-10ms
- Actual triangulation: ~10-15ms

### Recommendation

**Use PyO3 native bindings** for:
- Interactive applications
- Batch processing many geometries
- Performance-critical code
- Integration with NumPy/SciPy workflows

**Use CLI subprocess** for:
- Standalone scripts
- Quick prototyping
- Non-Python languages
- When you can't install compiled extensions

## API Reference

### `spade_python.triangulate()`

```python
def triangulate(
    outer: List[Tuple[float, float]],
    inner_loops: List[List[Tuple[float, float]]],
    maxh: Optional[float] = None,
    quality: str = "default",
    enforce_constraints: bool = False,
    min_angle: Optional[float] = None,
    exclude_holes: bool = True
) -> Tuple[
    List[Tuple[float, float, float]],  # points (x, y, z=0)
    List[Tuple[int, int, int]],        # triangles (i, j, k)
    List[Tuple[int, int]]              # constraint_edges (i, j)
]
```

**Parameters:**

- `outer`: Exterior boundary vertices as (x, y) tuples
- `inner_loops`: List of holes/islands, each as list of (x, y) tuples
- `maxh`: Maximum edge length for refinement (converted to area = 0.433 × maxh²)
- `quality`: "default" (no angle limit) or "moderate" (25° minimum angle)
- `enforce_constraints`: Whether to enforce polygon edges as constraints
- `min_angle`: Minimum angle in degrees (overrides quality setting)
- `exclude_holes`: If True, exclude inner_loops as holes; if False, triangulate them

**Returns:**

- `points`: List of (x, y, z) vertex coordinates (z always 0.0)
- `triangles`: List of (i, j, k) triangle vertex indices (0-based)
- `constraint_edges`: List of (i, j) constrained edge indices

**Raises:**

- `RuntimeError`: If triangulation fails or parameters are invalid

## Examples

### Example 1: Simple mesh with holes excluded

```python
import spade_python

# 500×500 box with buildings as holes
outer = [(0, 0), (500, 0), (500, 500), (0, 500)]
buildings = [
    [(100, 100), (150, 100), (150, 150), (100, 150)],
    [(200, 200), (250, 200), (250, 250), (200, 250)],
]

points, triangles, edges = spade_python.triangulate(
    outer, buildings,
    maxh=10.0,
    min_angle=20.0,
    enforce_constraints=True,
    exclude_holes=True  # Don't mesh inside buildings
)

# Result: ~13,857 triangles covering 175,098 m² (excludes buildings)
```

### Example 2: Mesh everything including building interiors

```python
points, triangles, edges = spade_python.triangulate(
    outer, buildings,
    maxh=10.0,
    quality="moderate",
    enforce_constraints=True,
    exclude_holes=False  # Mesh building interiors too
)

# Result: ~15,501 triangles covering full 250,000 m² area
```

### Example 3: High-quality mesh with strict angle bounds

```python
points, triangles, edges = spade_python.triangulate(
    outer, buildings,
    maxh=7.0,
    min_angle=20.0,        # Strict 20° minimum angle
    enforce_constraints=True,
    exclude_holes=True
)

# Result: All triangles have ≥20° minimum angle
# 13,857 triangles, mean angle 43.68°, 74.4% near-equilateral
```

## Development

### Rebuilding after changes

```bash
# Update lib.rs, core.rs, or main.rs
vim src/lib.rs

# Rebuild CLI
cargo build --release --bin spade-cli

# Rebuild Python module
maturin build --release --interpreter python3.11
pip install --force-reinstall target/wheels/*.whl
```

### Testing

```bash
# Test CLI
echo '{"outer": [[0,0],[1,0],[1,1],[0,1]], "inner_loops": [], "enforce_constraints": false}' | \
  ./target/release/spade-cli

# Test Python module
python3 -c "
import spade_python
pts, tris, edges = spade_python.triangulate([(0,0), (1,0), (1,1), (0,1)], [])
assert len(tris) == 2
print('✅ Tests passed')
"
```

## License

Same as Spade library (Apache-2.0 or MIT)
