"""Type stubs for pyspade"""

from typing import Dict, List, Optional, Tuple

def triangulate(
    outer: List[Tuple[float, float]],
    holes: Optional[List[List[Tuple[float, float]]]] = None,
    max_edge_length: Optional[float] = None,
    min_angle: Optional[float] = None,
    triangulate_holes: bool = False,
) -> Dict[str, List]:
    """
    Triangulate a polygon with optional holes using constrained Delaunay triangulation.

    Args:
        outer: Exterior boundary vertices as list of (x, y) tuples
        holes: List of hole polygons, each as list of (x, y) tuples
        max_edge_length: Target maximum edge length for mesh refinement
        min_angle: Minimum angle constraint in degrees (0-33.9°)
        triangulate_holes: If True, mesh inside holes; if False, exclude them

    Returns:
        Dictionary with keys:
            - 'vertices': List of (x, y, z) vertex coordinates (z=0.0)
            - 'triangles': List of (i, j, k) triangle vertex indices (0-based)
            - 'edges': List of (i, j) constrained edge indices

    Example:
        >>> import pyspade
        >>> result = pyspade.triangulate(
        ...     outer=[(0, 0), (10, 0), (10, 10), (0, 10)],
        ...     max_edge_length=1.0
        ... )
        >>> print(len(result['triangles']))
    """
    ...

__all__ = ["triangulate"]
