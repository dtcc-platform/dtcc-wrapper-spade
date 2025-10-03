#!/usr/bin/env python3
"""
Reference benchmark harness for 2D mesh generators.
"""

import argparse
import json
import time
import platform
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Optional
import importlib.util

try:
    import numpy as np
    import meshio
except ImportError:
    print("ERROR: Required packages not installed. Run: pip install meshio numpy")
    sys.exit(1)


def load_testcase(filepath: str) -> Tuple[List[Tuple[float, float]], List[List[Tuple[float, float]]]]:
    """Load polygon data from testcase file."""
    with open(filepath, 'r') as f:
        lines = f.readlines()

    def parse_loop(line: str) -> List[Tuple[float, float]]:
        coords = list(map(float, line.strip().split()))
        return [(coords[i], coords[i+1]) for i in range(0, len(coords), 2)]

    outer = parse_loop(lines[0])
    inner_loops = [parse_loop(line) for line in lines[1:] if line.strip()]

    return outer, inner_loops


def get_system_meta() -> dict:
    """Gather system metadata."""
    meta = {
        'os': platform.system(),
        'os_version': platform.version(),
        'platform': platform.platform(),
        'machine': platform.machine(),
        'processor': platform.processor(),
        'python_version': platform.python_version(),
    }

    # Try to get compiler version
    try:
        result = subprocess.run(['rustc', '--version'], capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            meta['rust_version'] = result.stdout.strip()
    except:
        pass

    try:
        result = subprocess.run(['cargo', '--version'], capture_output=True, text=True, timeout=5)
        if result.returncode == 0:
            meta['cargo_version'] = result.stdout.strip()
    except:
        pass

    return meta


def compute_mesh_quality(points: List[Tuple[float, float, float]],
                         triangles: List[Tuple[int, int, int]]) -> dict:
    """Compute mesh quality metrics."""
    points_np = np.array(points)
    triangles_np = np.array(triangles)

    min_angles = []
    aspect_ratios = []
    areas = []

    for tri in triangles_np:
        # Get triangle vertices
        p0, p1, p2 = points_np[tri[0]], points_np[tri[1]], points_np[tri[2]]

        # Compute edge vectors
        e0 = p1 - p0
        e1 = p2 - p1
        e2 = p0 - p2

        # Edge lengths
        l0 = np.linalg.norm(e0)
        l1 = np.linalg.norm(e1)
        l2 = np.linalg.norm(e2)

        # Triangle area (2D cross product)
        area = 0.5 * abs(e0[0] * (-e2[1]) - e0[1] * (-e2[0]))
        areas.append(area)

        # Angles using law of cosines
        if l0 > 0 and l1 > 0 and l2 > 0:
            angle0 = np.arccos(np.clip((l0**2 + l2**2 - l1**2) / (2 * l0 * l2), -1, 1))
            angle1 = np.arccos(np.clip((l0**2 + l1**2 - l2**2) / (2 * l0 * l1), -1, 1))
            angle2 = np.arccos(np.clip((l1**2 + l2**2 - l0**2) / (2 * l1 * l2), -1, 1))

            min_angle_rad = min(angle0, angle1, angle2)
            min_angles.append(np.degrees(min_angle_rad))

            # Aspect ratio: ratio of longest edge to shortest altitude
            # altitude = 2 * area / base
            min_altitude = 2 * area / max(l0, l1, l2)
            max_edge = max(l0, l1, l2)
            aspect_ratio = max_edge / min_altitude if min_altitude > 0 else float('inf')
            aspect_ratios.append(aspect_ratio)

    # Compute distributions
    min_angles_np = np.array(min_angles)
    aspect_ratios_np = np.array(aspect_ratios)
    areas_np = np.array(areas)

    # Angle distribution bins
    angle_bins = [0, 10, 20, 30, 40, 50, 60, 90]
    angle_hist, _ = np.histogram(min_angles_np, bins=angle_bins)

    # Aspect ratio distribution bins
    ar_bins = [1, 2, 5, 10, 20, 50, 100, np.inf]
    ar_hist, _ = np.histogram(aspect_ratios_np, bins=ar_bins)

    return {
        'num_triangles': len(triangles),
        'total_area': float(np.sum(areas_np)),
        'min_angle': {
            'min': float(np.min(min_angles_np)) if len(min_angles_np) > 0 else 0,
            'max': float(np.max(min_angles_np)) if len(min_angles_np) > 0 else 0,
            'mean': float(np.mean(min_angles_np)) if len(min_angles_np) > 0 else 0,
            'median': float(np.median(min_angles_np)) if len(min_angles_np) > 0 else 0,
            'distribution': {
                f'{angle_bins[i]}-{angle_bins[i+1]}°': int(angle_hist[i])
                for i in range(len(angle_hist))
            }
        },
        'aspect_ratio': {
            'min': float(np.min(aspect_ratios_np)) if len(aspect_ratios_np) > 0 else 0,
            'max': float(np.max(aspect_ratios_np)) if len(aspect_ratios_np) > 0 else 0,
            'mean': float(np.mean(aspect_ratios_np)) if len(aspect_ratios_np) > 0 else 0,
            'median': float(np.median(aspect_ratios_np)) if len(aspect_ratios_np) > 0 else 0,
            'distribution': {
                f'{ar_bins[i]}-{ar_bins[i+1]}': int(ar_hist[i])
                for i in range(len(ar_hist))
            }
        },
        'area': {
            'min': float(np.min(areas_np)) if len(areas_np) > 0 else 0,
            'max': float(np.max(areas_np)) if len(areas_np) > 0 else 0,
            'mean': float(np.mean(areas_np)) if len(areas_np) > 0 else 0,
            'median': float(np.median(areas_np)) if len(areas_np) > 0 else 0,
        }
    }


def write_vtu(filepath: str, points: List[Tuple[float, float, float]],
              triangles: List[Tuple[int, int, int]],
              lines: Optional[List[Tuple[int, int]]] = None):
    """Write mesh to VTU format using meshio."""
    points_array = np.array(points, dtype=float)

    cells = [("triangle", np.array(triangles, dtype=int))]
    if lines:
        cells.append(("line", np.array(lines, dtype=int)))

    mesh = meshio.Mesh(points=points_array, cells=cells)
    meshio.write(filepath, mesh)


def run_benchmark(adapter_module, outer, inner_loops, maxh, quality, enforce_constraints, repeats=3):
    """Run triangulation benchmark with timing."""
    best_time = float('inf')
    best_result = None

    for _ in range(repeats):
        start = time.perf_counter()
        result = adapter_module.triangulate(
            outer=outer,
            inner_loops=inner_loops,
            maxh=maxh,
            quality=quality,
            enforce_constraints=enforce_constraints
        )
        elapsed = time.perf_counter() - start

        if elapsed < best_time:
            best_time = elapsed
            best_result = result

    points, triangles, lines = best_result
    return points, triangles, lines, best_time


def main():
    parser = argparse.ArgumentParser(description='Benchmark harness for 2D mesh generators')
    parser.add_argument('--software', required=True, help='Software name (e.g., "Spade")')
    parser.add_argument('--adapter', required=True, help='Path to adapter module (e.g., adapter_spade.py)')
    parser.add_argument('--testcase', required=True, help='Path to testcase file')
    parser.add_argument('--outdir', required=True, help='Output directory')
    parser.add_argument('--sizes', nargs='+', type=float, default=[100, 50, 20, 10, 5, 2, 1],
                       help='Size parameters for sweep')
    parser.add_argument('--repeats', type=int, default=3, help='Number of timing repeats')

    args = parser.parse_args()

    # Load adapter module
    spec = importlib.util.spec_from_file_location("adapter", args.adapter)
    adapter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(adapter)

    # Create output directory
    outdir = Path(args.outdir)
    outdir.mkdir(exist_ok=True)

    # Write metadata
    meta = get_system_meta()
    meta['software'] = args.software
    with open(outdir / f'meta_{args.software}.json', 'w') as f:
        json.dump(meta, f, indent=2)

    # Load testcase
    outer, inner_loops = load_testcase(args.testcase)

    # Results storage
    results = []
    quality_metrics = []

    print(f"Running benchmarks for {args.software}...")

    # Test A: Unit square, defaults
    print("Test A: Unit square (default)")
    unit_square = [(0, 0), (1, 0), (1, 1), (0, 1)]
    points, triangles, lines, t = run_benchmark(
        adapter, unit_square, [], None, "default", False, args.repeats
    )
    write_vtu(str(outdir / 'A_unit_square_default.vtu'), points, triangles, lines)
    quality = compute_mesh_quality(points, triangles)
    quality['test'] = 'A'
    quality['description'] = 'unit_square_default'
    quality_metrics.append(quality)
    results.append({
        'test': 'A',
        'description': 'unit_square_default',
        'num_triangles': len(triangles),
        'time_sec': t,
        'triangles_per_sec': len(triangles) / t if t > 0 else 0
    })

    # Test B: Unit square + inner polygon
    print("Test B: Unit square with inner polygon")
    inner_poly = [(0.3, 0.3), (0.7, 0.3), (0.7, 0.7), (0.3, 0.7)]
    points, triangles, lines, t = run_benchmark(
        adapter, unit_square, [inner_poly], None, "default", True, args.repeats
    )
    write_vtu(str(outdir / 'B_unit_square_with_inner_polygon.vtu'), points, triangles, lines)
    quality = compute_mesh_quality(points, triangles)
    quality['test'] = 'B'
    quality['description'] = 'unit_square_with_inner'
    quality_metrics.append(quality)
    results.append({
        'test': 'B',
        'description': 'unit_square_with_inner',
        'num_triangles': len(triangles),
        'time_sec': t,
        'triangles_per_sec': len(triangles) / t if t > 0 else 0
    })

    # Test C: City testcase
    print("Test C: City testcase (maxh=100)")
    points, triangles, lines, t = run_benchmark(
        adapter, outer, inner_loops, 100.0, "moderate", True, args.repeats
    )
    write_vtu(str(outdir / 'C_city_100.vtu'), points, triangles, lines)
    quality = compute_mesh_quality(points, triangles)
    quality['test'] = 'C'
    quality['description'] = 'city_maxh_100'
    quality_metrics.append(quality)
    results.append({
        'test': 'C',
        'description': 'city_maxh_100',
        'num_triangles': len(triangles),
        'time_sec': t,
        'triangles_per_sec': len(triangles) / t if t > 0 else 0
    })

    # Test D: Size sweep
    print("Test D: Size sweep")
    for size in args.sizes:
        print(f"  Size: {size}")
        points, triangles, lines, t = run_benchmark(
            adapter, outer, inner_loops, size, "moderate", True, args.repeats
        )
        write_vtu(str(outdir / f'D_city_{size}.vtu'), points, triangles, lines)
        quality = compute_mesh_quality(points, triangles)
        quality['test'] = 'D'
        quality['description'] = f'city_maxh_{size}'
        quality['maxh'] = size
        quality_metrics.append(quality)
        results.append({
            'test': 'D',
            'description': f'city_maxh_{size}',
            'maxh': size,
            'num_triangles': len(triangles),
            'time_sec': t,
            'triangles_per_sec': len(triangles) / t if t > 0 else 0
        })

    # Write results
    with open(outdir / f'bench_{args.software}.json', 'w') as f:
        json.dump(results, f, indent=2)

    # Write CSV
    csv_path = outdir / f'bench_{args.software}.csv'
    with open(csv_path, 'w') as f:
        f.write('test,description,maxh,num_triangles,time_sec,triangles_per_sec\n')
        for r in results:
            maxh = r.get('maxh', '')
            f.write(f"{r['test']},{r['description']},{maxh},{r['num_triangles']},"
                   f"{r['time_sec']:.6f},{r['triangles_per_sec']:.2f}\n")

    # Write quality metrics to log file
    metrics_path = outdir / f'metrics_{args.software}.log'
    with open(metrics_path, 'w') as f:
        f.write(f"Mesh Quality Metrics for {args.software}\n")
        f.write("=" * 80 + "\n\n")

        for metric in quality_metrics:
            f.write(f"Test: {metric['test']} - {metric['description']}\n")
            if 'maxh' in metric:
                f.write(f"  maxh: {metric['maxh']}\n")
            f.write(f"  Triangle Count: {metric['num_triangles']}\n")
            f.write(f"  Total Surface Area: {metric['total_area']:.2f}\n")
            f.write("\n")

            f.write("  Minimum Angle Statistics:\n")
            f.write(f"    Min:    {metric['min_angle']['min']:.2f}°\n")
            f.write(f"    Max:    {metric['min_angle']['max']:.2f}°\n")
            f.write(f"    Mean:   {metric['min_angle']['mean']:.2f}°\n")
            f.write(f"    Median: {metric['min_angle']['median']:.2f}°\n")
            f.write("    Distribution:\n")
            for bin_range, count in metric['min_angle']['distribution'].items():
                pct = 100 * count / metric['num_triangles'] if metric['num_triangles'] > 0 else 0
                f.write(f"      {bin_range:15s} {count:6d} ({pct:5.1f}%)\n")
            f.write("\n")

            f.write("  Aspect Ratio Statistics:\n")
            f.write(f"    Min:    {metric['aspect_ratio']['min']:.2f}\n")
            f.write(f"    Max:    {metric['aspect_ratio']['max']:.2f}\n")
            f.write(f"    Mean:   {metric['aspect_ratio']['mean']:.2f}\n")
            f.write(f"    Median: {metric['aspect_ratio']['median']:.2f}\n")
            f.write("    Distribution:\n")
            for bin_range, count in metric['aspect_ratio']['distribution'].items():
                pct = 100 * count / metric['num_triangles'] if metric['num_triangles'] > 0 else 0
                f.write(f"      {bin_range:15s} {count:6d} ({pct:5.1f}%)\n")
            f.write("\n")

            f.write("  Triangle Area Statistics:\n")
            f.write(f"    Min:    {metric['area']['min']:.4f}\n")
            f.write(f"    Max:    {metric['area']['max']:.4f}\n")
            f.write(f"    Mean:   {metric['area']['mean']:.4f}\n")
            f.write(f"    Median: {metric['area']['median']:.4f}\n")
            f.write("\n" + "-" * 80 + "\n\n")

    print(f"\nBenchmark complete! Results in {outdir}")
    print(f"Generated {len(results)} test results")
    print(f"Quality metrics saved to {metrics_path}")


if __name__ == '__main__':
    main()
