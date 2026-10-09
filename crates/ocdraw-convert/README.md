# ocdraw-convert

`ocdraw-convert` connects standalone OCDraw to opencadcodec's `CadDocument` through
`cad_document_to_ocdraw_document` and `ocdraw_document_to_cad_document`.
The encoded `cad_document_to_encoded_ocdraw` and validated `ocdraw_source_to_cad_document`
entry points are convenience wrappers over logical conversion. These implementations
live in [`src/from_cad`](src/from_cad) and [`src/to_cad`](src/to_cad),
with native mapping helpers under [`src/mapping`](src/mapping). Pure CAD source classification lives in
[`src/source`](src/source). This direction/mapping/source organization is shared with
`ifccad-convert`; each retains its own format adapters. The model adapter in
[`src/mapping/geometry.rs`](src/mapping/geometry.rs) uses the common
[`cad-geometry-convert`](../cad-geometry-convert) numerical and occurrence proof engine.
Format-specific tolerance domains and report adaptation live in
`geometry_context.rs` and `geometry_assessment.rs`. There is no local numerical
geometry implementation or forwarding `geometry/` module.
Both formats use [neutral core geometry](../../docs/geometry/shared-geometry.md).
The core format implementation remains usable without opencadcodec.
CAD presentation decisions and strict XRecord grammar use the ID-free
cad-presentation-convert companion. Native IDs, reference binding and diagnostics
stay in this adapter. See [presentation coverage](../../docs/presentation.md),
including qualified perspective Paper cameras and the remaining grid/ShadePlot
transport gates.

The dependency and public Rust reexport use the upstream name `opencadcodec`
at a shared fixed revision with `ifccad-convert`. The
[dependency audit](../../docs/geometry/opencadcodec-update-2026-10-05.md) records
the current public-model classification and exchange fixes.

## Standalone conversion

```rust,no_run
use ocdraw::ocdraw::load_ocdraw_bytes;
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, ocdraw_source_to_cad_document, CadToOcdrawOptions, OcdrawToCadOptions,
};
use ocdraw_convert::opencadcodec::CadDocument;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let source = CadDocument::new();
let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default())?;
let drawing = load_ocdraw_bytes(exported.encoded().bytes())?;
let imported = ocdraw_source_to_cad_document(&drawing, OcdrawToCadOptions::default())?;
for diagnostic in imported.diagnostics() {
    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
}
let cad_document = imported.into_document();
# let _ = cad_document;
# Ok(())
# }
```

The converter consumes typed `OcdrawDocument` content. Raw input is validated
before CAD construction; a reader snapshot already carries that guarantee. It does not inspect
JSON columns or construct JSON. Logical export returns a document and the same loss, mapping and accuracy
evidence without encoding. The encoded wrapper uses the core encoder and strict
production readback. Filesystem storage is a
separate application responsibility.

The standalone route supports modelspace, paperspace, shared local blocks,
lines, oriented points, placed circles/arcs/ellipses, placed planar polylines
including bulges, direct XYZ spatial polylines, signed block transforms, layers,
appearance choices, named simple line patterns and scales, layouts/plot settings, named UCS definitions, current UCS,
model windows, paper canvases and paper viewports. Source order is retained
across supported entity families. An open polyline's dormant final bulge is
stored without treating it as an active segment.

Active paper clips accept circles, full ellipses and closed planar polylines,
including bulges and supported classic POLYLINE2D sources. Shared OCDraw validation
checks the entire boundary in paper Z=0 within the frame, including tangent
curves. Both conversion directions resolve clip references independently of
scope draw order. Missing, unsupported or conflicting active boundaries skip
the whole viewport under Allow and receive loss evidence; Reject refuses them.
Active and dormant boundaries retain their reference independently of activation.
Only active clips require the core's family/plane/frame eligibility. Missing,
wrong-scope, skipped or conflicting stored references skip the dependent viewport;
an active clip without a boundary is diagnosed rather than turned into a rectangle.

