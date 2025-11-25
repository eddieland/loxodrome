# ECEF Interop and Conversions

## Purpose

- Provide first-class ECEF (Earth-Centered, Earth-Fixed) interoperability so callers can convert to/from geodetic coordinates, compute distances, and run Hausdorff directly in Cartesian space.
- Align conversion accuracy with our ellipsoidal geodesic solver and the accepted 3D chord metric, avoiding duplicated or inconsistent math across Rust and Python surfaces.
- Non-goals: projection support beyond ECEF, datum shifts, or arbitrary CRS transformations.

## Guiding Constraints

- Reuse the existing ellipsoid model and validation (WGS84 default) from `Ellipsoid`; no extra datum configs.
- Preserve API symmetry: Rust and Python must expose the same ECEF capabilities, with `_geodist_rs.pyi` kept in sync.
- Keep 2D and 3D hot paths branch-free; ECEF entrypoints should be distinct, not mode switches inside geodetic kernels.
- Ensure numerical stability for typical altitude ranges; document limits and test round-trips against trusted references.
- Favor lightweight dependencies; prefer in-crate math over new external crates.

## Target Capabilities

1. Public ECEF types and conversion helpers (`geodetic_to_ecef`, `ecef_to_geodetic`) that accept a validated ellipsoid and are exposed in Rust and Python.
2. Direct ECEF distance/Hausdorff entrypoints that accept ECEF coordinates without re-validating geodetic inputs, plus batch helpers for slices/arrays.
3. CLI and docs coverage (README + notebook) demonstrating conversions, defaults, and accuracy expectations, with deterministic fixtures/benchmarks.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Public ECEF structs + conversions | Expose `EcefPoint`; add `ecef_to_geodetic`; keep `geodetic_to_ecef` validating; document defaults; unit tests with WGS84 fixtures | Aligns with 3D metric decision (ECEF chord) and ellipsoid upgrade spec | 📝 |
| P0 | Python surface + typings | PyO3 bindings + `_geodist_rs.pyi` + `ops.py` wrappers for ECEF points and conversions; validation mirrors Rust; pytest fixtures | Maintain snake_case and keep public API at top of modules | 📝 |
| P1 | Direct ECEF kernels | Rust + Python entrypoints for single/batch distance and Hausdorff that accept ECEF coords; avoid geodetic revalidation; benches for conversion vs direct ECEF | Share math with existing 3D chord kernels where possible | 📝 |
| P1 | CLI and docs | Extend CLI with `to-ecef`/`from-ecef` (and ECEF distance) commands; README + notebook examples; note ellipsoid defaults and accuracy ranges | Add deterministic round-trip examples | 📝 |
| P2 | Benchmarks and perf guardrails | Criterion benches for conversions and ECEF kernels; doc expected ranges; consider CI gating later | Reuse existing perf harness patterns | ⏸️ |
| P3 | Extended validation options | Optional altitude/radius bounds and custom ellipsoid inputs for callers needing stricter checks | Only if requested; keep defaults lenient | ⏸️ |

### Risks & Mitigations

- **Risk:** Divergence between Rust and Python surfaces or stale stubs. **Mitigation:** Update `_geodist_rs.pyi` with code changes; add integration tests that import both paths.
- **Risk:** Numerical drift in `ecef_to_geodetic` near poles/antipodes. **Mitigation:** Use trusted formulas with tolerances; add fixtures from GeographicLib/authoritative sources.
- **Risk:** Hidden perf regressions from conversions in hot paths. **Mitigation:** Offer direct ECEF APIs to bypass conversions; benchmark and document expected overhead.

### Open Questions

- Do we need explicit altitude bounds (e.g., clamp to ±100 km) or keep current validation only?
- Should ECEF batch helpers accept/return `numpy` arrays directly when available, or stay list-based in v1?
- Do we expose ellipsoid selection on CLI commands or keep WGS84 default with a flag?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _None yet._
- **Next up:** _Publish ECEF structs and conversions._

## Lessons Learned (ongoing)

- _TBD._
