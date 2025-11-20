# Publishing pipeline (dry-run first)

- Workflow: `.github/workflows/release.yml` triggers on tags matching `vMAJOR.MINOR.PATCH` and defaults to dry-run.
- Gates: runs Python lint/tests (`make lint`, `make test`) and Rust fmt/clippy/tests before any packaging starts.
- Outputs in dry-run: cargo publish dry-run builds `geodist-rs` and uploads the crate tarball; `maturin build` builds wheels across Linux/macos/Windows (x86_64 + macOS aarch64) for Python 3.10 and 3.12, plus an sdist from Linux.

## Cutting a dry-run release

1) Ensure versions match: bump `geodist-rs/Cargo.toml` and `pygeodist/pyproject.toml` to the same value.  
2) Tag the release: `git tag -s v0.1.0 && git push origin v0.1.0` (re-sign if you prefer lightweight tags).  
3) Inspect artifacts in the run named `Release Publishing (dry-run default)`:
   - `geodist-rs-crate`: cargo-produced `.crate` tarball (from `cargo publish --dry-run`).
   - `python-wheels-*`: platform wheels and the Linux-built sdist.
4) Optionally exercise a wheel locally by downloading the artifact and installing with `pip install dist/<wheel>` to sanity-check metadata and importability.

## Flipping to live publish (gated)

- Default stays dry-run. Live uploads require **both** the repository variable `PUBLISH_LIVE=true` and secrets seeded:
  - `CRATES_IO_TOKEN` (crates.io API token).
  - `PYPI_API_TOKEN` (PyPI token). Swap to PyPI Trusted Publishing later by configuring OIDC on PyPI; the workflow already grants `id-token: write`.
- Repository must be public; the workflow aborts live publishing while private.
- Optional manual flip: `workflow_dispatch` input `publish_live=true` can enable uploads for a single run (still requires secrets + public repo).
- Live path: same tag trigger, artifacts still upload, and gated steps run `cargo publish --locked` plus `pypa/gh-action-pypi-publish` over the gathered wheels/sdist.

## Quick verification / rollback notes

- Verify versions with the preflight log (tag vs manifests) and check the dry-run `cargo publish` and `maturin build` outputs for warnings (warnings fail the jobs).
- If a live publish is started with missing secrets or a private repo, the workflow halts before pushing to registries with an explicit error.
- Roll back a mistaken live push by yanking/replacing the release on the registries; clear `PUBLISH_LIVE` or remove the tokens to restore the dry-run-only behavior.
