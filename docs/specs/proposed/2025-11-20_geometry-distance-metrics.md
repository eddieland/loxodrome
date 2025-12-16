# Geometry Distance Metrics and Clipping Semantics

## Purpose

- Lock scope for geometry-aware distance metrics (Hausdorff, Fréchet, Chamfer) over LineString/Polyline, Polygon with holes, and Multi* inputs so Rust and Python surfaces converge.
- Define sampling and clipping semantics up front to keep results auditable, reproducible, and consistent with existing geodesic kernels.

## Guiding Constraints

- Coordinates are lat/lon degrees on WGS84 by default; alternate ellipsoids or spherical radius remain explicit inputs. No implicit projections or axis reordering.
- Validate bounds/finiteness for every vertex; reject degenerate rings (fewer than 4 coords or unclosed) and self-intersections where detectable without heavy overlays. Require holes to lie inside the outer ring and use CCW exterior / CW holes with closure tolerance (e.g., ≤1e-9 deg).
- Sampling is deterministic: densification uses fixed tolerances (max segment length in meters or max angular separation in degrees) and always includes original vertices. Default knobs follow common GIS practice: `max_segment_length_m=100` (or `max_segment_angle_deg=0.1`), `interior_spacing_m=200`, and a hard cap of 50_000 samples/geometry (error if exceeded).
- Metrics operate in 2D geodesic space; altitude-aware variants defer to a later spec. All distances use the same kernel as `geodesic_distance`/Hausdorff today.
- Clipping uses axis-aligned lat/lon bounding boxes with inclusive edges; clipping affects evaluation points, not input validation. Bboxes that wrap the antimeridian are allowed (lon_min > lon_max).
- Performance envelope: nearest-neighbor search uses kd-tree or equivalent O(N log N); targets ≤500 ms and ≤200 MB for 50k samples/side on a modern laptop. Document deviations and tuning levers.

## Target Capabilities

1. Support LineString/Polyline and MultiLineString inputs with densification controls for metric accuracy without exploding samples.
2. Support filled Polygon geometries (exterior + holes) and MultiPolygon by sampling boundaries plus interior coverage points so distances reflect filled areas, not just ring perimeters.
3. Implement directed and symmetric Hausdorff, discrete Fréchet for path similarity, and Chamfer (mean and optional max) with consistent witness reporting, tolerances, and clipping semantics.

## Geometry & Sampling Semantics

- **LineString/Polyline:** Treat segments as great-circle arcs between vertices. Densify segments where chord length exceeds `max_segment_length_m` (default 100 m) or heading change exceeds `max_segment_angle_deg` (default 0.1 deg); either knob required. Always preserve endpoints. MultiLineString flattens to per-part samples with part offsets retained for witness mapping. Raise `InvalidGeometryError` if densification would exceed the 50k-sample cap.
- **Polygon:** Polygons represent filled regions (exterior minus holes). Sample every ring with the same densification knobs as polylines, respecting orientation and closure tolerance. Add interior coverage points per polygon via a quasi-grid seeded from bbox intersections at `interior_spacing_m` (default 200 m) so distances account for filled area. Holes contribute samples that mark voids (distance counts from hole boundary outward). Reject polygons whose holes overlap or touch the exterior. For shapes crossing the antimeridian or touching poles, grid seeding respects wraparound and clips at ±90° without distorting longitudes.
- **MultiPoint/Line/Polygon:** Validate homogeneous dimensionality. Evaluate metrics componentwise but report a unified witness that includes the originating part index.

### Polyline densification & validation (P0)

- Validate every vertex before sampling: lat in [-90°, 90°], lon finite (±180° inclusive), no NaN/Inf. Report `InvalidGeometryError` with the 0-based vertex index on failure so callers can prune bad inputs early.
- Consecutive duplicate vertices are allowed but generate a single sample; zero-length segments are skipped for densification to avoid exploding counts while preserving original vertex ordering for witness indices.
- Per segment, compute the number of subsegments as `n = max(ceil(dist_m / max_segment_length_m), ceil(heading_change_deg / max_segment_angle_deg))`, ignoring a term when its knob is unset. Enforce `n >= 1`; insert intermediate points at equal arc-length fractions along the great-circle between the two vertices. Always emit the first vertex of the geometry, then append intermediates, then the segment’s end vertex to keep deterministic ordering.
- Require at least one densification knob; if both are missing, raise `InvalidGeometryError` with a message pointing to `max_segment_length_m` and `max_segment_angle_deg` defaults.
- Apply the 50_000-sample cap across the fully densified geometry (after duplicate collapsing). When predicted samples exceed the cap, fail fast with the expected count and part index. Example: a 6,000 km LineString at the 100 m default would need ~60,001 samples, so it errors and suggests raising `max_segment_length_m` or using a larger `max_segment_angle_deg`.
- Deterministic sampling examples to anchor fixtures: a 10 km LineString at defaults produces 101 samples (100 m spacing with endpoints retained); a 250 m segment with both knobs set uses the max of the two splits so spacing never exceeds either tolerance.