The [explicit local codec patches](../../patches/opencadcodec-viewports/README.md)
are selected in the current development worktree. Patched DXF/DWG roundtrips
retain clipping activation, hidden boundaries and either draw-order position;
DWG qualification includes a conventional paper canvas. Literal DXF angle tests
check degrees at the file boundary and radians in CadDocument. The unmodified
base still strips activation and DXF group 340, so patched results do not qualify
it. Explorer DXF downloads emit `DXF_VIEWPORT_CLIP_LOSS` when that base is selected.

Complex text/shape line patterns retain their names, descriptions and local
references but become continuous under Allow, with one loss diagnostic per
definition. Reject refuses this fallback, including unused complex definitions.
Simple unused records, entity scales and polyline generation are retained.

Conversion diagnoses unsupported source properties and target limitations;
`Reject` rejects semantic loss. Numerical accuracy is a separate hard limit.
The default is exactly 1e-9 in each domain's own coordinates. Exact conversion is
explicitly available. A physical tolerance requires known physical meaning unless
`with_coordinate_fallback(value)` supplies an independent limit for unknown domains. Assessment
covers nested block occurrences as well as definition-local geometry; scale
can amplify local rounding. Outcomes expose `geometry_assessment()` and
source/target entity mappings.

See [standalone coverage](docs/COVERAGE.md) for scope and limitations, and
[block codec limits](../../docs/geometry/block-cad-boundary.md) for upstream
DXF/DWG restrictions. Conversion through the real pinned DXF and DWG readers
and writers is tested without dependency patches.

## Opt-in spline preservation

Set `CadToOcdrawOptions::preservation_capture` to
`OcdrawPreservationCapture::SupportedTyped` to capture every available Spline
variant in ModelSpace, PaperSpace and supported local block definitions, without
a degree/knot/weight/flag/fit profile filter. Disabled remains the default. The
core stores generic records and opaque entities with optional exactly representable
native layer/appearance and ordinary visibility. All remaining source common data
is retained in the snapshot, including fields skipped by upstream Serde.

Source parameters survive encode, closing the source instance and strict production
readback. They have no native evaluator, certified bounds or rendering claim.
`preservation_report()` distinguishes capture, native capability and restoration;
`geometry_assessment().is_complete()` and `unassessed_sources()` expose opaque
definition/occurrence content without weakening native tolerance gates.

`OcdrawToCadOptions::preservation_restore` defaults to RestoreSupported. Eligible
splines restore into a fresh CadDocument with current native common properties and
scope order. Unknown/unqualified context receives located evidence. Skip is
explicit loss under Allow; Reject refuses omitted live content. Unknown payloads
remain core-transportable, and detached archives do not create phantom entities.
LossRejected errors retain the preservation report. Full original records,
raw/private/shared codec-state replay and other providers are outside this slice.
See [payload/field/qualification contract](docs/SPLINE-SNAPSHOT-V1.md).

## Logical conversion without serialization

```rust
use ocdraw::ocdraw::{encode_ocdraw_document, validate_ocdraw_document};
use ocdraw_convert::{cad_document_to_ocdraw_document, ocdraw_document_to_cad_document,
    opencadcodec::CadDocument, CadToOcdrawOptions, OcdrawToCadOptions};
# fn example() -> Result<(), Box<dyn std::error::Error>> {
let exported = cad_document_to_ocdraw_document(&CadDocument::new(), CadToOcdrawOptions::default())?;
let drawing = exported.document();
validate_ocdraw_document(drawing)?;
let cad = ocdraw_document_to_cad_document(drawing, OcdrawToCadOptions::default())?;
let bytes = encode_ocdraw_document(drawing)?; // Only when native storage is wanted.
# let _ = (cad, bytes);
# Ok(())
# }
```

These are fresh conversions. CAD handles, OCDraw IDs and allocation history are
not roundtripped through `CadDocument`. A future CAD editor save route needs
explicit session context. See the [core lifecycle](../../docs/ocdraw-document-lifecycle.md).

## Layout output revision

Both models retain layout media without complete plot settings. Plot unit and
fixed mapping determine Paper output meaning; IFCCAD no longer stores an independent
Paper coordinate unit. Effective plot settings, limits and layout PSLTSCALE have
separate native/CAD coverage. The provisional field/API migration, strict physical
scalar conversion limits, raster restrictions and per-domain accuracy reports are
specified in [layout output](../../docs/layout-output.md). The later workspace slice is documented separately; no renderer, release or controlled measurement is implied.

