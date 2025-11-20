# Python package structure plan (simplified)

## Purpose

- Keep the Python surface constrained to what the Rust kernels expose while still presenting expressive geometry values (Rust structs surfaced via PyO3) instead of bare tuples.
- Avoid mirroring Shapely’s full API, but provide opt-in converters to and from Shapely objects so users can interoperate without rewriting pipelines.
- Document the minimal contract between Python and Rust to avoid overpromising capabilities and to keep interop helpers honest about supported geometry kinds.

## Guiding Constraints

- Keep `_geodist_rs` private and map it to a minimal public API; expose Rust geometry structs directly (opaque handles) with lightweight Python wrappers for typing and convenience.
- Prefer explicit geometry structs over free-form tuples so we can evolve kernels without breaking positional contracts; keep constructors simple and validated.
- Shapely interop is optional and isolated: helpers live in a small module, imports are guarded, and conversions are explicit.
- Strict typing and docs remain required; keep `_geodist_rs.pyi` in sync with compiled symbols and include Rust-backed geometry classes.
- CLI remains a dev-only helper and must not imply capabilities that the Rust layer does not provide.

## Target Capabilities

1. Re-export constants supplied by Rust (currently `EARTH_RADIUS_METERS`) with stable typing.
2. Expose Rust geometry structs (e.g., `Point`, `LineString`) through thin Python wrappers that preserve immutability and validation semantics.
3. Add stateless functions once kernels exist (e.g., `geodesic_distance(point_a, point_b)`) operating on the Rust-backed geometry types.
4. Provide optional Shapely converters (`to_shapely`, `from_shapely`) so users can bridge ecosystems without treating Shapely as a dependency.
5. Keep packaging/dev tooling aligned with the minimal surface (entrypoint optional, extension import guarded).

## Subagent Execution Plan

The following backlog is prioritized for a single subagent (or small group) to implement iteratively. Update the _Status_ and _Lessons Learned_ sections while working.

### Task Backlog

Use emoji for status (e.g., ✅ done, 🚧 in progress, 📝 planned, ⏸️ deferred).

| Priority | Task | Definition of Done | Notes | Status |
| -------- | ---- | ------------------ | ----- | ------ |
| P0 | Rewrite public API scope doc and `__all__` to reflect the minimal surface (constants, errors, Rust-backed geometry wrappers). | README and docstrings describe the small API; no promises of Shapely breadth. | Aligns consumer expectations with reality. | ✅ |
| P0 | Define Rust-backed geometry wrappers and constructors. | `_geodist_rs.pyi` exposes core structs; Python wrappers validate inputs and keep immutability; no Shapely dependency. | Keeps typing ready while matching Rust models. | ✅ |
| P1 | Ship optional Shapely conversion helpers and verify packaging/CLI alignment. | `interop_shapely.py` converts to/from wrappers (guarded imports, skipped tests when missing); packaging keeps deps lean; CLI reflects the limited API. | Bundles small tasks to avoid churn. | 📝 |
| P2 | Document non-goals and future kernel exposures without promising timelines. | README states Shapely parity is out of scope; guidance for interop users; backlog of candidate functions gated on Rust readiness. | Reduces support burden and avoids churn. | 📝 |

_Add or remove rows as necessary while keeping priorities sorted (P0 highest)._

### Risks & Mitigations

- **Risk:** Overpromising capabilities leads users to expect Shapely-like breadth. **Mitigation:** Keep docs and `__all__` minimal; explicitly list non-goals.
- **Risk:** Divergence between `_geodist_rs` and `_geodist_rs.pyi` once functions/structs arrive. **Mitigation:** Block merges on updating stubs and adding smoke tests for each exported symbol.
- **Risk:** Shapely conversions hide unsupported geometry kinds. **Mitigation:** Validate and error loudly for unsupported types; keep compatibility tests minimal and explicit.
- **Risk:** CLI implies broader features. **Mitigation:** Wire CLI to current exports only and exit with clear messaging when extension is absent.

### Open Questions

- Which kernel ships first from Rust, and what minimal Python function should wrap it?
- How do we surface Rust geometry structs cleanly (direct PyO3 classes vs. thin Python facades)?
- How strongly do we version/guard Shapely compatibility (minimum version, geometry kinds supported)?
- Should the package ship without any console entrypoint once stabilized, keeping it import-only?
- Do we expose CRS metadata at all, or defer until kernels support it?

## Status Tracking (to be updated by subagent)

- **Latest completed task:** _Added Rust-backed `Point` handle with Python wrapper validation and updated the stub/export surface._
- **Next up:** _Ship optional Shapely conversion helpers and verify packaging/CLI alignment._

## Lessons Learned (ongoing)

- _Match ambition to available kernels; keep wrappers aligned with Rust structs rather than speculative Python-only models._
- _Documenting the intentionally small Python surface early prevents overpromising and keeps stubs honest about what exists today._
- _Keep public exports tiny so packaging and documentation stay credible during early development._
- _Guard extension imports and fail loudly but politely when kernels are absent._
- _Interop helpers must be explicit and optional to avoid dragging in heavy dependencies by default._
- _Validate coordinate ranges in Python before constructing Rust handles to keep the extension surface minimal._