### Polyline Hausdorff + Chamfer APIs (P0)

- Python shape (public API):
  - `loxodrome.hausdorff(polyline_a, polyline_b, *, symmetric=True, bbox=None, max_segment_length_m=100.0, max_segment_angle_deg=0.1, sample_cap=50_000, return_witness=False) -> float | tuple[float, Witness]`.
  - `loxodrome.chamfer(polyline_a, polyline_b, *, symmetric=True, reduction="mean", bbox=None, max_segment_length_m=100.0, max_segment_angle_deg=0.1, sample_cap=50_000, return_witness=False) -> float | tuple[float, Witness]`.
  - `polyline_*` accept `LineString` or `MultiLineString`-shaped inputs (array-likes or shapely/geojson-compatible tuples). Require at least one densification knob. `symmetric=False` computes the directed A→B variant.
- Rust shape (crate API sketch):
  - `fn hausdorff_polyline(a: &Polyline, b: &Polyline, opts: HausdorffOpts) -> HausdorffResult;`
  - `fn chamfer_polyline(a: &Polyline, b: &Polyline, opts: ChamferOpts) -> ChamferResult;`
  - Options mirror Python: `symmetric: bool`, `reduction: Reduction`, `bbox: Option<BoundingBox>`, densification knobs, `sample_cap`, and `return_witness` equivalent.
- `_loxodrome_rs.pyi` witness typing: `class Witness(TypedDict)` with fields: `source_part: int`, `source_index: int`, `target_part: int`, `target_index: int`, `source_coord: tuple[float, float]`, `target_coord: tuple[float, float]`, `distance_m: float`.
- Witness emission rules:
  - Directed Hausdorff returns the farthest (max) distance sample in the A→B direction; symmetric picks the realizing witness from the worse direction (A→B if equal after distance tie-break below).
  - Chamfer only emits a witness when `reduction="max"` (otherwise aggregates distances without per-point payload). Witness schema matches Hausdorff; `distance_m` reflects the worst offending sample.
  - Tie-break ordering: compare `distance_m` (with tolerance 1e-12 m for floating noise), then lowest `source_part`, then `source_index`, then `target_part`, then `target_index`. This must be deterministic across Rust/Python.
- Return types: when `return_witness=False`, return scalar distance (float). When `True`, return `(distance, witness)` tuple; witness is `None` if the geometry empties from clipping (but note Hausdorff/Chamfer already error on empty after clip).
- Error paths: propagate validation errors from densification; raise `InvalidGeometryError` when sampling cap is exceeded or inputs are empty post-clip. Clipping rules follow the section below.

## Distance Metric Semantics

- **Hausdorff:** Provide directed (`A→B`) and symmetric (`max(A→B, B→A)`) over sampled point sets. Witness includes `source_part`, `source_index`, `target_part`, `target_index`, `source_coord`, `target_coord`, `distance_m`, with zero-based indices and tie-breaker on lowest `source_index` then `target_index`. Optionally return both distance and realizing coordinates.
- **Fréchet:** Use discrete Fréchet on densified polylines (and ring boundaries). Respect vertex order; multi-part inputs either match paired parts or apply a documented set rule—default is max over directed distances across parts with witness of traversal paths (arrays of indices) to mirror Shapely/JTS expectations. Continuous variant deferred. Zero-based indices.
- **Chamfer:** Compute mean nearest-neighbor distance for each direction with optional `reduction="mean" | "sum" | "max"`; symmetric mode averages (or maxes) both directions. Witness reports worst offending sample with the same schema as Hausdorff when `reduction="max"`. Default reduction is `mean` (scale-invariant).

### MultiLineString acceptance (P0)

- **Shape + validation:** Accept sequences of LineStrings (array-like, GeoJSON, or Shapely MultiLineString) with at least one part containing two or more vertices. Validate every part independently with polyline rules (bounds, finiteness, densification knobs). Reject MultiLineStrings where all parts are empty/degenerate (`InvalidGeometryError` with part index when applicable).
- **Densification + caps:** Densify each part using the same `max_segment_length_m`/`max_segment_angle_deg` knobs. Collapse consecutive duplicates within a part before counting samples. Apply the 50_000-sample cap to the flattened, densified geometry; error messages report the offending part index and predicted total so callers can coarsen tolerances.
- **Flattening for search:** Flatten part samples into a single ordered array for nearest-neighbor search while retaining `part_start_offsets` so witness indices map back to `(part, vertex_within_part)`. KD-tree (or equivalent) builds over the flattened array; per-part boundaries do not alter distance evaluation.
- **Witness payloads:** `Witness.source_part`/`target_part` refer to the originating MultiLineString part; `source_index`/`target_index` are the vertex positions within those parts post-densification (duplicates collapsed). Tie-break ordering remains distance → source_part → source_index → target_part → target_index to keep deterministic results across Rust/Python.
- **Fixtures (sketch):** (1) Two-part A vs single-part B where only part 1 contributes the Hausdorff witness; ensure part index surfaces. (2) MultiLineString with a zero-length part and a valid part errors on the bad part index. (3) Antimeridian-wrapping part retains ordering and part indices after densification.