## Upstream pin update — 2026-10-07

Historical note: this records the previous base. The 2026-10-08 update below supersedes its pin and remaining spline/paperspace limitations.


Both converters use opencadcodec `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
Clipping activation/group 340 and VIEWPORT angle units now come from merged
upstream PRs #88/#89; only the independent viewport-off repair remains selected.
References above to clipping/angle defects of the previous unmodified pin are
historical evidence, not limitations of this new base. The [current dependency
audit](../../docs/geometry/opencadcodec-update-2026-10-07.md) records new public
fields and compatibility decisions. Native spline semantics remain absent;
new OCDraw snapshots use payload v2 and retain v1 read/restore support. Fit-only
spline DXF parameterization remains a target-codec limitation pending PR #99.
Unresolved source layer handles must not silently become a native layer 0.
Known resolved handles are relationship identity and are rebuilt from native
layer references. Canonical one-byte-per-INSERT count framing is derived;
unfamiliar/mismatched count storage retains loss evidence. Additional table,
associative/count and solid-history data remain at the existing unsupported
family boundaries. No benchmark evidence is extended by this update.

The codec-specific spline byte DTO now lives in the model-independent
[`cad-preservation`](../cad-preservation) companion. OCDraw envelopes, conditions,
references and restoration stay in this adapter. Payload v1/v2 bytes and the
public `OcdrawSplineSnapshotError` alias remain compatible; extraction introduces
no codec dependency into either core and does not change the active pin.
## Workspace and upstream update — 2026-10-08

Both converters select opencadcodec 0.6.0 at `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` plus the explicit viewport-off repair [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). Merged spline DXF parameterization and DWG Paper owner/overall-role repairs now come from upstream. See the [dependency audit](../../docs/geometry/opencadcodec-update-2026-10-08.md) and [workspace contract](../../docs/workspace-state.md) for the current field/transport boundary.

UCS definitions, Model windows, canvas frame/grid/snap/UCS and authored Paper viewport aids map through shared ID-free scalar helpers. Native identities, ownership, source classification and located losses stay format-specific. Disabled zero snap spacing and stored-UCS activation survive independently of current choices. Model-viewport aids retain Model coordinates inside Paper; canvas aids use Paper coordinates. Canvas frames do not enter geometry bounds or clip ownership.

A uniquely available active Model window may be selected; multiple unqualified windows survive with unspecified activation. Paper current viewport/UCS association remains unavailable on the codec surface. Model/Paper mode and the active Paper tab are retained through the unique reserved *Paper_Space block and its consistent LAYOUT association, including multiple sheets. Export synchronizes BLOCK_RECORD names, existing BLOCK begin names and the reserved header handle together; setting the header cache alone does not change the active role. Unknown choices remain omitted with located loss. Skipped viewports receive no workspace references.

Dot grid style and frequencies beyond CAD i16 receive field-specific substitutions. VIEWPORT grid beyond-limits/adaptive/subdivision/follow-workplane flags stay in CadDocument but its pinned DXF/DWG routes do not retain them; target diagnostics identify each nondefault field and Reject refuses that portability loss. Model VPORT grid flags survive both routes. Unrepresented display/icon/base/orthographic/plot/visual state remains diagnosed. Numeric and structural failures remain fatal under both policies.

## Hatch

Native Solid and LinePattern Hatch map stored contours and the normal/outer/ignore area rule.
Per-loop single same-owner source references bind after entity allocation.
Unsupported associations retain contour geometry with located loss.
`hatch_join_tolerance` is a CAD-to-native creation option, separate from numeric
conversion tolerance. Nondefault native limits are not persisted in CAD and
produce rejectable loss. Fill remains unassessed despite contour curve proofs.
Explicit pattern families retain phase, signed spacing and dash/gap/dot order.
CAD definitions are evaluated once; qualified continuous UserDefined and a narrow
double profile are supported. Other active dependencies omit the whole Hatch.
See [Hatch support](../../docs/hatch.md) and the coverage inventories.
