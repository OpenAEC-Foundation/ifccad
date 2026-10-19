# Parked OCDraw source reference

OCDraw is frozen at the complete checkpoint recorded in [FREEZE.json](FREEZE.json).
It remains provisional and unpublished. This directory preserves source, schemas,
fixtures, tests and selected original adapters. It is excluded from active Cargo,
Explorer assets and routine verification. It has no standalone root Cargo manifest:
old converter manifests are reference source, not a supported build entry point.

Shared kernels are maintained for IFCCAD. The old model must not silently compile
against those evolving kernels. Restore the **complete** freeze checkpoint in a
separate checkout instead. Copy `reproduction/Cargo.lock.snapshot` to `Cargo.lock`
and the portable config snapshot to `.cargo/config.toml`; clone the recorded codec
revision to the config's repository-local path and apply `selected-codec.patch`.
Then run the historical workspace gates for that checkout with its recorded tools.
No automatic benchmark or release operation is implied.

`archivedPaths` records original path, new path and raw Git-byte SHA256.
`splitTests` and assertion-review notes map mixed tests to retained active coverage.
Numbered conformance and accepted benchmark report/result pairs remain at their
original paths, indexed as historical evidence. They do not measure this migration.
See [the parking decision and worktree order](../../docs/ocdraw-parking.md).
