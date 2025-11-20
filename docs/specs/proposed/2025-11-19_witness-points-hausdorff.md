# Witness Points for Hausdorff (Rust + Python)

## Purpose

- Add witness point reporting to Hausdorff distances so callers can inspect which pair of points realizes the maximum distance in each direction.
- Keep Rust and Python surfaces aligned, covering 2D, 3D, and clipped variants without breaking existing distance-only APIs.
- Avoid ambiguity around indexing and clipping by defining precise return shapes and validation semantics up front.

## Guiding Constraints

- Preserve current distance return paths; add witness details as explicit new APIs or structured returns to avoid silent breakage.
- Maintain lat/lon `_deg` naming and validation rules; clipping continues to use latitude/longitude only (even for 3D).
- Keep performance guardrails: indexed vs naive switch stays; witness reporting must not regress small-set performance materially.
- Propagate clear errors for empty sets and fully clipped sets; witness indices must map back to the provided iterable order.
- Update PyO3 stub typings (`_geodist_rs.pyi`) alongside Rust exports; Python public API stays snake_case with typed returns.

## Target Capabilities

1. Rust Hausdorff kernels return witness information (distance meters + indices of the realizing pair) for directed and symmetric calls, including clipped and 3D variants.
2. Python bindings expose typed results (dataclass/struct-like) that include distance and witness point handles/indices; backwards-compatible helpers remain available.
3. Tests cover 2D/3D, clipped/unclipped, symmetric vs directed, and degenerate cases (duplicates, singletons, fully clipped).

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Define Rust return structs/enums for witness output across 2D, 3D, and clipped Hausdorff | Chosen shape (e.g., `HausdorffWitness { distance_meters, a_idx, b_idx }`, symmetric returns both directions) documented with RustDoc | Decide whether symmetric returns two structs or a combined type; confirm index semantics post-clipping | 📝 |
| P0 | Implement witness-capable kernels in Rust | All Hausdorff paths (directed/symmetric, clipped/unclipped, 3D) compute witness pair and return typed result; errors for empty/fully clipped sets; tests added | Keep indexed vs naive strategy; ensure clipped paths track source indices | 📝 |
| P0 | Expose witness outputs through PyO3 and update `_geodist_rs.pyi` | PyO3 module exports new witness-returning functions/structs; stub updated; Python type hints align | Decide Python result type (frozen dataclass or NamedTuple) and keep distance-only helpers stable | 📝 |
| P1 | Add Python-side tests and docs | Pytest coverage for witness returns across variants; README + notebook mention how to access witnesses | Include negative cases (empty, fully clipped) and ensure indices match original iterable order | 📝 |
| P2 | Performance validation | Bench comparisons showing negligible overhead on small sets and acceptable overhead on large sets with indexing | Add micro-bench or reuse existing Criterion harness; document findings | 📝 |
| P3 | Optional: expose witness point coordinates directly in Python convenience layer | Helper that maps indices back to `Point`/`Point3D` objects for ergonomic use | Only if not too costly; otherwise document how to reconstruct manually | ⏸️ |

### Risks & Mitigations

- **Risk:** Symmetric witness shape could be confusing (two directions). **Mitigation:** Return explicit per-direction structs and document which direction each corresponds to.
- **Risk:** Clipping makes indices ambiguous. **Mitigation:** Define indices relative to original input order before clipping and document behavior when all points are clipped.
- **Risk:** Indexed path loses track of the farthest pair. **Mitigation:** Carry indices alongside coordinates in the index payload; add tests that detect mismatched indices.
- **Risk:** Python API breakage. **Mitigation:** Keep existing distance-only functions unchanged; add new functions or result types for witness-bearing calls with clear typing.

### Open Questions

- Should symmetric witness results be a tuple of two directed results or a dedicated struct with `a_to_b` / `b_to_a` fields?
- Do we return just indices or also the corresponding coordinates in Python for convenience?
- For clipped variants, should we surface the post-clipping point coordinates alongside original indices to aid debugging?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _None yet (new spec)._
- **Next up:** _Lock in Rust return shape and index semantics for all variants._

## Lessons Learned (ongoing)

- _TBD during implementation._
