# OCDraw parking and IFCCAD ownership

IFCCAD is the sole active track. The root package is `ifccad`; public imports move
from `ocdraw::ifccad` to `ifccad`. The internal model stays under `src/ifccad`.
No legacy compatibility feature, parity requirement or active OCDraw build remains.
The Explorer/CLI defaults to IFCCAD; explicit old routes/files report `OCDRAW_PARKED`.

Neutral kernels remain in the active core, without CAD runtime dependencies.
Unicode name equality uses an active internal helper and its frozen Unicode-17
source/license. Coordinate-unit validation uses the neutral enum, preserving the
same 25 tokens. CAD companions and `ifccad-convert` were later archived separately; see
[converter parking](converter-parking.md).
No new abstraction or renamed generic model is introduced to support legacy.

OCDraw-specific model, converter, schemas, candidate fixtures, tests and generators
are under `legacy/ocdraw`. Mixed tests preserve IFCCAD and general kernel/UI
assertions actively and preserve complete originals in the archive. The
[manifest](../legacy/ocdraw/FREEZE.json) records checkpoint, exact source hashes,
test mapping, dependency base/patch, lock, tools and evidence. Restore the complete
checkpoint as described in [the archive README](../legacy/ocdraw/README.md),
including its old kernels and adapters. The partial source archive is not buildable.

Ordinary `--workspace` checks now exercise only IFCCAD and its active companions.
`scripts/check_active_workspace.py` rejects parked package names, legacy path
dependencies and active Rust references to parked fixtures/APIs. CI separates the
default browser processor from optional native Zstd qualification. Cache namespaces
are changed for the new graph. Historical reports and numbered conformance remain
byte-for-byte at their original paths. No measurements are rerun automatically.

The accepted CBOR end gate completed before this task. CBOR mapping revision 2,
optional Zstd transport revision 1 and JSON semantics/bytes are unchanged by parking.
There is no CBOR UI expansion. Current main integration still requires a separate
explicit instruction, and verification is tied to the final candidate's inputs.

Hatch B remains paused until this migration has been integrated. Then update it
once onto that basis, replace the old codec pin and remove dual-model obligations;
review changes against IFCCAD and CBOR mapping before resuming. Dimension graphics
follows the stable dependency boundary. Keep xformop separate. Do not rebase or
modify these active worktrees during migration. Avoid concurrent full workspace
runs; develop with focused tests and qualify the final integration candidate once.
