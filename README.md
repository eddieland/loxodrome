# geodist

**High-performance geometric distance algorithms with Rust and Python bindings.**

`geodist` provides fast, exact, and index-accelerated geometric distance
operations—starting with Hausdorff distance—implemented in Rust with optional
Python bindings via `pygeodist`.

Key goals:

- Robust geometric distance kernels (point–point, point–segment, polygon edges, etc.)
- Full *witness point* reporting (the exact control points that determine the distance)
- R-tree and other spatial indexing accelerators for performance
- Clean Rust API (`geodist-rs`)
- Easy Python installation via prebuilt wheels (`pygeodist`)
- Eventually competitive with Shapely/GEOS performance

This project aims to become a flexible Rust geometry kernel for distance
computations, suitable for GIS, computer vision, robotics, and scientific
computing applications.

## Crates & Packages

| Component      | Purpose                               | Link            |
|----------------|---------------------------------------|-----------------|
| **geodist-rs** | Rust crate providing core algorithms  | *(coming soon)* |
| **pygeodist**  | Python bindings (PyO3 + maturin)      | *(coming soon)* |

## Features (current & roadmap)

- [ ] Hausdorff distance (directed + undirected)
- [ ] Witness point pairs for all metrics
- [ ] R-tree accelerated distance search
- [ ] LineString/Polygon sampling & densification
- [ ] Geometry type coverage (Point, LineString, Polygon, Multi*)
- [ ] Parallel computation (rayon)
- [ ] Frechét distance (future)
- [ ] Chamfer distance (future)

## Installation

### Rust

```bash
cargo add geodist-rs
```

### Python

```bash
pip install pygeodist
```

(Wheels for macOS, Linux, Windows planned.)

## Project Status

The project is in early active development. APIs may evolve until the initial
stable release. Contributions, suggestions, and issue reports are welcome.
