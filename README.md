# geodist

**High-performance geometric distance algorithms with Rust kernels and Python bindings.**

`geodist` focuses on transparent distance calculations you can audit and explain
rather than treating results as opaque numbers. The Rust crate provides the core
geodesic kernels; Python bindings will layer ergonomic geometry wrappers once
the Rust surface settles.

## Distance types at a glance

- **Great-circle geodesic:** Shortest path on a sphere (default WGS84 mean
  radius). Use for global-scale routing and quick, explainable answers without
  projection quirks.
- **Ellipsoidal geodesic:** Same idea on a chosen ellipsoid when you need
  tighter agreement with geodesy references or region-specific spheroids.
- **Bearings (initial/final):** Direction of travel at the start and end of a
  geodesic. Handy for navigation cues, route snapping, or aligning segment
  splits downstream.
- **Hausdorff distance (directed and symmetric):** Maximal mismatch between two
  point sets. Directed asks “how far is A from being covered by B?”; symmetric
  asks “how far apart are these shapes overall?”
- **Bounding-box-clipped variants:** Restrict Hausdorff evaluation to a region
  to ignore distant outliers and focus on the area of interest.

## Crates & Packages

| Component      | What exists today                                   | Path |
|----------------|-----------------------------------------------------|------|
| **geodist-rs** | Rust crate with geodesic distance + Hausdorff APIs  | `geodist-rs/` |
| **pygeodist**  | Python package with a PyO3 extension smoke test     | `pygeodist/` |

## What works now

- Rust kernels expose validated geodesic primitives (`Point`, `Distance`, `Ellipsoid`, `BoundingBox`) with strict input checking.
- Great-circle distance on a spherical Earth (WGS84 mean radius by default) plus custom radius/ellipsoid helpers.
- Batch distance calculation for many point pairs.
- Initial/final bearing output that reuses the distance kernel.
- Directed and symmetric Hausdorff distance over point sets, with bounding-box-clipped variants and an automatic switch between an `rstar` index and an O(n*m) fallback for tiny inputs.
- Python bindings currently re-export the compiled `EARTH_RADIUS_METERS` constant and error types only; geometry wrappers will map the Rust structs once the surface settles.

## Roadmap highlights

- Witness point reporting for all metrics.
- Geometry coverage beyond point sets (LineString/Polygon sampling, densification).
- Parallel computation paths and richer distance metrics (Frechét, Chamfer).
- Full Python geometry wrappers and vectorized operations backed by the Rust kernels.

## Rust quickstart

```rust
use geodist_rs::{Point, geodesic_distance, geodesic_with_bearings, hausdorff};

let origin = Point::new(40.7128, -74.0060)?;
let destination = Point::new(51.5074, -0.1278)?;

let meters = geodesic_distance(origin, destination)?.meters();
let bearings = geodesic_with_bearings(origin, destination)?;

let path_a = [origin, Point::new(40.0, -73.5)?];
let path_b = [destination, Point::new(51.0, -0.2)?];
let hausdorff_meters = hausdorff(&path_a, &path_b)?.meters();
```

While the API stabilizes, use the crate from this workspace or add it as a path
dependency.

## Python quickstart (smoke test)

The Python package includes the PyO3 extension stub and a small Typer CLI to
confirm the extension loads. Kernels are not wired into the Python wrapper yet,
and the public API is intentionally tiny until Rust-backed geometry wrappers are
ready.

```bash
cd pygeodist
uv sync --all-extras --dev
uv run maturin develop  # builds the extension module
uv run geodist info     # prints whether the extension loaded
uv run pytest           # exercises the stub surface
```

## Why PyO3 / Maturin?

- PyO3 exposes the Rust kernels directly to Python with predictable type conversions, Rust-side validation, and minimal glue code to keep the Python surface thin (vs CFFI/ctypes shims that tend to grow bespoke adapters).
- Maturin aligns the build with Cargo, producing Python wheels/SDists without custom setup.py plumbing and with sane multi-platform defaults compared to hand-rolled setuptools-rust configs.
- Error mapping, type checking, and memory safety all live in Rust, so the Python package is effectively a typed railing over the same kernel instead of a partial reimplementation that can drift.
- This pairing keeps Rust and Python artifacts in lockstep while fitting cleanly into uv/pip workflows, reducing packaging noise and keeping reviewable diffs focused on kernel changes.

## Python API scope and non-goals

- Public exports today: `EARTH_RADIUS_METERS` and error types (`GeodistError` and the derived error classes).
- Non-goals: mirroring Shapely parity, accepting arbitrary geometry tuples, or silently coercing unsupported geometry kinds.
- Interop guidance: install the `shapely` extra when needed; conversions are explicit, guard imports, and currently error for any geometry beyond `Point` instead of guessing.
- Future Python surface (no promised dates): wrappers around the Rust kernels for geodesic distance/bearings, Hausdorff (including bounding-box-clipped variants), and `LineString`/`Polygon` once Rust exposes them (gated on Rust readiness to avoid drift).
- The Typer CLI is for local development only and should not be treated as a user-facing entrypoint.

## Project Status

The project is in early active development. APIs may evolve until the initial
stable release. Contributions, suggestions, and issue reports are welcome.

## Tooling

- Python uses [uv](https://docs.astral.sh/uv/). Install it via `curl -LsSf https://astral.sh/uv/install.sh | sh` or `brew install uv` on macOS, then provision a toolchain with `uv python install 3.13`.
- Set up the Python environment with `cd pygeodist && uv sync --all-extras` (or `make install` for the same effect). Run `uv run maturin develop` after Rust changes to rebuild the extension.
- Common Python shortcuts from `pygeodist/Makefile`: `make lint`, `make test`, `make build`, `make clean`.
- Rust work happens under `geodist-rs`; use `cargo fmt`, `cargo clippy`, and `cargo nextest run` (or the root `make fmt|lint|test`) while iterating on kernels.
