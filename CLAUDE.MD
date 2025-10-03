# Benchmarking 2D Triangle Mesh Generator Spade

## 0) Goal
Evaluate alternative 2D triangulators (Spade) for:
- correctness (constraints honored),
- basic quality (no degenerates),
- performance (triangles/second),
- practicality (build friction & API usability).

## 1) Inputs you will receive
- Spade and it's repo in this folder spade/.
- A local file `testcase.txt`:
  - Line 1: exterior polygon as `x1 y1 x2 y2 ...`
  - Lines 2..N: inner polygons (holes / islands) in the same format.
  - Coordinates in a ~500×500 box. **All inner edges must be constraints**.
  ## 2) What you must produce (artifacts)
Create an output folder (e.g., `results_Spade/`) containing:

1. **Build log:** `build_Spade.log`
2. **Meta info:** `meta_Spade.json` (OS/CPU/versions)
3. **VTU files:**
   - `A_unit_square_default.vtu`
   - `B_unit_square_with_inner_polygon.vtu`
   - `C_city_100.vtu`
   - `D_city_<maxh>.vtu` for each size in the sweep
4. **Benchmarks:**
   - `bench_Spade.csv`
   - `bench_Spade.json`
5. **Run log (optional):** `run_Spade.log` with notable errors/decisions

## 3) Environment & setup
- Use an **isolated environment** (conda env or virtualenv). Containers are OK.
- Prereqs: `git`, C/C++ toolchain, the project’s build system (`cmake`/`meson`/`make`), `python3`, `pip`.
- Install Python libs used by the harness:
  ```bash
  pip install --no-cache-dir meshio numpy
  ```
- The harness will record OS/CPU/RAM/compiler/Python versions into the meta JSON.

## 4) Download, build, install
- Download the **latest stable release** of the tool (or the latest tagged release if no stable tarball).
- Build in **Release** mode using the project’s documented steps.
- Install to a temporary prefix OR run from the build directory.
- Verify with `--version` or an equivalent help/health command.
- If build fails, stop and capture logs in `build_Spade.log`.

## 5) Use the reference harness (required)
- Save the code in the *Reference Harness* section below as **`mesh_bench_harness.py`**.
- Implement **one adapter** file `adapter_<tool>.py` exposing a single function:

  ```python
  def triangulate(
      outer,                # List[Tuple[float,float]]
      inner_loops,          # List[List[Tuple[float,float]]]
      *,
      maxh,                 # Optional[float]; None = default
      quality,              # "default" | "moderate"
      enforce_constraints,  # bool; honor PSLG edges if supported
  ):
      """
      Returns:
        points_xyz  : List[Tuple[float,float,float]]  # z=0.0
        triangles   : List[Tuple[int,int,int]]        # 0-based CCW
        lines       : List[Tuple[int,int]]            # optional, 0-based
      """
  ```

### Interpreting options inside adapters
- **Constraints:** If the tool supports PSLG/constrained edges, use it.
  If it only accepts “polygon + holes”, that’s acceptable for A/B; for C/D you must honor edges as constraints (split segments if needed). If impossible, note **FeatureMissing** in logs.
- **`maxh` (size):** If the tool uses **area**, set `area ≈ 0.433 * maxh^2`. If it uses a sizing function, provide a uniform field with that target. If unsupported, document and proceed.
- **`quality="moderate"`:** Use a non-extreme minimum angle/aspect setting if available; otherwise defaults.

## 6) Tests the harness will run
- **A:** Unit square, defaults (sanity).
- **B:** Unit square + one irregular inner polygon; edges must appear (constrained).
- **C:** City testcase (outer + many inner loops), `maxh=100`, moderate quality; constrained edges required.
- **D:** Same as C with `maxh` sweep: `100, 50, 20, 10, 5, 2, 1`. Record time, triangle count, triangles/sec.

**Timing** excludes file I/O; the harness times only the meshing call. Each size runs best-of `--repeats` (default 3).

## 7) How to run
```bash
python3 mesh_bench_harness.py \
  --software "Spade" \
  --adapter adapter_<tool>.py \
  --testcase testcase.txt \
  --outdir results_Spade \
  --sizes 100 50 20 10 5 2 1 \
  --repeats 3
```

## 8) Quality/validity checks (lightweight)
- Adapters should avoid zero-area elements; triangles must be valid.
- For constraint verification (B/C/D), each input segment should appear as an edge or chain of edges in the output. If the tool flags constrained edges, prefer that; otherwise nearest-node matching is acceptable for inspection.

## 9) Failure policy
- **BuildFailed:** stop; write logs and artifacts produced so far.
- **FeatureMissing:** if constraints or size control aren’t supported, mark in the notes and continue with what’s possible.
- **MeshFailed:** record error for that case/size and continue remaining sizes.