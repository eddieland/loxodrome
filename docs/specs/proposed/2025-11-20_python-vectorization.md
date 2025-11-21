# Python Vectorization for Geometry Batches

## Purpose

- Enable Python users to construct geodist objects (points, polylines, polygons) in bulk and run distance/area/heading computations over large collections with minimal Python overhead.
- Provide ergonomic, well-typed APIs that interoperate with common data containers (NumPy, memoryviews, Arrow arrays) while preserving correctness constraints on lat/lon units and order.

## Guiding Constraints

- Keep imports lightweight: vectorized paths rely on optional extras (e.g., NumPy/pyarrow) and degrade to list/tuple inputs without pulling heavy deps by default.
- Preserve invariants: lat_deg in [-90, 90], lon_deg in [-180, 180]; avoid silent coordinate reordering and validate per-element failures with clear error reporting or masks.
- Favor zero-copy where possible (memoryview/ndarray buffers) and minimize allocations; expose chunking knobs for very large inputs to bound memory.
- API parity with existing scalar constructors and operations; avoid shadow APIs that drift from Rust-backed behavior and keep `_geodist_rs.pyi` in sync.
- Release the GIL for Rust-backed kernels and keep interfaces deterministic (no implicit multithreading unless opt-in).

## Target Capabilities

1. Bulk constructors for core geometry types accepting contiguous buffer inputs (NumPy arrays, array-like lat/lon pairs, Arrow arrays when available) with fast-path validation.
2. Vectorized computation APIs (e.g., distances, bearings, areas) that operate on homogeneous collections and return array outputs aligned with inputs, with optional mask/error reporting.
3. Interop helpers to ingest/export tabular data (pandas/Arrow) without extra copies when shapes and dtypes permit.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Decide API surface for vectorized constructors and outputs | Docstring-level spec for functions/classes (names, args, return types, error handling, optional extras) reviewed and aligned with scalar APIs | Include decisions on mask semantics vs exceptions and table interop naming | 📝 |
| P0 | Implement bulk point/polyline constructors over buffer inputs | Rust/PyO3 entry points accept memoryview/ndarray; validation fast path with lat/lon bounds; optional chunking for large arrays; `_geodist_rs.pyi` updated | Add unit/regression tests covering mixed valid/invalid rows and fallback paths | 📝 |
| P0 | Add vectorized distance/bearing/area kernels for homogeneous batches | Support pairwise and fixed-to-many variants; return ndarray/pyarrow or lists as negotiated; errors surfaced deterministically | Include GIL release and benchmarks for baseline throughput | 📝 |
| P1 | Provide tabular interop adapters (pandas/Arrow) | Helpers to build geometries from DataFrame/Arrow columns with zero-copy where feasible; documented behavior for missing/invalid values | Integration tests across pandas/pyarrow versions in CI matrix | 📝 |
| P1 | Documentation and examples | Add user-facing guide in `pygeodist` docs with realistic examples and performance notes; update README if API is public | Include guidance on coordinate units/order and optional dependencies | 📝 |
| P2 | Performance profiling and tuning | Benchmarks comparing scalar vs vectorized paths; identify bottlenecks and add micro-optimizations or chunk defaults | Could live under `pygeodist/devtools` and inform future tuning | 📝 |
| P3 | Optional multicore/streaming execution | Evaluate opt-in multithreading/streaming for extremely large batches without breaking determinism | Only proceed if P0/P1 performance goals unmet | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Heavy optional deps increase install friction. **Mitigation:** Keep NumPy/pyarrow optional extras; maintain pure-Python fallback with reasonable performance for small batches.
- **Risk:** Silent data corruption from coordinate ordering or invalid values. **Mitigation:** Enforce shape/dtype checks, bounds validation, and explicit mask/exception choices documented in API.
- **Risk:** Memory pressure on huge inputs. **Mitigation:** Provide chunked processing interfaces and document memory expectations; test with large synthetic datasets.
- **Risk:** Divergence between Rust kernels and Python stubs. **Mitigation:** Update `_geodist_rs.pyi` alongside Rust changes; add cross-language parity tests.

### Open Questions

- Should outputs default to NumPy arrays, Arrow arrays, or Python lists when no optional deps are installed?
- How should invalid rows be reported: boolean mask, exceptions with offending indices, or filtered outputs?
- Do we support heterogeneous geometry batches (e.g., mixed points/polylines) or only homogeneous collections?
- What minimum versions of NumPy/pandas/pyarrow do we target for compatibility and wheels?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _None yet — planned._
- **Next up:** _Decide API surface for vectorized constructors and outputs._

## Lessons Learned (ongoing)

- _TBD._
# Python Vectorization for Geometry Batches

## Purpose

