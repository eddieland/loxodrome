# Python Vectorization for Geometry Batches

## Purpose

- Enable Python users to construct geodist objects (points, polylines, polygons) in bulk and run distance/area/heading computations over large collections with minimal Python overhead.
- Provide ergonomic, well-typed APIs that interoperate with common data containers (NumPy and Python buffers) while preserving correctness constraints on lat/lon units and order.

## Guiding Constraints

- Keep imports lightweight: vectorized paths rely on optional extras (NumPy) and degrade to list/tuple inputs without pulling heavy deps by default.
- Preserve invariants: lat_deg in [-90, 90], lon_deg in [-180, 180]; avoid silent coordinate reordering and validate per-element failures with clear error reporting.
- Favor zero-copy where possible (memoryview/ndarray buffers) and minimize allocations; avoid extra knobs until profiling shows need.
- API parity with existing scalar constructors and operations; avoid shadow APIs that drift from Rust-backed behavior and keep `_geodist_rs.pyi` in sync.
- Release the GIL for Rust-backed kernels and keep interfaces deterministic (no implicit multithreading unless opt-in).

## Target Capabilities

1. Bulk constructors for core geometry types accepting NumPy buffers or Python list/tuple inputs with fast-path validation.
2. Vectorized computation APIs (distances, bearings, areas) over homogeneous collections returning NumPy arrays aligned with inputs; lists as a compatibility fallback.

## Chosen API Surface (Python)

- **Module + extra:** Expose `geodist.vectorized` behind optional extra `geodist[vectorized]` installing `numpy>=1.24`. Imports stay lazy; pure-Python buffers continue to work when NumPy is absent.
- **Input containers:** Accept `array_like` lat/lon as two 1-D buffers or a 2-D `(..., 2)` buffer; accept `float32` and `float64` with native endianness. Memoryviews over contiguous data follow the ndarray fast path.
- **Validation contract:** Support `errors="raise"` only for v1; bounds + finiteness checks raise `InvalidGeometryError` with the offending index and a short reason.
- **Outputs:** Return NumPy arrays for numeric results; fall back to Python lists when NumPy is unavailable. No Arrow/pandas interop in v1.
- **Batch types:** Constructors return lightweight handles that keep NumPy arrays (or lists) and adapters:
  - `PointBatch` exposing `.lat_deg`, `.lon_deg` and converters `.to_numpy() -> np.ndarray[float64, (N, 2)]`, `.to_python() -> list[tuple[float, float]]`.
  - `Point3DBatch` adds `.altitude_m` and returns `(N, 3)` layouts.
  - `PolylineBatch`/`PolygonBatch` store flat coordinate arrays plus `offsets` (`np.ndarray[int64]`). Offsets must start at 0 and be monotonically increasing; empty geometries allowed.
- **Constructors (in `geodist.vectorized`):**
  - `points_from_coords(lat_deg, lon_deg, *, errors="raise") -> PointBatch`
  - `points3d_from_coords(lat_deg, lon_deg, altitude_m, *, errors="raise") -> Point3DBatch`
  - `polylines_from_coords(coords, offsets, *, errors="raise") -> PolylineBatch` where `coords` is `(N, 2|3)` and `offsets` length = `num_polylines + 1`.
  - `polygons_from_coords(coords, ring_offsets, polygon_offsets, *, errors="raise") -> PolygonBatch` mirroring Arrow geometry layout without Arrow dependencies.
- **Vectorized operations (all in `geodist.vectorized`):**
  - `geodesic_distance_batch(origins: PointBatch | ArrayLike, destinations: PointBatch | ArrayLike, *, ellipsoid=None, errors="raise") -> DistanceResult`
  - `geodesic_with_bearings_batch(origins, destinations, *, ellipsoid=None, errors="raise") -> BearingsResult`
  - `geodesic_distance_to_many(origin: Point | tuple[float, float], destinations: PointBatch | ArrayLike, *, ellipsoid=None, errors="raise") -> DistanceResult`
  - `area_batch(polygons: PolygonBatch, *, ellipsoid=None, errors="raise") -> AreaResult`
- **Result containers:** Functions return dataclasses with array fields:
  - `DistanceResult`: `.distance_m`
  - `BearingsResult`: `.distance_m`, `.initial_bearing_deg`, `.final_bearing_deg`
  - `AreaResult`: `.area_m2`
  Each result container exposes `.to_numpy()` and `.to_python()` matching batch conversion semantics.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Decide API surface for vectorized constructors and outputs | Docstring-level spec for functions/classes (names, args, return types, error handling, optional extras) reviewed and aligned with scalar APIs | This document is source of truth | ✅ |
| P0 | Implement bulk point/polyline constructors over NumPy/list inputs | Rust/PyO3 entry points accept memoryview/ndarray; validation fast path with lat/lon bounds; `_geodist_rs.pyi` updated | Add unit/regression tests covering mixed valid/invalid rows | 📝 |
| P0 | Add vectorized distance/bearing/area kernels for homogeneous batches | Support pairwise and fixed-to-many variants; return NumPy arrays (lists fallback); deterministic errors | Include GIL release and benchmarks for baseline throughput | 📝 |
| P1 | Documentation and examples | Add user-facing guide in `pygeodist` docs with realistic examples and performance notes; update README if API is public | Include guidance on coordinate units/order and optional dependencies | 📝 |
| P2 | Performance profiling and tuning | Benchmarks comparing scalar vs vectorized paths; identify bottlenecks and add micro-optimizations or chunk defaults | Could live under `pygeodist/devtools` and inform future tuning | 📝 |
| P3 | Optional Arrow/pandas interop | Evaluate zero-copy adapters once core kernels stabilize | Only proceed if requested by workloads | 📝 |
| P3 | Optional multicore/streaming execution | Evaluate opt-in multithreading/streaming for extremely large batches without breaking determinism | Only proceed if P0/P1 performance goals unmet | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Silent data corruption from coordinate ordering or invalid values. **Mitigation:** Enforce shape/dtype checks and bounds validation; fail fast with `InvalidGeometryError`.
- **Risk:** Memory pressure on huge inputs. **Mitigation:** Document expected memory footprints; add chunking only if profiling shows need.
- **Risk:** Divergence between Rust kernels and Python stubs. **Mitigation:** Update `_geodist_rs.pyi` alongside Rust changes; add cross-language parity tests.
- **Risk:** Future interop scope creep. **Mitigation:** Keep v1 to NumPy/lists; defer Arrow/pandas until requested and benchmarked.

### Open Questions

- Should we expose shapely/GeoPandas helpers, or keep table interop limited to Arrow/pandas to reduce dependencies?
- Do we want streaming iterators for outputs (generator of chunks) in addition to assembled results for memory-constrained users?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Decide API surface for vectorized constructors and outputs._
- **Next up:** _Implement bulk point/polyline constructors over NumPy/list inputs._

## Lessons Learned (ongoing)

- Keeping the v1 surface NumPy-first with list fallbacks improves deliverability and keeps validation paths simple.
