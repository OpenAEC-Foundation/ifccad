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

Dependencies are recorded in `format-explorer/ocs-source.json`. Explorer adopts
the official [Document PR #7 merge](https://github.com/HakanSeven12/opencaddocument/pull/7),
`e6135a72ec46046cd23a84df1b13c8b4fe9541f2`, including upstream PRs #4/#5/#6.
It adds the seven remaining ordinary Dimension kinds. Its
[coverage contract](https://github.com/HakanSeven12/opencaddocument/blob/e6135a72ec46046cd23a84df1b13c8b4fe9541f2/docs/ifcx-dimension-kinds.md)
records qualified cases and explicit losses. CAD review of sample_AC1032 and a
nine-kind gallery was accepted by the user; that does not certify all host rendering.
This revision includes PR #7's line endpoint-identity correction: both roles
on the same source line retain distinct endpoints during Studio refresh.
Studio's grip/undo picture history remains a separate
[editing investigation](../dimension/editing-investigation-2026-10-29.md).

Studio base `fae5629888b0c16f820a8a176906a38508fa2a2b` is built with explicit
Document overrides in its main, save, worker and plugin API manifests. Both host
and Explorer select codec `75dd8f5cd3b5dd2c56d9c9d4e8c14b816d31d68d`; the host
selects native core `bce9da06a8e1684e86e35f0302e3870141e6f9f9`. Explorer's
workspace-root patch unifies the typed core with this checkout. Studio selects
matching kernel `faf2ee7ef3000ed65692cf8a65c134461ca06288` and graph
`5e4dc20c50d1280bed312598fb757ecf71513b54` through its frozen lockfile.
The reader requires profile 0.2.0/CBOR2. The host's development core pin has
identical model/IO, schema and conformance sources to native release 2026.44.1;
its earlier package version does not make it the published 2026.44.0 reader.
The native release tag and artifacts remain unchanged by this application update.

The Document merge changes only dependency manifests/lock relative to the
reviewed PR revision `4723b0e`. The codec update also exposes additional header
settings. Explorer supplements upstream losses for nondefault values of
dimension text direction, alternate factors/suffixes, shadow mode, orthographic
UCS origins, legacy dimension fitting/units, UNITMODE, Paper UCS base and the
current dimension text/line-style names, which the mapping does not carry as
drawing defaults. Both input and physical readback use the same diagnostic.
Codec maintenance/context framing flags are transport state; parsed MText end
properties are derived from existing content consumed by the shared formatting
parser. New Arch classes and unsupported spline/context families remain outside
the native conversion profile; adopting a codec does not add native semantics.

`crates/viewer/tests/document_examples.rs` checks strict physical DWG/DXF
roundtrips of the entity catalogue and house: all nine catalogue kinds,
measurement values, owner names, units and presence of saved pictures. Its
explicit external-practice check exercises sample_AC1032's eleven dimensions;
set `IFCCAD_REVIEW_SAMPLE_DWG` and run that ignored test deliberately. These
fixture budgets and saved-picture presence are not visual equivalence or a
general numerical accuracy certificate.

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

Native release 2026.44.1 uses logical profile 0.2.0 and CBOR mapping 2;
Zstd framing stays at 1 with inner mapping 2. This dependency adoption does not
advance the native format or create another native release.
Open issues #4 (splines), #12/#18 (accuracy), #20
(evaluation API) and #6 (encoding research) overlap in evidence boundaries and
remain separate work. No automatic size/exchange measurement is authorized.

## Extrusion source pins and native-only release — 2 November 2026

Native source alpha 2026.45.0 publishes profile 0.3.0 / CBOR mapping 3, optional
Zstd transport 1. Its initial Explorer integration selected official Document PR #8 merge
`28f1ad1214816e6b5c21162b732df0604bdf38bd`, codec
`0a5542baa6052a2b07fc700b7ba041549650a997`, and compatible host core
`07e1ef1c4e975afcab0a07012bed8a82a56202cb`. These exact source pins replace the
sibling development patch. The codec adoption was audited against 75dd8f5;
primitive/width fields are unchanged. New dictionaries/group helpers, ACIS
explode/surface fixes, DWG sharding/size fixes, named plot-style handles, MText
attachment precedence and malformed Hatch handling do not expand claimed
conversion coverage. See the [extrusion contract](../geometry/extrusion-thickness.md).

The user accepted the synthetic native/DWG/DXF control files and the separate
local Explorer preview, including explicit classic POLYLINE width-dependency
losses. Native Defaulted widths remain exact in native storage; CAD output
materialization is diagnosed. The catalogue primitives have thickness 15 mm and
base Z=0. The separate gallery remains local review material, not a shipped
example. The initial host lacked the Circle/Arc display fix from
[OCS #2361](https://github.com/HakanSeven12/OpenCADStudio/issues/2361); the current
upstream set below supersedes these application pins.

The native attached artifacts exclude applications. The separately authorized
Explorer update includes signed-thickness inspection and is built and deployed
through its own checked browser/host pipeline. General rendering and editing-history
fidelity remain unassessed beyond the recorded bounded local smoke.

The host kernel pin 3a77b22 aligns its codec with Document. Compared with
faf2ee7, it also includes the upstream periodic-rim seam fix in B-rep meshing;
this dependency adoption does not claim additional native CAD coverage.

## Current upstream Explorer set — 2 November 2026

The separate Explorer follow-up selects OCS 2026.44 source
`3d2c1bf45a836959865e0a9ec18bbec606d29a11`, Document
`3a3676b12e824292bcf46b29f4eaf02d9264c3a4`, codec
`ef27529c868a81bac0872e9260f597a2c6f85665` and kernel
`b2656bdbd4ac4c2c66043d52ad91e9e5f23106d2`. Host native core remains compatible
07e1ef1. These are the current main revisions inspected for this update, frozen
rather than followed automatically. Native release/tag/assets 2026.45.0 are not
replaced or republished by this application update.

OCS includes circle/arc extrusion wire fix 4a637fdbe for closed issue #2361. The
previous fae56298 host lacked it. Qualification must exercise the actual new
host preview, not infer a rendering pass from the Document thickness values.

Compared with the PR #8 merge, Document only advances its codec pin. The codec retains
primitive/width model fields and fixes serialized CAD groups, including existing
Dimension flip flags (already mapped both ways) and MText column transport for
older DWG through existing column semantics. TABLE override helpers do not add
native TABLE coverage. ProxyObject.dxf_data is outside the active opaque capture/
restoration coverage; preservation remains unavailable. The kernel bounds the
finest B-rep mesh retry and adjusts centerline selection/span behavior. These do
not introduce native annotation contexts, new file semantics or general editing
fidelity. Existing conversion losses/assessment limits remain applicable.

## Annotation development candidate

The native/browser candidate is based on native main `908aed6b` and selects
Document development commit `6fcede024604a91a8490bafaf7e2b99467eca7aa`
from `BjornFidder/opencaddocument`. An explicit root Cargo patch selects
`BjornFidder/opencadcodec` commit `4018037daf8fd3e40aa1aa6455b80232849a2756`,
based on upstream `7e7940f6fb17419629958ceeb79dfbd2d954dabb` with the HatchView
soft-pointer correction. These are public development revisions, not releases.
The workspace substitutes its native core for Document's native dependency,
maintaining one model identity without local source-path patches.

The codec advancement adds TABLE/Field roundtrip support and fixes R12 deleted
entries and DIMASSOC intersection serialization. It does not change the selected
annotation fields; new TableTemplate/field semantics remain outside native
conversion coverage. No additional entity families are claimed.

The accepted local embedded-host bundle predates this rebase and retains its own
recorded source/configuration and asset hashes. The current host metadata and
frozen lock still require coordinated integration before production deployment;
the native gate does not qualify that separate host build.

The local annotation contribution targets unpublished profile 0.4.0 and CBOR
mapping 4; it has not advanced Explorer's deployed dependencies. The paired
shared Document/codec development route currently verifies unused named scales,
Text/MText saved contexts and Dimension sizing intent through strict native
readback and physical DWG/ASCII DXF/binary DXF. Equal numeric scale ratios retain
separate identities; explicit disabled object markers retain saved contexts;
nonempty memberships imply enabled only when an object marker is absent.
Text coordinates are converted through the entity frame, including oblique
placements. MText context direction magnitude and XYZ values are retained.
Derived context text extents are not a complete rendered enclosure. Object and definition paper-orientation markers are retained independently,
including disabled intent and explicit object Fixed. Dormant
column heights without a CAD file field and incompatible style flag/sizing
combinations produce explicit diagnostics. The user accepted grouped review files and the local catalogue. The final native
workspace audit, fmt, Clippy and tests pass on the pinned development pair; the
Explorer dependency guard and frontend tests also pass. This is source-candidate
qualification, not a native release or production Explorer deployment.


The local annotation candidate also maps Hatch scale and view collections as
separate saved records, preserving repeated/null targets and independent defaults.
Bounded named VIEW cameras and model-window/Paper-viewport targets retain identity.
Block references retain context WCS positions, rotations and signed XYZ scales;
canonical base transforms remain independent. Native inherited mode with nonempty
memberships materializes its current effective CAD bool with an explicit loss of
future inheritance. Independent context planes are refused. Hatch local-context
boundary source IDs have no CAD context handle field and diagnose that loss.
All nine Dimension kinds now retain contextual XYZ points, dormant layout hints
and independent pictures through physical CAD and strict native readback.
ArcLength uses the Angular context mapping under the accepted SDK-based assumption;
an independently authored multi-scale ArcLength source remains unavailable.

Saved drawing/layout policies retain explicit false and absence separately:
ANNOTATIVEDWG and MSLTSCALE use AcDbVariableDictionary DICTIONARYVAR records;
Model CANNOSCALE and ANNOALLVISIBLE are independent of Paper AcadAnnoAV visibility.
Paper current scale uses the unique overall ID1 viewport, rather than LAYOUT's
active floating viewport. Floating viewport ASDK_XREC_ANNOTATION_SCALE_INFO and
named VIEW ADSK_XREC_VTR_ANNOSCALE_DATA retain separate scale choices. Model VPORT
windows share Model's scale. Small independent BricsCAD source files prove both
ANNOTATIVEDWG values; the setting is not a ModelSpace BTR annotation marker.
Nonzero MText context layout-extents caches are omitted with an explicit diagnostic;
saved rectangles and columns remain semantic state. The native candidate passes its full gate; official Document adoption and host
build/deployment qualification remain subsequent integration steps.

## Resource review candidate

The resource contribution builds on native annotation basis `100deb04`. The
viewer pins Document resource commit
`044bfcb7d096a50718cd15e9835477a23b813f1f`, with unchanged codec `4018037d`.
Document pins native resource commit `0e8f5e6175c236f0bbef6466da7dd1aff5834045`;
the workspace uses its own identical native core model. No local dependency
path override is required. These are development Git revisions, not releases.
It targets unpublished profile 0.5.0 / CBOR5 and preserves exact STYLE/SHX
requests, qualified face flags and separate same-named source records through
AC1032 DWG and ASCII/binary DXF production readback. Missing external resources
remain valid; broken internal STYLE roles fail structurally. MText grammar
limits are diagnosed or refused without changing a selected name.

Review uses one compact gallery and sample_AC1032. Existing unsupported sample
entities remain diagnosed; resource equality does not imply lossless geometry
or rendered equivalence. Backend request inspection and the snapshot report
adapter are tested separately. Deployed Explorer, embedded host and UI are not
yet integrated with this candidate. Gallery and sample review is accepted;
Explorer integration and deployment qualification remain separate stages.