- Enable Python users to construct geodist objects (points, polylines, polygons) in bulk and run distance/area/heading computations over large collections with minimal Python overhead.
- Provide ergonomic, well-typed APIs that interoperate with common data containers (NumPy, memoryviews, Arrow arrays) while preserving correctness constraints on lat/lon units and order.

## Guiding Constraints

- Keep imports lightweight: vectorized paths rely on optional extras (e.g., NumPy/pyarrow) and degrade to list/tuple inputs without pulling heavy deps by default.
- Preserve invariants: lat_deg in [-90, 90], lon_deg in [-180, 180]; avoid silent coordinate reordering and validate per-element failures with clear error reporting or masks.
- Favor zero-copy where possible (memoryview/ndarray buffers) and minimize allocations; expose chunking knobs for very large inputs to bound memory.
- API parity with existing scalar constructors and operations; avoid shadow APIs that drift from Rust-backed behavior and keep `_geodist_rs.pyi` in sync.
- Release the GIL for Rust-backed kernels and keep interfaces deterministic (no implicit multithreading unless opt-in).

## Target Capabilities

1. Bulk constructors for core geometry types accepting contiguous buffer inputs (NumPy arrays, array-like lat/lon pairs, Arrow arrays when available) with fast-path validation.
2. Vectorized computation APIs (e.g., distances, bearings, areas) that operate on homogeneous collections and return array outputs aligned with inputs, with optional mask/error reporting.
3. Interop helpers to ingest/export tabular data (pandas/Arrow) without extra copies when shapes and dtypes permit.

## Chosen API Surface (Python)

- **Module + extra:** Expose `geodist.vectorized` behind optional extra `geodist[vectorized]` installing `numpy>=1.24` and `pyarrow>=14`. Imports stay lazy; pure-Python buffers continue to work when extras are absent. No pandas dependency; DataFrame support is best-effort when installed.
- **Input containers:** Accept `array_like` lat/lon as two 1-D buffers or a 2-D `(..., 2)` buffer; accept `float32` and `float64` with native endianness. `pyarrow.StructArray`/`RecordBatch` with `lat_deg`/`lon_deg` fields is supported zero-copy. Memoryviews over contiguous data follow the ndarray fast path.
- **Validation contract:** Default `errors="raise"` performs bounds + finiteness checks and raises `InvalidGeometryError` with the offending index and a short reason. `errors="mask"` returns a boolean mask aligned with inputs and fills invalid outputs with `nan`; no partial exceptions. `errors="ignore"` skips per-element validation (shape/dtype checks still enforced) for trusted data.
- **Output negotiation:** `output` accepts `auto` (mirror primary input, preferring Arrow > NumPy > Python), `numpy`, `arrow`, or `python`. If the requested backend is unavailable, raise `ImportError` rather than silently changing formats.
- **Batch types:** Constructors return lightweight handles:
  - `PointBatch` exposing `.lat_deg`, `.lon_deg`, optional `.mask`, and converters `.to_numpy() -> np.ndarray[float64, (N, 2)]`, `.to_arrow() -> StructArray`, `.to_python() -> list[tuple[float, float]]`.
  - `Point3DBatch` adds `.altitude_m` and returns `(N, 3)` layouts.
  - `PolylineBatch`/`PolygonBatch` store flat coordinate arrays plus `offsets` (`np.ndarray[int64]` or Arrow `ListArray`). Offsets must start at 0 and be monotonically increasing; empty geometries allowed.
- **Constructors (in `geodist.vectorized`):**
  - `points_from_coords(lat_deg, lon_deg, *, errors="raise", output="auto", chunk_size: int | None = None) -> PointBatch`
  - `points3d_from_coords(lat_deg, lon_deg, altitude_m, *, errors="raise", output="auto", chunk_size=None) -> Point3DBatch`
  - `polylines_from_coords(coords, offsets, *, errors="raise", output="auto") -> PolylineBatch` where `coords` is `(N, 2|3)` and `offsets` length = `num_polylines + 1`.
  - `polygons_from_coords(coords, ring_offsets, polygon_offsets, *, errors="raise", output="auto") -> PolygonBatch` mirroring Arrow geometry layout (rings within polygons).
  - `points_from_table(table, lat_column="lat_deg", lon_column="lon_deg", altitude_column=None, *, errors="raise", output="auto")` for pandas DataFrames or Arrow tables; zero-copy when dtypes align, else copy with validation.
