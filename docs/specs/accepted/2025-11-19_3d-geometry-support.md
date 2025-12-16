# 3D Geometry Support

**Status:** ✅ Done (accepted; core 3D geometry support landed)

## Purpose

- Explore adding optional 3D geometry handling (lat/lon + altitude) while keeping existing 2D performance and API stability.
- Capture assumptions and approach options for the accepted baseline; serves as the record for current 3D support decisions.

## Guiding Constraints

- Preserve fast 2D hot paths: no per-point branching in inner loops.
- Inputs are either all 2D or all 3D per call; no mixed-dimension batches.
- Altitude expressed in meters and finite; lat/lon validation remains unchanged.
- Keep FFI/PyO3 surfaces consistent with Rust types; update `_loxodrome_rs.pyi` alongside Rust bindings.
- Avoid breaking public APIs; new types/flags should be additive and clearly named.

## Target Capabilities

1. Accept validated 3D points (lat, lon, alt_m) alongside existing 2D points.
2. Compute 3D distances using ellipsoid + altitude (ECEF chord or selected metric) with a fixed mode per call.
3. Support Hausdorff over 3D points with appropriate spatial indexing.
4. Expose Python bindings that mirror 2D/3D Rust surfaces without mixed-dimension footguns.
5. Maintain or improve current 2D performance baselines.

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Define distance semantics (surface arc vs 3D chord; which is default) | Decision recorded; downstream tasks reference chosen metric | Default: straight-line chord in ECEF; keep surface arc as future optional mode if needed | ✅ |
| P0 | Add 3D point type + validation (Rust + PyO3 stub) | `Point3D` (or equivalent) validated alt; doc + tests; pyi updated | Ensure no per-point branching for 2D | ✅ |
| P0 | Add mode-aware distance kernel | Mode fixed per call; 2D path unchanged; 3D uses chosen metric; tests | Consider trait or enum mode | ✅ |
| P1 | Extend Hausdorff to 3D | R-tree envelopes in 3D; clipped variants defined or deferred | Keep 2D perf unaffected | ✅ |
| P1 | Python wrappers for 3D | Public API mirrors Rust; docs; tests | Avoid mixed-dimension inputs | ✅ |
| P2 | Benchmarks and perf guardrails | Baseline 2D vs 3D; ensure no regressions | Integrate into CI later; track in perf/CI backlog | ⏸️ |
| P3 | CLI/interop helpers | Optional Typer/interop updates | Only if APIs stabilize | ⏸️ |

### Risks & Mitigations

- **Risk:** Ambiguous “3D distance” definition. **Mitigation:** Decide up front (surface arc, straight-line chord, or both via mode), document default.
- **Risk:** 2D performance regression from branching. **Mitigation:** Fix mode per call; keep 2D code path identical and benchmark.
- **Risk:** Mixed-dimension misuse. **Mitigation:** Validate consistency once per call; offer distinct types/constructors for 2D vs 3D.
- **Risk:** Index/memory overhead in Hausdorff. **Mitigation:** Keep naive vs indexed switch; only use 3D envelopes when in 3D mode.

### Open Questions

- Should 3D “distance” default to straight-line chord or retain surface arc semantics with altitude-adjusted radius?
- Do we need bounding volumes for altitude (3D AABB) or keep 2D clipping only?
- Are there max/min altitude constraints we should enforce?
- Should we expose both 2D and 3D APIs or a single mode-parameterized entry?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** Added 3D Hausdorff kernels (ECEF chord, index + naive), clipping support, and mirrored Python bindings/tests.
- **Next up:** None. Future benchmarks/perf guardrails will be tracked in the perf/CI backlog or separate specs.

## Lessons Learned (ongoing)

- ECEF chord conversion needs explicit ellipsoid validation; keeping it a separate path avoids branching in the 2D geodesic kernels.
- Reusing shared ECEF helpers keeps 3D Hausdorff distance aligned with the pairwise chord metric and avoids re-validating points mid-search.

## Decision: 3D Metric Choice

- Default 3D distance metric: straight-line chord in ECEF using the chosen ellipsoid. This matches “as-the-crow-flies” through-space measurements, lines up with altitude-aware use cases (LOS, flight paths), and avoids adding branching to the 2D hot path.
- Rationale: Arc distance would overstate true 3D straight-line travel and requires altitude-adjusted surface math; the chord keeps the implementation simple and fast while staying consistent with an ellipsoid model.
- Future work: If users need surface-arc semantics with altitude-adjusted radius (or other models), we can add an explicit mode/entrypoint without changing the chord default. Contributions for additional algorithms are welcome; keep the 2D and chord paths monomorphized so hot loops stay branch-free.
