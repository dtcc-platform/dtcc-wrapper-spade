#!/usr/bin/env python3
"""
Basic examples of using pyspade for 2D triangulation.
"""

import pyspade


def example_simple_square():
    """Triangulate a simple square"""
    print("="*60)
    print("Example 1: Simple Square")
    print("="*60)

    outer = [(0, 0), (10, 0), (10, 10), (0, 10)]
    result = pyspade.triangulate(outer)

    print(f"Vertices: {len(result['vertices'])}")
    print(f"Triangles: {len(result['triangles'])}")
    print(f"First triangle: {result['triangles'][0]}")
    print()


def example_with_hole():
    """Triangulate a square with a hole"""
    print("="*60)
    print("Example 2: Square with Hole")
    print("="*60)

    outer = [(0, 0), (100, 0), (100, 100), (0, 100)]
    holes = [[(25, 25), (75, 25), (75, 75), (25, 75)]]

    result = pyspade.triangulate(outer, holes=holes)

    print(f"Vertices: {len(result['vertices'])}")
    print(f"Triangles: {len(result['triangles'])}")
    print(f"Constraint edges: {len(result['edges'])}")
    print()


def example_mesh_refinement():
    """Triangulate with mesh refinement"""
    print("="*60)
    print("Example 3: Mesh Refinement")
    print("="*60)

    outer = [(0, 0), (100, 0), (100, 100), (0, 100)]

    # Without refinement
    result_coarse = pyspade.triangulate(outer)

    # With refinement
    result_fine = pyspade.triangulate(
        outer,
        max_edge_length=10.0
    )

    print(f"Coarse mesh: {len(result_coarse['triangles'])} triangles")
    print(f"Fine mesh:   {len(result_fine['triangles'])} triangles")
    print()


def example_quality_mesh():
    """Triangulate with quality constraints"""
    print("="*60)
    print("Example 4: Quality Mesh (Angle Constraints)")
    print("="*60)

    outer = [(0, 0), (100, 0), (100, 100), (0, 100)]

    result = pyspade.triangulate(
        outer,
        max_edge_length=15.0,
        min_angle=20.0  # Avoid skinny triangles
    )

    print(f"Triangles: {len(result['triangles'])}")
    print(f"With min_angle=20°, most triangles will have good aspect ratios")
    print()


def example_complex_geometry():
    """Complex geometry with multiple holes"""
    print("="*60)
    print("Example 5: Complex Geometry (City Buildings)")
    print("="*60)

    # City block
    city = [(0, 0), (500, 0), (500, 500), (0, 500)]

    # Building footprints
    buildings = [
        [(100, 100), (150, 100), (150, 150), (100, 150)],
        [(200, 200), (250, 200), (250, 250), (200, 250)],
        [(300, 50), (350, 50), (350, 100), (300, 100)],
        [(50, 300), (100, 300), (100, 350), (50, 350)],
    ]

    result = pyspade.triangulate(
        outer=city,
        holes=buildings,
        max_edge_length=20.0,
        min_angle=20.0
    )

    print(f"City mesh (excluding buildings):")
    print(f"  Vertices: {len(result['vertices'])}")
    print(f"  Triangles: {len(result['triangles'])}")
    print(f"  Buildings excluded as holes: {len(buildings)}")
    print()


def example_triangulate_holes():
    """Mesh the holes instead of excluding them"""
    print("="*60)
    print("Example 6: Triangulating Inside Holes")
    print("="*60)

    outer = [(0, 0), (100, 0), (100, 100), (0, 100)]
    holes = [[(25, 25), (75, 25), (75, 75), (25, 75)]]

    # Exclude holes (default)
    result_excluded = pyspade.triangulate(
        outer, holes=holes, max_edge_length=10.0
    )

    # Include holes
    result_included = pyspade.triangulate(
        outer, holes=holes, max_edge_length=10.0,
        triangulate_holes=True
    )

    print(f"Holes excluded: {len(result_excluded['triangles'])} triangles")
    print(f"Holes included: {len(result_included['triangles'])} triangles")
    print()


if __name__ == "__main__":
    example_simple_square()
    example_with_hole()
    example_mesh_refinement()
    example_quality_mesh()
    example_complex_geometry()
    example_triangulate_holes()

    print("✅ All examples completed successfully!")
