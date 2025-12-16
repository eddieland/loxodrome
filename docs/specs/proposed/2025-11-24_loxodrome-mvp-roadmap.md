# Loxodrome MVP geometry roadmap

## Purpose

- Capture the MVP target list driven by the latest MARS/LIMA use case so we stop drifting and can track delivery against specific capabilities.
- Acknowledge prior specs moved in this direction without enumerating the must-have features; this document sets the explicit checklist.
- Exclude non-embeddable dependencies (e.g., C++ tessellator) while allowing us to study their approaches if useful.

## Guiding Constraints

- Keep the Rust kernels as the source of truth; Python bindings stay thin and typed (`strict = true` mypy, Ruff formatting).
- Maintain WGS84 as the default geodesic model; if ENU helpers are added they must be origin-bound, opt-in, and reversible.
- Preserve Z coordinates end-to-end when a geometry is 3D; avoid silent Z drops, especially across interop layers.
- Optional deps remain optional (e.g., Shapely); any GEOS-backed boolean ops must not bloat the core install.
- No direct adoption of the legacy C++ tessellator; lessons learned may inform a Rust-native tessellation.
- Performance guardrails: spatial indexing and tessellation must outperform O(n²) baselines on representative road-network workloads.

## Target Capabilities

1. Round-trip-safe 2D/3D geometry wrappers (Point, LineString, Polygon, Multi- equivalents or documented collection pattern) with Shapely interop that preserves Z where supported; build on existing Point/Point3D/LineString/Polygon wrappers.
2. Core operations: geodesic distance/bearings (sphere + ellipsoid) and point Hausdorff are already shipped; remaining scope is linestring/polygon Hausdorff and polygon/linestring boolean ops (union/intersection) with clear fallbacks.
3. Spatial index: keep the existing `rstar` R-tree as the baseline (already used for Hausdorff fast paths) and layer nearest/intersects wrappers with configurable bounds and Z-aware storage; document Euclidean envelope vs geodesic caveats.
4. Tessellation pipeline for Z comparison: densify boundaries by distance/angle, triangulate strips, compute per-segment/surface metrics.
5. Coordinate system story: default WGS84; optional WGS84↔ENU helpers if justified, with docs on when to use each for metrics and visualization.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Lock geometry surface for MVP | Decide on Multi* wrappers vs. endorsed `list[LineString|Polygon]`; add 3D LineString/Polygon (with validation/orientation checks) and update Shapely interop to preserve Z | Update `_loxodrome_rs.pyi`, docs, and tests; no silent Z drops | 📝 |
| P0 | Spatial index API and docs (rstar baseline) | Lock `rstar` as the index, expose build + nearest/intersects wrappers using Euclidean envelopes, and benchmark vs. O(n²) on representative datasets | Preserve Z payloads, surface fanout/bounds knobs, and document geodesic/antimeridian limitations | 🚧 |
| P0 | Boolean ops path | Provide polygon/linestring union/intersection via Rust kernel or optional GEOS backend; document error cases and precision tradeoffs | Ensure optional dependency isolation; add tests for holes/degenerate cases | 📝 |
| P0 | Tessellation for Z comparison | Design densify-then-triangulate pipeline (configurable spacing/angle/sample caps) and surface comparison metrics | Include tests on synthetic surfaces; document failure modes and performance knobs | 📝 |
| P1 | Coordinate system guidance | Decide on WGS84-only vs. WGS84+ENU helpers; document accuracy implications and visualization workflows | Add examples for LIMA debugging (WKT export) and MARS metrics | 📝 |
| P1 | Z-handling contract | Document guarantees and errors across all geometry/interop paths; add regression tests for Z retention | Covers Shapely conversions, indexes, and operations | 📝 |
| P2 | Batch and vectorized UX polish | Expand docs/snippets for batch geodesics/Hausdorff; ensure new geometries fit vectorized APIs | Include performance notes and memory caveats | 📝 |
| P3 | Stretch: area-based Hausdorff/overlays | Evaluate need for area-weighted Hausdorff or overlay-derived metrics | Only if driven by user demand; may defer | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Precision and robustness regress with new boolean ops or tessellation. **Mitigation:** Property-based and fixture tests across edge cases; document tolerances and coordinate bounds.
- **Risk:** Optional dependency creep (GEOS/Shapely) bloats install. **Mitigation:** Keep core extra-free; gate optional ops behind extras and guard imports.
- **Risk:** Z handling regresses via interop or index storage. **Mitigation:** Enforce Z-round-trip tests and explicit errors on unsupported 3D paths.
- **Risk:** Performance goals not met on road-network-scale data. **Mitigation:** Benchmarks in `devtools/`; tune index params and tessellation sampling caps; provide defaults plus override knobs.

### Open Questions

- Should MultiLineString/MultiPolygon be first-class or should we standardize on lists with helper utilities?
- Do we commit to WGS84-only for metrics, or add ENU helpers for specific workflows (and how do we document the choice)?
- Are GEOS-backed boolean ops acceptable as an optional extra, or do we wait for Rust-native kernels?
- What outputs are expected from tessellation (triangles, per-segment stats, max deltas) to satisfy the Z comparison use case?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Locked `rstar` as the spatial index baseline for Hausdorff fast paths per accepted 2025-11-19 spec._
- **Next up:** _Expose public index wrappers + docs on bounds/lat-lon caveats, then deepen geometry surface and boolean/tessellation coverage._

## Lessons Learned (ongoing)

- _Prior specs gestured at richer geometry support but lacked a crisp target list; this roadmap pins the MVP commitments._