## Clipping Rules

- Accept optional `BoundingBox` (lat_min, lon_min, lat_max, lon_max). Samples on edges count as inside. lon_min > lon_max denotes antimeridian-wrap; clip using great-circle intersections and preserve vertex order.
- For Hausdorff/Chamfer, discard samples outside the bbox before nearest-neighbor search; empty result after clipping raises `InvalidGeometryError` and reports which side emptied.
- For Fréchet, clip lines by truncating to the bbox-intersecting portions (keep vertex order, insert intersection points as needed) before densification; if a path fully clips out, treat it as empty and raise `InvalidPathError`.
- Clipping never rewrites input coordinates outside validation; it only gates evaluation.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working. Each task must meet accuracy (within sampling tolerance/2), perf (≤500 ms, ≤200 MB at 50k samples/side), and stability (witness schema unchanged) gates to call DoD.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Lock densification + validation for LineString/Polyline inputs | Defaults, bounds, and deterministic sampling order documented; sample-cap behavior called out with examples | Sets the first shippable surface | ✅ |
| P0 | Specify Hausdorff + Chamfer APIs and witness payloads for polylines | Function signatures, reduction modes, tie-break rules, and `_loxodrome_rs.pyi` shape captured; matches current point Hausdorff contract | Enables early Rust/Python delivery on polylines | ✅ |
| P0 | Add end-to-end MultiLineString acceptance | Validation + sampling rules defined; witness shape records part indices; tests/fixtures sketched for LineString + MultiLineString parity | Delivers the first “additional data type” increment | ✅ |
| P0 | Define clipping behavior for polyline metrics | Bbox rules, antimeridian handling, and empty-geometry failures documented with examples | Keeps first wave auditable | ✅ |
| P1 | Ring validation + densification for Polygon/MultiPolygon (boundary-only) | Closure/orientation/containment checks and sampling defaults captured; explicit note that interior coverage is deferred | Unblocks perimeter-only distances as a second increment | ✅ |
| P1 | Polygon boundary Hausdorff/Chamfer witness + API shape | Witness payloads and tie-breaks defined; `_loxodrome_rs.pyi` updates described; fixtures outlined | Builds on polyline work before interior fill | ✅ |
| P1 | Draft polyline-focused test matrix | Golden cases for multi-part polylines, crossing lines, and clipped evaluations; tolerances stated | Guards early delivery | 📝 |
| P2 | Extend to interior coverage grids + Fréchet semantics across polygons | Grid seeding and Fréchet traversal rules documented; perf/complexity notes updated | Activates filled-area accuracy | 📝 |
| P2 | Performance + memory targets | Sampling defaults justified with complexity bounds; profiling plan outlined | Adjust after initial benchmarks | 📝 |
| P3 | Optional extras (adaptive sampling, altitude variants) | Conditions for expanding scope documented | Defer until core metrics stabilize | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Overly coarse sampling skews distances or misses hole coverage. **Mitigation:** Provide explicit spacing knobs with conservative defaults and fixture-based regression tests.
- **Risk:** Clipping semantics diverge between metrics. **Mitigation:** Centralize bbox handling helpers and reuse across kernels; document error paths.
- **Risk:** Witness payloads become unstable across implementations. **Mitigation:** Define deterministic tie-breaking (e.g., lowest index) and include in tests.

### Open Questions

- Are interior coverage points sufficient, or do we need adaptive refinement near thin polygons/holes?
- Should Fréchet expose both discrete and continuous variants, or is discrete plus densification enough for v1?
- Do we expose Chamfer `reduction="sum"` by default or prefer `mean` to stay scale-invariant?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Ring validation + densification for Polygon/MultiPolygon (boundary-only)._
- **Next up:** _Draft polyline-focused test matrix._

## Lessons Learned (ongoing)

- Deterministic sampling and shared clipping helpers are essential to keep Rust/Python parity and reproducibility.
- Sample-cap rejections must spell out expected counts and tuning knobs so callers can pick coarser tolerances without silent truncation.
- Witness schemas need deterministic tie-break ordering and shared `TypedDict`/struct definitions to keep Rust and Python aligned.
