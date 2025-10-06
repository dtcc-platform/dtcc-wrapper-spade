#!/usr/bin/env python3
"""
Adapter for Spade 2D triangulator using native PyO3 bindings.
This version uses the compiled extension module instead of subprocess.
"""

from typing import List, Tuple, Optional
import spade_python


def triangulate(
    outer: List[Tuple[float, float]],
    inner_loops: List[List[Tuple[float, float]]],
    *,
    maxh: Optional[float] = None,
    quality: str = "default",
    enforce_constraints: bool = False,
    min_angle: Optional[float] = None,
    exclude_holes: bool = True
) -> Tuple[List[Tuple[float, float, float]], List[Tuple[int, int, int]], List[Tuple[int, int]]]:
    """
    Triangulate a polygon using Spade (native PyO3 bindings).

    Args:
        outer: Exterior polygon vertices as list of (x, y) tuples
        inner_loops: List of inner polygons (holes/islands), each as list of (x, y) tuples
        maxh: Target maximum edge length (converted to area constraint)
        quality: "default" or "moderate" - refinement quality level
        enforce_constraints: If True, enforce PSLG edges as constraints
        min_angle: Minimum angle in degrees (overrides quality setting)
        exclude_holes: If True, exclude inner loops as holes; if False, triangulate them (default: True)

    Returns:
        Tuple of:
        - points_xyz: List of (x, y, z) vertex coordinates (z=0.0)
        - triangles: List of (i, j, k) triangle vertex indices
        - lines: List of (i, j) constraint edge indices
    """
    # Call native Rust function directly
    points, triangles, lines = spade_python.triangulate(
        outer,
        inner_loops,
        maxh=maxh,
        quality=quality,
        enforce_constraints=enforce_constraints,
        min_angle=min_angle,
        exclude_holes=exclude_holes,
    )

    return points, triangles, lines
