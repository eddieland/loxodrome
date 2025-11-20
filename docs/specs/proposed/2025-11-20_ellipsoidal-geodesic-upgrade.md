# Ellipsoidal Geodesic Upgrade

## Purpose

- Deliver true ellipsoidal geodesic computations (distance + bearings) so results align with geodesy references (e.g., WGS84) rather than mean-radius approximations.
- Serve users who need sub-meter to meter-level accuracy over long baselines (aviation routing, maritime planning), compliance-sensitive domains (surveying, cadastral, governmental datasets), and regional datum work where spherical error (~0.3–0.5%) is unacceptable.
- Keep the Python bindings and Rust kernels consistent: no divergence between the Rust API and PyO3 surface. Non-goal: broad projection support or datum shifts; focus on WGS84 and caller-provided ellipsoids.

## Guiding Constraints

- Numerical stability across polar/antipodal cases; avoid NaNs and large bearing swings.
- Deterministic outputs for identical inputs; avoid randomization or heuristic fallbacks.
- Performance: single-call latency close to the current spherical path for short ranges; allow slower but bounded performance for long-range/high-precision paths. Batch APIs should remain efficient (vectorized code paths or batched solver).
- API compatibility: preserve existing signatures and add new ones without breaking callers; deprecate only when necessary.
- Tests must use deterministic fixtures and cite authoritative references (e.g., GeographicLib, NGS examples).

## Target Capabilities

1. True ellipsoidal distance and bearing solver for `geodesic_distance_on_ellipsoid`, `geodesic_with_bearings_on_ellipsoid`, and 3D ECEF conversions consistent with the chosen model.
2. Batch support (`geodesic_distances_*`) using the ellipsoidal solver without losing ordering or validation semantics.
3. Python bindings expose ellipsoidal variants with accurate typing and documented accuracy expectations.
4. Benchmarks and validation suite against trusted references (GeographicLib, published geodesic test sets).

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | PoC: integrate `geographiclib-rs` via adapter | Add a thin adapter trait around `geographiclib-rs` inverse solver; wire `geodesic_*_on_ellipsoid` + bearings to it; keep spherical path unchanged | Build behind minimal dependency; guarded by tests | ✅ Done |
| P0 | Reference fixtures + validation | Add authoritative test pairs (GeographicLib tables) incl. antipodal/polar/short-haul; verify meters/bearings match within tolerance | Cite sources; codify tolerances | 📝 |
| P0 | Expose PoC through PyO3 + Python API | Extend `python.rs`, `_geodist_rs.pyi`, and `ops.py` to surface ellipsoidal variants; keep spherical exports intact | Pytest coverage for new functions and error paths | ✅ Done |
| P1 | Bench PoC vs spherical | Bench single + batch calls; record perf delta and accuracy; decide if PoC is acceptable for initial ship | Document findings in spec/README | 📝 |
| P1 | Decide fork/vendoring strategy | Based on PoC results + upstream health, choose: keep dep, vendor fork, or build in-tree solver | Decision logged with rationale | 📝 |
| P2 | Deprecation messaging | If mean-radius remains default, add doc/docs clarifying approximation and steering to ellipsoid | Non-breaking; doc-level first | 📝 |
| P3 | Future: flexible solvers | Pluggable strategy interface for alternate geodesic solvers (e.g., lower-precision fast path) | Design sketch only; no code yet | ⏸️ |

### Risks & Mitigations

- **Risk:** Vincenty fails to converge near antipodal points. **Mitigation:** Prefer Karney/GeographicLib formulation or add robust fallback with iteration caps and error surfacing.
- **Risk:** Performance regression on batch workloads. **Mitigation:** Benchmark early; consider caching constants, vectorization, or hybrid spherical fast-path when errors are below tolerance and caller opts in.
- **Risk:** API churn across Rust/Python surfaces. **Mitigation:** Add new functions first; deprecate spherical-only approximations via docs, not removals; keep `_geodist_rs.pyi` and Rust PyO3 in sync.
- **Risk:** Reference fixtures mismatch due to unit or ellipsoid discrepancies. **Mitigation:** Pin ellipsoid parameters in tests and cross-check with multiple sources; document provenance alongside fixtures.

### Decisions

- **Solver choice:** Start with Karney via `geographiclib-rs` (pure Rust, no C FFI). Keep an adapter boundary so we can vendor/fork or replace with an in-crate solver if upstream health becomes a concern.
- **Default behavior:** Make ellipsoidal the default (accuracy-first) once the PoC lands; keep spherical available as an explicit fast-path override for callers who accept the approximation.
- **Precision/iterations:** Target sub-millimeter agreement vs GeographicLib references on WGS84; accept ≤1e-6 relative error on distance and ≤1e-9 radians (~5.7e-8 degrees) on bearings. Use the library defaults for iterations in PoC; cap at ~50 iterations if we own the loop later and surface errors on non-convergence.
- **Batch strategy parameter:** Not needed in the PoC. If demand for mixed precision arises, add a strategy enum later; for now, batch APIs will use the default ellipsoidal path or explicit spherical entrypoints.

## Status Tracking (to be updated by subagent)

- **Latest completed task:** PoC adapter + Python exposure using `geographiclib-rs`.
- **Next up:** Reference fixtures and validation coverage (P0).

## Lessons Learned (ongoing)

- _TBD_