- **Vectorized operations (all in `geodist.vectorized`):**
  - `geodesic_distance_batch(origins: PointBatch | ArrayLike, destinations: PointBatch | ArrayLike, *, ellipsoid=None, errors="raise", output="auto", release_gil=True) -> DistanceResult`
  - `geodesic_with_bearings_batch(origins, destinations, *, ellipsoid=None, errors="raise", output="auto", release_gil=True) -> BearingsResult`
  - `geodesic_distance_to_many(origin: Point | tuple[float, float], destinations: PointBatch | ArrayLike, *, ellipsoid=None, errors="raise", output="auto") -> DistanceResult`
  - `hausdorff_batch(a: PointBatch | ArrayLike, b: PointBatch | ArrayLike, *, clipped_to: BoundingBox | None = None, errors="raise", output="auto") -> HausdorffResult` (witness indices surfaced as int arrays).
  - `area_batch(polygons: PolygonBatch, *, ellipsoid=None, errors="raise", output="auto") -> AreaResult`
- **Result containers:** Functions return dataclasses with array fields and optional `mask`:
  - `DistanceResult`: `.distance_m`, `.mask`
  - `BearingsResult`: `.distance_m`, `.initial_bearing_deg`, `.final_bearing_deg`, `.mask`
  - `HausdorffResult`: `.distance_m`, `.a_to_b_origin_index`, `.a_to_b_candidate_index`, `.b_to_a_origin_index`, `.b_to_a_candidate_index`, `.mask`
  - `AreaResult`: `.area_m2`, `.mask`
  Each result container exposes `.to_numpy()`, `.to_arrow()`, `.to_python()` mirroring batch conversion semantics.
- **Chunking & streaming:** Constructors and operations accept `chunk_size` to bound peak memory. When set, internal batching yields a single assembled result; a future streaming iterator is deferred to avoid API churn.
- **Threading/GIL:** Rust kernels release the GIL around computation-heavy loops. No implicit multithreading; future multicore support would be an opt-in `concurrency` argument after benchmarking.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Decide API surface for vectorized constructors and outputs | Docstring-level spec for functions/classes (names, args, return types, error handling, optional extras) reviewed and aligned with scalar APIs | Mask semantics, output negotiation, and table interop naming defined; this document is source of truth | ✅ |
| P0 | Implement bulk point/polyline constructors over buffer inputs | Rust/PyO3 entry points accept memoryview/ndarray; validation fast path with lat/lon bounds; optional chunking for large arrays; `_geodist_rs.pyi` updated | Add unit/regression tests covering mixed valid/invalid rows and fallback paths | 📝 |
| P0 | Add vectorized distance/bearing/area kernels for homogeneous batches | Support pairwise and fixed-to-many variants; return ndarray/pyarrow or lists as negotiated; errors surfaced deterministically | Include GIL release and benchmarks for baseline throughput | 📝 |
| P1 | Provide tabular interop adapters (pandas/Arrow) | Helpers to build geometries from DataFrame/Arrow columns with zero-copy where feasible; documented behavior for missing/invalid values | Integration tests across pandas/pyarrow versions in CI matrix | 📝 |
| P1 | Documentation and examples | Add user-facing guide in `pygeodist` docs with realistic examples and performance notes; update README if API is public | Include guidance on coordinate units/order and optional dependencies | 📝 |
| P2 | Performance profiling and tuning | Benchmarks comparing scalar vs vectorized paths; identify bottlenecks and add micro-optimizations or chunk defaults | Could live under `pygeodist/devtools` and inform future tuning | 📝 |
| P3 | Optional multicore/streaming execution | Evaluate opt-in multithreading/streaming for extremely large batches without breaking determinism | Only proceed if P0/P1 performance goals unmet | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Heavy optional deps increase install friction. **Mitigation:** Keep NumPy/pyarrow optional extras; maintain pure-Python fallback with reasonable performance for small batches.
- **Risk:** Silent data corruption from coordinate ordering or invalid values. **Mitigation:** Enforce shape/dtype checks, bounds validation, and explicit mask/exception choices documented in API.
- **Risk:** Memory pressure on huge inputs. **Mitigation:** Provide chunked processing interfaces and document memory expectations; test with large synthetic datasets.
- **Risk:** Divergence between Rust kernels and Python stubs. **Mitigation:** Update `_geodist_rs.pyi` alongside Rust changes; add cross-language parity tests.
- **Risk:** Table interop brittleness across pandas/pyarrow versions. **Mitigation:** Constrain supported versions in extras, add CI coverage, and gate zero-copy paths behind dtype checks with safe fallbacks.

### Open Questions

- Should we expose shapely/GeoPandas helpers, or keep table interop limited to Arrow/pandas to reduce dependencies?
- Do we want streaming iterators for outputs (generator of chunks) in addition to assembled results for memory-constrained users?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Decide API surface for vectorized constructors and outputs._
- **Next up:** _Implement bulk point/polyline constructors over buffer inputs._

## Lessons Learned (ongoing)

- Clear error/mask semantics and explicit output negotiation prevent surprises when optional dependencies are missing or when callers mix container types.
