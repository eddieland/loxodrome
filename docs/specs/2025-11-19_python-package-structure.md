# Python package structure plan

## Purpose

- Define a stable Python package layout that cleanly layers ergonomic APIs on top of the Rust geodist kernels while staying maintainable for future contributors.
- Capture lessons from Shapely, GEOS/pygeos, and geo-rs on separation of geometry models, IO, and kernel dispatch so we avoid ad-hoc growth and keep a clear compatibility story.

## Guiding Constraints

- Keep Rust bindings thin and private; public Python surface should be pure Python shims with narrow, typed calls into `_geodist_rs`.
- Geometry objects stay lightweight, immutable wrappers holding opaque Rust handles; avoid Python-level mutation to mirror Shapely/pygeos semantics.
- Preserve deterministic, thread-safe behavior (no hidden globals); explicitly manage allocator/threading handoffs across the FFI boundary.
- Strict typing and docs for public APIs; match `_geodist_rs.pyi` with the compiled module and enforce mypy strictness.
- Keep import graph acyclic and layered: `geometry` → `ops` → `io`/`vectorized` should not depend inward on CLI/tooling.
- Shapely interop is supported but optional; no hard dependency on install, with helpers that only import shapely when needed.

## Target Capabilities

1. Geometry core module providing user-facing classes (`Point`, `LineString`, `Polygon`, `GeometryCollection`) backed by Rust handles plus constructors from common inputs.
2. Operations module exposing predicates and measures (distance, intersects, within, equals, buffer when available), dispatching through the Rust kernels with stable return types.
3. IO module handling WKT/WKB/GeoJSON round-trips and SRID propagation, mapping cleanly to Rust parsers/serializers.
4. Vectorized facade for batch operations over Python sequences/NumPy arrays mirroring pygeos/Shapely `vectorized` ergonomics when kernels support it.
5. Utilities for error handling, CRS validation, and feature flags, keeping CLI/tooling separate from library concerns.
6. Optional shapely interop helpers for converting to/from shapely geometries without introducing a hard dependency.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Finalize package module skeleton (`geodist/geometry.py`, `ops.py`, `io.py`, `vectorized.py`, `errors.py`, `types.py`) and re-export width under `geodist/__init__.py`. | Files exist with placeholder classes/functions, layered imports, and docstrings describing contracts. | Mirror Shapely public surface where sensible; keep `_geodist_rs` private. | ✅ |
| P0 | Design geometry wrappers and lifetime model around Rust handles. | `Point`/`LineString`/`Polygon` classes declared with slots, opaque handle storage, equality/repr/len semantics defined; `_geodist_rs.pyi` updated. | Follow pygeos immutability; document thread-safety expectations; include factory utilities. | ✅ |
| P0 | Define operations API with typed predicates/measures and error mapping. | `ops.py` exposes functions/methods routing to kernels; consistent exceptions in `errors.py`; unit tests add shape+distance happy-paths. | Align naming/return types with Shapely where possible; ban silent coercions. | ✅ |
| P1 | IO layer for WKT/WKB/GeoJSON and CRS metadata propagation. | `io.py` implements serializers/parsers with round-trip tests; explicit CRS argument validation and passthrough. | Prefer Rust parsing for performance; Python-side validation of inputs. | ✅ |
| P1 | Vectorized facade and small-array fallback. | `vectorized.py` provides array-oriented wrappers with graceful fallback to Python loops; NumPy optional dependency guarded. | Benchmarks documented; matches Shapely vectorized ergonomics. | 📝 |
| P1 | Shapely interop helpers. | Optional `interop_shapely.py` (or similar) with `to_shapely`/`from_shapely` converters; imports guarded and typed; tests skipped when shapely absent. | No mandatory shapely dependency; clear error messages when helpers used without shapely installed. | 📝 |
| P2 | CLI and devtools alignment with new structure. | CLI uses public API only; devtools (benchmarks/fixtures) updated; no private imports. | Keep CLI optional dependency. | 📝 |
| P3 | Extension hooks for future features (buffer, densify, nearest-neighbor). | Extension points documented; stubs with `NotImplementedError` behind feature flags. | Avoid API churn; document roadmap. | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Divergence between `_geodist_rs` signatures and Python type hints leads to runtime mismatches. **Mitigation:** Gate merges on updating `_geodist_rs.pyi` plus smoke tests that import and call each public symbol.
- **Risk:** Circular imports across geometry/ops/io when adding helpers. **Mitigation:** Enforce layer boundaries; prefer small helper modules (`types.py`, `errors.py`) for shared pieces.
- **Risk:** Performance regressions from Python-side loops before vectorized kernels land. **Mitigation:** Provide optional NumPy fast paths and document expected complexity; add benchmarks to catch regressions.
- **Risk:** CRS/GeoJSON semantics drift from Shapely/GEOS expectations. **Mitigation:** Write compatibility tests against simple shapely fixtures; document deviations explicitly.

### Open Questions

- How closely should we mirror Shapely method names vs. provide functional APIs (e.g., `geom.distance(other)` vs. `distance(a, b)`)?
- Do we require NumPy at install time for vectorized helpers, or keep it optional with clear performance notes?
- What CRS metadata model do we adopt (integer SRID only vs. full authority strings), and how is it stored alongside Rust handles?
- Which shapely versions do we target for interop, and how do we behave when shapely is missing or outdated?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _IO layer for WKT/WKB/GeoJSON and CRS metadata propagation._
- **Next up:** _Vectorized facade and small-array fallback._

## Lessons Learned (ongoing)

- _Document gaps between Rust kernel capabilities and Python-facing expectations early to avoid API rework._
- _Raise explicit KernelUnavailableError stubs so imports succeed while keeping missing kernels visible during early scaffolding._
- _Provide placeholder handle types so geometry wrappers remain importable before kernels attach real handles._
- _Initialize kernel stubs before importing the public surface to avoid circular import issues when the extension is absent._
- _Apply light CRS validation in Python before deferring to kernels; accept GeoJSON `crs` hints when provided._
