# loxodrome

**High-performance geometric distance algorithms with Rust kernels and Python bindings.**

`loxodrome` focuses on transparent distance calculations you can audit and explain
rather than treating results as opaque numbers. The Rust crate provides the core
geodesic kernels; Python bindings will layer ergonomic geometry wrappers once
the Rust surface settles.

## Distance types at a glance

- **Great-circle geodesic:** Shortest path on a sphere (default WGS84 mean
  radius). Use for global-scale routing and quick, explainable answers without
  projection quirks; expect ~0.3–0.5% error vs ellipsoidal results and prefer
  the ellipsoid variants when accuracy matters.
- **Ellipsoidal geodesic:** Same idea on a chosen ellipsoid when you need
  tighter agreement with geodesy references or region-specific spheroids.
- **ECEF chord (3D straight-line):** Altitude-aware “as-the-crow-flies” path
  through space using `Point3D` for pairwise distances or Hausdorff over 3D
  point sets.
- **Bearings (initial/final):** Direction of travel at the start and end of a
  geodesic. Handy for navigation cues, route snapping, or aligning segment
  splits downstream.
- **Hausdorff distance (directed and symmetric):** Maximal mismatch between two
  point sets. Directed asks “how far is A from being covered by B?”; symmetric
  asks “how far apart are these shapes overall?”
- **Bounding-box-clipped variants:** Restrict Hausdorff evaluation to a region
  to ignore distant outliers and focus on the area of interest.

Note on 3D: 3D helpers expose a straight-line chord in ECEF (altitude-aware,
“as-the-crow-flies” through space) via `Point3D` + `geodesic_distance_3d` and
Hausdorff variants on the WGS84 ellipsoid. Surface arc with altitude adjustment
can be added later if needed; clipping still uses latitude/longitude bounds.

## Crates & Packages

| Component      | What exists today                                   | Path |
|----------------|-----------------------------------------------------|------|
| **loxodrome-rs** | Rust crate with geodesic distance + Hausdorff APIs  | `loxodrome-rs/` |
| **loxodrome**  | Python package with PyO3 bindings (published to PyPI) | `loxodrome/` |
| **experiments** | Notebooks, benchmarks, and visualizations (not published) | `experiments/` |

## What works now

- Rust kernels expose validated geodesic primitives (`Point`, `Distance`, `Ellipsoid`, `BoundingBox`) with strict input checking.
- Great-circle distance on a spherical Earth (WGS84 mean radius by default) plus ellipsoidal inverse geodesics (WGS84 by default) using a GeographicLib-backed solver and custom radius/ellipsoid helpers.
- Batch distance calculation for many point pairs.
- Initial/final bearing output that reuses the distance kernel.
- Directed and symmetric Hausdorff distance over point sets with witness reporting (distance + realizing indices), with bounding-box-clipped variants and an automatic switch between an `rstar` index and an O(n*m) fallback for tiny inputs.
- 3D straight-line distances and Hausdorff evaluation over altitude-bearing point sets (with witnesses), reusing the same validation and clipping semantics (clip on lat/lon).
- Python bindings expose Rust-backed geometry handles along with `geodesic_distance`, `geodesic_with_bearings`, and Hausdorff helpers that return typed witness records. Imports stay guarded so Shapely interop remains optional.

## Roadmap highlights

- Witness point reporting for all metrics.
- Geometry coverage beyond point sets (LineString/Polygon sampling, densification).
- Parallel computation paths and richer distance metrics (Frechét, Chamfer).
- Full Python geometry wrappers and vectorized operations backed by the Rust kernels.

## Rust quickstart

```rust
use loxodrome_rs::{HausdorffDirectedWitness, Point, geodesic_distance, geodesic_with_bearings, hausdorff};

let origin = Point::new(40.7128, -74.0060)?;
let destination = Point::new(51.5074, -0.1278)?;

let meters = geodesic_distance(origin, destination)?.meters();
let bearings = geodesic_with_bearings(origin, destination)?;

let path_a = [origin, Point::new(40.0, -73.5)?];
let path_b = [destination, Point::new(51.0, -0.2)?];
let hausdorff_witness = hausdorff(&path_a, &path_b)?;
let hausdorff_meters = hausdorff_witness.distance().meters();
let HausdorffDirectedWitness { origin_index, candidate_index, .. } = hausdorff_witness.a_to_b();
```

While the API stabilizes, use the crate from this workspace or add it as a path
dependency.

## Python quickstart (smoke test)

The Python package includes the PyO3 extension stub and a small Typer CLI to
confirm the extension loads. Build the extension before exercising the Python
helpers to ensure the Rust kernels are available.

```bash
cd loxodrome
uv sync --all-extras --dev
uv run maturin develop  # builds the extension module
uv run lox info         # prints whether the extension loaded
uv run pytest           # exercises the stub surface
```

```python
from loxodrome import Point, geodesic_with_bearings, hausdorff

origin = Point(0.0, 0.0)
east = Point(0.0, 1.0)

result = geodesic_with_bearings(origin, east)
print(result.distance_m, result.initial_bearing_deg, result.final_bearing_deg)

witness = hausdorff([origin], [east])
print(witness.distance_m, witness.a_to_b.origin_index, witness.a_to_b.candidate_index)
```

## Python vectorized batches

Install the optional extra to enable NumPy-backed batch helpers:

