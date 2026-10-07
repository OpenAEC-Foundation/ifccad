# Standalone OCDraw conversion coverage

The complete typed `OcdrawDocument` is the logical conversion boundary.
Logical export returns it with the existing loss/mapping/accuracy evidence;
encoded export delegates to the core encoder. Raw document import validates
before construction; validated reader import uses the same implementation.
These are fresh conversions: no identity-preserving CAD editing session,
watermark reconstruction or new source coverage is implied.

Dependency: opencadcodec revision
`063c10671fe7833d562f772159771318c7a0ebb9`, as pinned in Cargo.toml. The current
development worktree selects the explicit local patch; Cargo.lock reflects that
selected path dependency rather than an unmodified upstream resolution.

The source public-model classification in FROM-CAD-COVERAGE.md defines the pinned source classification. OCDraw is read through typed validated records;
encoding metadata and raw JSON are outside the CAD conversion boundary.

| Area | Standalone behavior |
| --- | --- |
| Geometry | LINE, POINT, CIRCLE, ARC, ELLIPSE, planar LWPOLYLINE/POLYLINE2D, spatial POLYLINE/POLYLINE3D; placement and active bulges retained |
| Blocks | Shared local definitions, base points, nested instances, signed scale, rotation and oblique placement; ownership is separate from the definition reference |
| Order | Mixed entity families follow each scope's stored drawing order |
| Layers/appearance | Explicit layer defaults; per-entity ByLayer/ByBlock/explicit values; local layer IDs and drawing-wide entity IDs |
| Layout/plot | Model and named paper layouts, plot units/settings/styles and flags; no extra bootstrap layout, including model-only sources |
| Saved state | Named/current UCS, active tiled model view, model windows, paper canvas views |
| Paper viewports | Frame, camera, clip, render mode, enabled/locked status, frozen local layers; active Circle/full Ellipse/closed straight or bulged PlanarPolyline clips resolve in either order |
| Numeric accuracy | Hard tolerance, whole-curve checks and nested occurrence-space checks; structured assessment on both outcomes |
| Source coverage | Unsupported fields diagnosed, bootstrap/runtime scaffolding classified, inconsistent references rejected |
| Opt-in spline preservation | Every available Spline parameter/common carrier is stored; native common properties, owner/order, source-free readback and separately qualified fresh CAD restoration; no native spline geometry |

Perspective CAD viewport export awaits a calibrated pinned-codec fixture.
Viewport appearance overrides, authored shading quality, and viewport workspace
state have explicit target/source loss diagnostics where not supported.
Clip family and full-curve eligibility are shared core rules. Stored boundary
references are independent of activation, including dormant references. The
[remaining viewport-off repair](../../../patches/opencadcodec-viewports/README.md)
preserves the independent off-state semantics. Clipping and DXF degree-angle
mapping are upstream capabilities in this pin; DWG tests use a conventional
paper canvas. Historical clipping-loss evidence applies to the previous pin.
Xrefs, external content, attributes/arrays/dynamic blocks, widths, unsupported
entity families, opaque vertex identities, and extended source metadata are
not silently approximated. Losses may be allowed with evidence or rejected.
The provisional standalone contract now has the bounded typed spline preservation
transport described in [SPLINE-SNAPSHOT-V1.md](SPLINE-SNAPSHOT-V1.md). Capture is
opt-in; unsupported extra source context may remain stored without safe restoration.
Other families and raw/private/shared codec-state restoration are not added.

Tiny block scales below the CAD setter threshold may fail conversion. The pinned
codec retains nonzero DWG block bases and anonymous names after the dependency
audit. Conflicting present markers remain fatal. The narrow stale DXF modelspace
cache recovery remains; extra-paper DWG marker ownership is still a structural
error, as described below.

