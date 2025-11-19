# geodist

**High-performance geometric distance algorithms with Rust and Python bindings.**

`geodist` provides fast, exact, and index-accelerated geometric distance
operations—starting with Hausdorff distance—implemented in Rust with optional
Python bindings via `pygeodist`.

The differentiator is *witness point* access: every distance result comes with the
specific control points that realized it, so you can audit and explain outcomes
instead of treating distances as opaque numbers, which is critical wherever
transparency matters (e.g., mapping QA, robotics safety checks, or scientific
reproducibility).

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

## Tooling

- Python uses [uv](https://docs.astral.sh/uv/). Install it via `curl -LsSf https://astral.sh/uv/install.sh | sh` or `brew install uv` on macOS, then provision a toolchain with `uv python install 3.13`.
- Set up the Python environment with `cd pygeodist && uv sync --all-extras` (or `make install` for the same effect).
- Common Python shortcuts from `pygeodist/Makefile`: `make lint`, `make test`, `make build`, `make clean`.
- Rust work happens under `geodist-rs`; use `cargo run` or `cargo test` there when kernels and tests are added.

## Validation

Validate the bootstrap Python binding end-to-end via `make install`, `make develop`, and `make test` in `pygeodist/`. Under the hood these run `uv sync --all-extras --dev`, `uv run maturin develop`, and `uv run pytest` (rerun `maturin develop` after Rust changes).