```bash
pip install loxodrome[vectorized]
```

Key entry points live under `loxodrome.vectorized`:

- Constructors: `points_from_coords`, `points3d_from_coords`, `polylines_from_coords`, and `polygons_from_coords` accept NumPy arrays or Python buffers. Validation is vectorized and reports the failing index.
- Pairwise operations: `geodesic_distance_batch`, `geodesic_with_bearings_batch`, and `geodesic_distance_to_many` return NumPy arrays when available (lists otherwise).
- Polygon area: `area_batch` consumes flat coordinate buffers plus ring/polygon offsets.

Examples:

```python
import numpy as np
from loxodrome import vectorized as vz

origins = vz.points_from_coords(np.array([[0.0, 0.0], [0.0, 0.0]]))
destinations = vz.points_from_coords(np.array([[0.0, 1.0], [1.0, 0.0]]))

distances = vz.geodesic_distance_batch(origins, destinations)
print(distances.to_numpy())  # array([111195.08023353, 111195.08023353])

poly = vz.polygons_from_coords(
    coords=[[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
    ring_offsets=[0, 5],
    polygon_offsets=[0, 1],
)
print(vz.area_batch(poly).to_numpy())  # array([1.23087784e10])
```

See `experiments/src/experiments/bench_vectorized.py` for a quick throughput comparison between the vectorized and scalar paths:

```bash
cd experiments && uv sync --all-extras
uv run python -m experiments.bench_vectorized --count 100000
```

## Shapely interoperability

Shapely stays optional; install the extra when you need it:

```bash
pip install loxodrome[shapely]
```

Converters live in `loxodrome.ext.shapely` and keep imports guarded so you can
depend on `loxodrome` without pulling Shapely everywhere:

```python
from loxodrome import Point
from loxodrome.ext.shapely import from_shapely, to_shapely

point = Point(12.5, -45.0)
shapely_point = to_shapely(point)
round_tripped = from_shapely(shapely_point)
```

`Point`, `Point3D`, and `BoundingBox` are supported; other geometry kinds still raise
`TypeError`, and non-rectangular polygons raise `InvalidGeometryError`, until matching kernels land.

## Why PyO3 / Maturin?

- PyO3 exposes the Rust kernels directly to Python with predictable type conversions, Rust-side validation, and minimal glue code to keep the Python surface thin (vs CFFI/ctypes shims that tend to grow bespoke adapters).
- Maturin aligns the build with Cargo, producing Python wheels/SDists without custom setup.py plumbing and with sane multi-platform defaults compared to hand-rolled setuptools-rust configs.
- Error mapping, type checking, and memory safety all live in Rust, so the Python package is effectively a typed railing over the same kernel instead of a partial reimplementation that can drift.
- This pairing keeps Rust and Python artifacts in lockstep while fitting cleanly into uv/pip workflows, reducing packaging noise and keeping reviewable diffs focused on kernel changes.

## Python API scope and non-goals

- What ships today: `EARTH_RADIUS_METERS`, error types (`GeodistError`, `InvalidGeometryError`), Rust-backed `Ellipsoid`/`Point`/`BoundingBox`, spherical + ellipsoidal geodesic distance/bearings, and Hausdorff (directed and clipped) helpers. Everything routes directly to the Rust kernels—there is no pure-Python fallback.
- Explicit non-goals:
  - Full Shapely/GeoPandas parity or best-effort shims for arbitrary geometry tuples, GeoJSON-like dicts, or mixed dimensionality.
  - Implicit projections, datum shifts, or axis-order guessing; inputs are lat/lon degrees on WGS84 unless an explicit ellipsoid/radius is provided.
  - Silently coercing unsupported shapes (e.g., LineString/Polygon) or 3D-to-2D drops; these raise `InvalidGeometryError` instead of guessing.
  - Hidden fallback kernels that trade accuracy for convenience; failures should be loud so you know what the Rust core actually computed.
- Interop guidance: install the `shapely` extra when you need conversions; imports stay guarded and only 2D `Point` is supported today.
- Near-term additions (subject to Rust readiness): witness point reporting on all metrics, more geometry wrappers, and vectorized/batched helpers.
- The Typer CLI is a developer smoke test, not a user-facing entrypoint; keep automation and scripting pinned to the Python API instead.

## Releases

- Tag `vMAJOR.MINOR.PATCH` to trigger `.github/workflows/release.yml`; it defaults to dry-run and uploads crate and wheel artifacts for inspection.
- See `docs/publishing.md` for the release checklist, live-mode guards, and artifact locations.

## Project Status

The project is in early active development. APIs may evolve until the initial
stable release. Contributions, suggestions, and issue reports are welcome.

## Tooling

- Python uses [uv](https://docs.astral.sh/uv/). Install it via `curl -LsSf https://astral.sh/uv/install.sh | sh` or `brew install uv` on macOS, then provision a toolchain with `uv python install 3.13`.
- Set up the Python environment with `cd loxodrome && uv sync --all-extras` (or `make install` for the same effect). Run `uv run maturin develop` after Rust changes to rebuild the extension.
- Common Python shortcuts from `loxodrome/Makefile`: `make lint`, `make test`, `make build`, `make clean`.
- Rust work happens under `loxodrome-rs`; use `cargo fmt`, `cargo clippy`, and `cargo nextest run` (or the root `make fmt|lint|test`) while iterating on kernels.
