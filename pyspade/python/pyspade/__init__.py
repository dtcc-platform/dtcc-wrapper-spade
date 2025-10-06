"""
pyspade - Fast 2D Delaunay triangulation for Python

This module provides Python bindings for the Spade library, offering high-performance
constrained Delaunay triangulation with mesh refinement capabilities.
"""

from .pyspade import triangulate

__version__ = "0.1.0"
__all__ = ["triangulate"]
