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
| Numerical accuracy | Unassessed, fixed upstream rounding | No old tolerance intervals, hard-error bounds or numeric guarantees |
| Spline preservation | Unavailable | Source splines may be reported/omitted, no opaque capture guarantee |
| Partial opening | Unavailable | Explicit requests rejected |
| Reject policy | Unavailable | Explicit requests rejected; loss detection is incomplete |
| Physical CAD readback | Real reader/version check + strict native projection | Entity kinds/counts only; complete semantic fidelity unassessed |
| IFCX host selection | Explicit typed import maps, exact owner/layout and current session checks | No physical serialization/readback needed; excludes shared block contents |
| Host rendering | Unassessed | Known foundation dimension-text observation remains unresolved |

The [pinned mapping](https://github.com/HakanSeven12/opencaddocument/blob/47d7888ca6ae1a13b80c8cf5076f5efdbc5a5f2b/src/ifcx/mod.rs)
uses selected position snapping at `1e-8`, selected angle snapping at `1e-12`,
and 40-significant-bit settling for some derived lengths. These are upstream
parameter rules, **not** a universal drawing/physical tolerance or a certified
source-to-native error bound. Unchanged values, rounded values and unassessed
geometric consequences must not inherit the reference converter's `1e-9` proof.

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

Dependencies are recorded in `format-explorer/ocs-source.json`. The typed API
was merged in [opencaddocument PR #4](https://github.com/HakanSeven12/opencaddocument/pull/4)
and is pinned to `47d7888ca6ae1a13b80c8cf5076f5efdbc5a5f2b`. The workspace-root
Cargo patch unifies its core model with the current IFCCAD workspace. Studio's
own historical Document/core pins are recorded separately. The codec change
between Studio's `68530d9` and Explorer's `eada4ff` only changes DXF spline write
flags; model construction and IFCX import allocation are unchanged. Exact host
selection still requires successful host-side selection confirmation.
Verification records dependency contents, not matching package names alone.

The [retired viewer route](retired-viewer-converter.md) records removed APIs,
helpers and route-specific tests, retained native core coverage, and recovery
of the former reference implementation from Git history.

No native contract identifiers, schema/conformance releases or storage
architecture change here. Open issues #4 (splines), #12/#18 (accuracy), #20
(evaluation API) and #6 (encoding research) overlap in evidence boundaries and
remain separate work. No automatic size/exchange measurement is authorized.
