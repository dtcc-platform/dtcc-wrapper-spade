#!/usr/bin/env python3
"""
Adapter for Spade 2D triangulator.
"""

import json
import subprocess
from pathlib import Path
from typing import List, Tuple, Optional

# Path to the Rust CLI executable
SPADE_CLI = Path(__file__).parent / "spade-cli" / "target" / "release" / "spade-cli"


def triangulate(
    outer: List[Tuple[float, float]],
    inner_loops: List[List[Tuple[float, float]]],
    *,
    maxh: Optional[float] = None,
    quality: str = "default",
    enforce_constraints: bool = False,
    min_angle: Optional[float] = None
) -> Tuple[List[Tuple[float, float, float]], List[Tuple[int, int, int]], List[Tuple[int, int]]]:
    """
    Triangulate a polygon using Spade.

    Args:
        outer: Exterior polygon vertices as list of (x, y) tuples
        inner_loops: List of inner polygons (holes/islands), each as list of (x, y) tuples
        maxh: Target maximum edge length (converted to area constraint)
        quality: "default" or "moderate" - refinement quality level
        enforce_constraints: If True, enforce PSLG edges as constraints
        min_angle: Minimum angle in degrees (overrides quality setting)

    Returns:
        Tuple of:
        - points_xyz: List of (x, y, z) vertex coordinates (z=0.0)
        - triangles: List of (i, j, k) triangle vertex indices
        - lines: List of (i, j) constraint edge indices
    """

    # Prepare input for Rust CLI
    input_data = {
        "outer": [[x, y] for x, y in outer],
        "inner_loops": [[[x, y] for x, y in loop] for loop in inner_loops],
        "maxh": maxh,
        "quality": quality,
        "enforce_constraints": enforce_constraints,
        "min_angle": min_angle,
    }

    # Call Rust CLI
    result = subprocess.run(
        [str(SPADE_CLI)],
        input=json.dumps(input_data),
        capture_output=True,
        text=True,
        timeout=300,
    )

    if result.returncode != 0:
        raise RuntimeError(f"Spade CLI failed: {result.stderr}")

    # Parse output
    output = json.loads(result.stdout)

    points = [tuple(p) for p in output["points"]]
    triangles = [tuple(t) for t in output["triangles"]]
    lines = [tuple(e) for e in output["constraint_edges"]]

    return points, triangles, lines