Tests: ocdraw_conversion (typed production readback, mixed order, saved state,
oblique geometry and tolerances), ocdraw_block_exchange (actual DXF/DWG IO),
ocdraw_viewport_clips (clip policies and direct mappings), and
ocdraw_viewport_clip_exchange (patched DXF/DWG active/dormant clip chains), and
ocdraw_viewport_codec (literal degree angles and activation/reference decoding).

Spline tests additionally cover snapshots/shape validation, disabled versus enabled
capture, native edits/refusal/reference rebinding, and actual direct-vs-preservation
DXF/AC1032 DWG in Model/Paper/local blocks, unused definitions and repeated/nested
occurrences. Comparison by Paper layout identity does not assert equal reserved
block names or remove the additional-paper DWG marker boundary described below.

The ordered-scopes fixture preserves model/paper/nested-block ownership and
order through direct CadDocument, actual DXF and actual DWG. Its single named
paper layout reuses the reserved primary paper block. Additional authored
paper layouts still expose conflicting DWG BLOCK marker ownership for
`*Paper_Space0`: its name is retained, but the pinned codec returns the primary
paper record owner on that additional paper block. DXF preserves these layouts;
DWG return is rejected as inconsistent source structure. The layout regression
in `ocdraw_conversion` records both behavior and this remaining boundary.

Fractional and negative XY/XYZ polyline coordinates pass strict native
production readback and direct conversion. Fractional planar polylines also
have actual DXF/DWG readback. Ordinary codec-created 3D vertex handles and empty/default vertex layers are
accepted as scaffolding; semantic vertex properties retain their source classification.

## Named line patterns

Drawing-local definitions retain name, description, ordered simple lengths and
unused records. Layers, explicit entities and native viewport overrides refer to
local IDs; ByLayer/ByBlock remain modes. Drawing/entity scales and planar/spatial
generation flags survive supported CAD conversion. The core has no CAD dependency.

Text and shapes are not natively represented. Allow replaces the complete complex
pattern with an empty sequence, preserving named identity and references, and
reports one ComplexLinePatternFallback per definition (including unused ones).
Reject returns no drawing. No source restoration is implied by the retained name.
Broken references and invalid simple definitions are errors under both policies.

The CAD importer retains the fresh document's Continuous scaffold when the source
has no such named definition. Consequently CAD return conversion can add that
unused target-required definition. ByLayer/ByBlock scaffolding never becomes
ordinary OCDraw definitions. Real DXF/DWG tests cover custom fractional patterns,
named empty fallback, scales and continuous planar generation. Spatial generation
is retained in direct CAD conversion and DXF. The pinned DWG codec writes only the
closed bit for Polyline3D and loses continuous spatial generation; the physical
exchange test records this boundary. The inspector DWG download adds an explicit
DWG_SPATIAL_PATTERN_GENERATION_LOSS diagnostic per affected entity; direct
CadDocument conversion retains the flag. These tests do not
certify text/font/shape dependencies or every CAD viewport/annotation behavior.

## Upstream pin update — 2026-10-07

Both converters use opencadcodec `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
Clipping activation/group 340 and VIEWPORT angle units now come from merged
upstream PRs #88/#89; only the independent viewport-off repair remains selected.
References above to clipping/angle defects of the previous unmodified pin are
historical evidence, not limitations of this new base. The [current dependency
audit](../../../docs/geometry/opencadcodec-update-2026-10-07.md) records new public
fields and compatibility decisions. Native spline semantics remain absent;
new OCDraw snapshots use payload v2 and retain v1 read/restore support. Fit-only
spline DXF parameterization remains a target-codec limitation pending PR #99.
Unresolved source layer handles must not silently become a native layer 0.
Known resolved handles are relationship identity and are rebuilt from native
layer references. Canonical one-byte-per-INSERT count framing is derived;
unfamiliar/mismatched count storage retains loss evidence. Additional table,
associative/count and solid-history data remain at the existing unsupported
family boundaries. No benchmark evidence is extended by this update.
