# Explorer Document migration coverage

The active browser route is CAD → opencaddocument Document → logical
IfccadDocument → production core IFCX writer/reader. Export reverses the typed
mapping before optional CAD serialization/readback. The core never imports a CAD
runtime. Browser and CLI use explicit `document_*` viewer entry points. The old viewer
conversion APIs and CAD helpers have been retired; native validation and source
inspection remain core-only. The browser dependency audit rejects any dependency
on `ifccad-convert` or its reference codec.

| Claim | New active route | Boundary |
| --- | --- | --- |
| Native/source validation | Production core, unchanged | Rendering is independent |
| Native bytes/fragments/composition | Original JSON retained | CAD generations are generated source; binary is decoded projection |
| Compact storage | Core CBOR/Zstd + strict readback | Binary excludes foreign IFCX data |
| CAD semantic mapping | Shared Studio mapping and upstream losses | Coverage differs from reference converter |
| Located losses | Original messages/optional handles retained | Most upstream losses are aggregated and unlocated |
| Reader recovery | Backend-default read and notifications | No reference dictionary audit or entity-local recovery qualification |
| Numerical accuracy | No automatic parameter quantization; runtime error bounds unassessed | No old tolerance intervals, hard-error bounds or numeric guarantees |
| Spline preservation | Unavailable | Source splines may be reported/omitted, no opaque capture guarantee |
| Partial opening | Unavailable | Explicit requests rejected |
| Reject policy | Unavailable | Explicit requests rejected; loss detection is incomplete |
| Physical CAD readback | Real reader/version check + strict native projection | Entity kinds/counts only; complete semantic fidelity unassessed |
| IFCX host selection | Explicit typed import maps, exact owner/layout and current session checks | No physical serialization/readback needed; excludes shared block contents |
| Host rendering | Unassessed | Known foundation dimension-text observation remains unresolved |

The [pinned mapping](https://github.com/HakanSeven12/opencaddocument/blob/8a31478370c1b877c7bcb42c5298d033354ad4e4/docs/ifcx-numerics.md)
writes directly calculated placement origins/axes and derived binary64 values
without automatic `snap`, `settle` or OCS-parameter quantization. Representable
signed INSERT scales are retained; setter substitutions are diagnosed rather
than silently changing magnitude or handedness. Dimension pictures use shared
frame validity and finite arithmetic, without the former residual/extent budget.
This removes avoidable quantization; binary64 rounding, overflow, unsupported
content and diagnostic coverage remain separate limitations. Upstream's bounded
3D/nested/unit/transport regression budgets are test-specific evidence, not
runtime tolerances or general source-to-native error certificates. Explorer's
tilted-circle regression checks actual DXF import, strict production native
readback and CAD export/readback without borrowing the archived converter's proof.

Stored entity families and core inspection have not changed. The shared mapping
covers tables, blocks, paper layouts/viewports, workspace, primitive/path/text,
Hatch, SOLID/TRACE and supported Dimensions/pictures within its own qualified
profiles. This table does not assert parity with the reference converter's
source-detail audits, graphics eligibility, association handling, color identity,
polyline width dependencies, Hatch file restrictions or physical setter checks.
Those old contracts remain reference contracts, never implicit new-route claims.
The new route suppresses reference export preflights instead of attributing their
codec qualification to another codec. Actual upstream notices/losses are shown.

Preview consumes the exact validated snapshot supplied by opening. IFCX download
uses that snapshot too. Downloads do not reconvert the original CAD; output-only
settings do not replace inspected native content. Original/dirty generated Studio
tabs survive replacement. The host receives real `.ifcx` bytes and imports them
internally into Document. Native validity, conversion losses, numerical proof,
physical CAD readback and host rendering remain distinct claims.

Dependencies are recorded in `format-explorer/ocs-source.json`. The development
candidate adopts [Document PR #7](https://github.com/HakanSeven12/opencaddocument/pull/7),
`ad6fec3f99dae5f7f0e93aca8ef5e4c98ca680ea`, including upstream PRs #4/#5/#6.
It adds the seven remaining ordinary Dimension kinds. Its
[coverage contract](https://github.com/BjornFidder/opencaddocument/blob/ad6fec3f99dae5f7f0e93aca8ef5e4c98ca680ea/docs/ifcx-dimension-kinds.md)
records qualified cases and explicit losses. CAD review of sample_AC1032 and a
nine-kind gallery was accepted by the user; that does not certify all host rendering.

Studio base `4d1cd17dafd7d47cd76f8674835636776eb90fd6` is built with explicit
Document overrides in its main, save, worker and plugin API manifests. Both host
and Explorer select codec `501b694eee3433b157e56a32da4268dc8b3b5bd3`; the host
selects native core `bce9da06a8e1684e86e35f0302e3870141e6f9f9`. Explorer's
workspace-root patch unifies the typed core with this checkout. The development
reader requires profile 0.2.0/CBOR2; the fixed published alpha reader is separate.
No live deployment is implied by these candidate pins.

`scripts/prepare_explorer_ocs.py` applies the overrides and frozen host lockfile.
The host cache identity includes those inputs, workflow/build configuration and
Studio revision. `IFCCAD-BUILD.json` records the actual main/worker assets and
resolved dependency identities. Reused bundles are checked against this record;
the Explorer build rejects mismatched reader pins or asset hashes. Explorer WASM
is rebuilt separately. Exact host selection still requires successful host-side
selection confirmation. Package-name agreement alone is insufficient evidence.

The [retired viewer route](retired-viewer-converter.md) records removed APIs,
helpers and route-specific tests, retained native core coverage, and recovery
of the former reference implementation from Git history.

The candidate advances the active logical profile to 0.2.0 and CBOR mapping to 2;
Zstd framing stays at 1 with the new inner mapping. This is not a software release.
Open issues #4 (splines), #12/#18 (accuracy), #20
(evaluation API) and #6 (encoding research) overlap in evidence boundaries and
remain separate work. No automatic size/exchange measurement is authorized.
