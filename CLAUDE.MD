# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a benchmarking harness for evaluating the Spade 2D triangulation library. The project tests Spade's performance on constrained Delaunay triangulation (CDT) tasks with various mesh sizes and quality settings.

**Key components:**
- `spade/` - The Spade Rust library (Delaunay triangulation implementation)
- `testcase.txt` - Test geometry data (polygon exterior + inner loops/holes)
- Python harness & adapter (to be created) - Interface between benchmark framework and Spade

## Spade Library Architecture

**Language:** Rust (Cargo-based project)

**Core concepts:**
- **DCEL (Doubly Connected Edge List):** Underlying topology structure
- **Handles:** Fixed (`FixedHandleImpl<Type, Marker>`) and dynamic (`DynamicHandleImpl`) references to mesh elements
- **Directed/Undirected edges:** Directed edges use adjacent u32 indices (0/1, 2/3, etc.) for cache efficiency
- **Triangulation types:**
  - Basic Delaunay triangulation (`delaunay_triangulation`)
  - Constrained Delaunay triangulation - CDT (`cdt`)
  - Delaunay refinement (quality mesh generation)

**Module hierarchy** (simplified):
```
dcel & handles & iterators
         ↑
    dcel_operations
         ↑
bulk_load & triangulation_ext
         ↑
    triangulation
         ↑
    cdt & delaunay_triangulation
```

## Common Commands

### Building Spade

```bash
# Build in release mode (required for benchmarking)
cd spade
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Benchmarking Workflow

The benchmark expects:
1. A Python harness script (`mesh_bench_harness.py`)
2. An adapter module (`adapter_spade.py`) that wraps Spade's Rust API
3. Test geometry in `testcase.txt`

**Adapter requirements:**
- Must expose a `triangulate()` function accepting:
  - `outer`: exterior polygon vertices
  - `inner_loops`: list of hole/island polygons
  - `maxh`: target edge length (convert to area ≈ 0.433 * maxh²)
  - `quality`: refinement level ("default" | "moderate")
  - `enforce_constraints`: honor PSLG edges
- Returns: `(points_xyz, triangles, lines)` tuple

### Integration Notes

**Spade uses Rust**, so Python integration requires either:
- **PyO3/maturin** - Create Python bindings to call Rust directly
- **CLI wrapper** - Build a Rust CLI tool and subprocess from Python
- **FFI** - Create C-compatible exports and use ctypes/cffi

The adapter should handle:
- Polygon data format conversion
- Constraint edge marking (Spade supports CDT natively)
- Mesh quality/size parameter mapping
- Output extraction (vertices + triangles)

## Test Cases

**A:** Unit square (sanity check)
**B:** Unit square + inner polygon (constraint verification)
**C:** City geometry from testcase.txt (maxh=100, moderate quality)
**D:** Same as C with maxh sweep: [100, 50, 20, 10, 5, 2, 1]

## Output Requirements

Results go in `results_Spade/`:
- `build_Spade.log` - Build output
- `meta_Spade.json` - System/compiler metadata
- `*.vtu` - VTK Unstructured Grid mesh files (use meshio)
- `bench_Spade.csv` & `bench_Spade.json` - Performance metrics
- `run_Spade.log` - Runtime notes (optional)

## Key Spade Features to Leverage

- **Bulk loading:** More efficient than incremental insertion for large datasets
- **Constraint edges:** Use CDT type for enforcing polygon boundaries
- **Refinement:** Built-in Delaunay refinement for quality control (minimum angle limits)
- **Robust predicates:** Uses exact arithmetic kernel to avoid geometric degeneracies
