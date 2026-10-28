# Dimension graphics mapping

Native model/validation/storage sections remain active. Direct `ifccad-convert`
exchange and codec qualification below describe the [archived reference](../converter-parking.md);
the [shared Document route](../architecture/explorer-document-route.md) has its own coverage.

Current follow-up: [native polyline widths](../geometry/polyline-widths.md) now extend
the historical zero-width boundary below. The original red source passes storage
acceptance and bounded unchanged-host comparison. Historical slice scope and earlier
evidence below do not override the current width contract.

Implementation contract, 21 October 2026. The active experimental IFCCAD profile
stores optional worked-out graphics; schemas and conformance remain authoritative.
This slice targets IFCCAD only. Polyline widths remain a separate follow-up,
with the red sample as its complete-picture acceptance case.

Implementation baseline: remote main commit `bdbcee1b24ebbf12504ec4d45d81a2eea2466bb1`
contains the verified [native primitive slice](../geometry/solid-trace-design.md).
The earlier SOLID/TRACE-to-Hatch substitution is superseded.

The native precursor is necessary but not sufficient for the red sample: its
`_ArchTick` marker has polyline width 0.15. The current PlanarPolyline profile
stores no width. This slice keeps that limitation explicit and uses supported
zero-width paths/native filled primitives for positive capture cases. No marker
width is zeroed or silently replaced. The
[separate width slice](../../ROADMAP.md#planned-polyline-width-follow-up) owns
constant/per-segment width support and the red sample's full acceptance.

This mapping preserves the agreed Dimension-owned child group in IFCX and uses
the same typed value directly in CBOR. It adds neither a Dimension graphics
generator nor source-graph editing. [Dimension support](../dimension.md),
[profile rules](../../schemas/ifccad/profile-rules-0.2.0.md),
[active CBOR mapping](../../schemas/ifccad/cbor-mapping-2.md) and the
[document lifecycle](../experiments/ifccad-document-lifecycle.md) are its basis.

## Logical records

`IfccadDimension` gains `graphics: Option<IfccadDimensionGraphics>`.
`IfccadDimensionGraphics` contains required `origin` and ordered `parts`.
`IfccadDimensionGraphicsOrigin` is closed: `SourceSnapshot` or `Generated`.
An absent value, an explicitly empty parts list and a nonempty list are distinct.
There is no validity flag, input basis, cached measurement or dependency snapshot.

CAD has no corresponding native `origin` field. Captured CAD pictures use
SourceSnapshot. Exporting Generated graphics can retain the complete picture,
but cannot retain that native provenance without a new extension. Under Allow,
report `dimension-graphics-helper-state` at `graphics/origin` and write the full
picture; Reject refuses this metadata loss. Do not relabel native data silently,
drop the picture, or invent XDATA to hide this boundary. A later CAD recapture is
a new SourceSnapshot. Native IFCX/CBOR roundtrips preserve either origin exactly.

Each `IfccadDimensionGraphicsPart` contains:

| Field | Meaning and default |
| --- | --- |
| `layer_id: Option<u64>` | None inherits the Dimension layer; Some refers to an existing local layer, including ID zero |
| `kind: IfccadDimensionGraphicsKind` | Required closed typed geometry/text payload |
| `appearance: IfccadEntityAppearance` | Existing four native appearance modes; default is ByLayer for every leaf |
| `visible: bool` | Default true; authored false survives |
| `line_pattern_scale: f64` | Default binary64 1.0; existing positive finite entity-scale rules |
| `purpose: IfccadDimensionGraphicsPurpose` | Display by default; ReferencePoint is permitted only with Point geometry |

ReferencePoint is a stored non-displaying helper point. It contributes no display
bounds and is never drawn by the ordinary point-glyph renderer. Its placement and
appearance remain stored. It is not a definition anchor, association or instruction
to update the measurement. In particular, the core does not compare it with the
Dimension definition. Display Points retain ordinary point-display semantics.

The new closed geometry enum reuses the existing value shapes for LineSegment,
Point, Circle, Arc, Ellipse, EllipseArc, PlanarPolyline, SpatialPolyline,
BlockInstance, Text, MText, Hatch, Solid and Trace. It does not wrap an unrestricted
`IfccadEntityKind`: Viewport, Dimension and opaque content are not legal parts.
The Hatch variant must have Solid fill, no gradient, association disabled and
empty source lists. Existing loop provenance and text-island state can survive;
they do not enable source updates. Existing geometry/closure validators still apply.

Parts have no ordinary entity ID or independent allocation counter. An index is
a draw position, not a persistent identifier. Resource blocks remain ordinary
identified definitions; their reachable contents must satisfy the same closed
graphics family, including the restrictions on Hatch and associations. Foreign,
external, dynamic or recursively cyclic marker contents are not eligible.

## IFCX child roles and paths

For drawing ID D and ordinary Dimension entity ID E, define
`P = /cad/dD/eE`. The existing owner still refers to P once in its draw order.
The new canonical nodes and role attributes are:

| Node | Path | Link and attributes |
| --- | --- | --- |
| Dimension | P | Optional `children.graphics` points to `P/graphics`; existing `ifccad::entity`, `ifccad::dimension` and placement are unchanged |
| Graphics group | `P/graphics` | Required `ifccad::dimensionGraphics: {"origin":"sourceSnapshot"}` or `{"origin":"generated"}`; required `children` object, empty or numbered from zero |
| Part at index I | `P/graphics/pI` | Group `children["I"]` targets this path; required `ifccad::dimensionGraphicsPart` and exactly one allowed payload attribute |

D, E and I use canonical unsigned decimal syntax without signs or leading zeros;
D and E retain their uint64 domains. I is a bounded list index, subject to reader
resource limits, not a new identity domain. There is no graphics ID or watermark.
The writer derives these paths from identity and order. Reordering parts can
rename their paths in fresh output; stable part paths across edits are not promised.

Both writer and reader use this canonical path layout. A graphics child cannot
target another Dimension's group or a different path. The group has only numeric
CAD child roles, contiguous `0..n-1`, with distinct targets at the matching pI
paths. A part has no further CAD children. There are no ordinary `ifccad::entity`,
opaque, layout, drawing, block-definition or Dimension roles on group/part nodes.
The group carries no placement, appearance or geometry.

These rules extend OWN-001/002 with a separate graphics ownership role; they do
not make arbitrary named children owned CAD entities. Nodes declaring the new
IFCCAD graphics roles must have exactly this ownership and canonical path.
Unlinked declared graphics nodes are structural errors. Existing genuinely
foreign nodes retain their existing source-retention/audit boundary and do not
become graphics because their paths or payloads look similar.

The single group path deliberately does not select a generation/version protocol.
Publishing alternate groups, retaining inactive known-role generations and editing
fragments require the later source-writeback design. No existing key-merge rule is
changed to make that future workflow appear available now.

## IFCX attributes and defaults

`ifccad::dimensionGraphics` is a closed Object with one required enum member,
`origin`. It has no `parts` member; child links own the ordered part nodes.

`ifccad::dimensionGraphicsPart` is a closed Object with the following members:

| JSON member | Physical value | Presence |
| --- | --- | --- |
| `layer` | Reference to `/cad/dD/layer/L` | Optional; omission means inherit Dimension layer |
| `appearance` | Existing native appearance object, with complete local line-pattern paths | Optional; same per-leaf ByLayer defaults as ordinary native entities |
| `visible` | Boolean | Optional, default true |
| `linePatternScale` | Real | Optional, default positive binary64 1.0 |
| `purpose` | Enum `display` or `referencePoint` | Optional, default display |

The marker attribute itself is required even when its value is `{}`. Null and
unknown members are rejected. These are physical defaults, not sparse Dimension
style overrides: existing style/override presence semantics are unchanged.
There is no redundant kind discriminator in this JSON object; the single payload
attribute selects the kind. A part uses direct registered attributes, not general
IFCX `inherits` evaluation or implicit child transform inheritance.

| Part kind | Payload attribute | Placement on the same node |
| --- | --- | --- |
| LineSegment | `ifccad::geom::lineSegment` | Forbidden; endpoints are direct Dimension-local XYZ |
| Point | `ifccad::geom::point` | Required `ifccad::geom::placement` |
| Circle / Arc | `ifccad::geom::circle` / `ifccad::geom::arc` | Required |
| Ellipse / EllipseArc | `ifccad::geom::ellipse` / `ifccad::geom::ellipseArc` | Required |
| PlanarPolyline | `ifccad::geom::planarPolyline` | Required |
| SpatialPolyline | `ifccad::geom::spatialPolyline` | Forbidden; vertices are direct Dimension-local XYZ |
| BlockInstance | `ifccad::blockInstance` | No separate placement; its existing transform contains it |
| Text / MText | `ifccad::text` / `ifccad::mText` | Required |
| Solid Hatch | `ifccad::hatch` | Required; restricted logical value described above |
| Solid / Trace | `ifccad::geom::solid` / `ifccad::geom::trace` | Required; shared IfccadFilledQuadrilateral value from the preparatory slice |

The payload members, reference syntax, angle units and physical defaults are the
existing ones. In particular, ordinary compact XY line endpoints mean XYZ with
positive-zero Z; they are not a general planar default for other kinds. Text and
Hatch placements are not also embedded as a second JSON geometry placement.
Block, TextStyle and line-pattern references remain complete same-drawing paths.

The following is an illustrative node fragment, not a complete drawing and not
a file supported by current readers. Entity e102 and layer 0 must already exist:

```json
[
  {
    "path": "/cad/d1/e102",
    "children": {"graphics": "/cad/d1/e102/graphics"}
  },
  {
    "path": "/cad/d1/e102/graphics",
    "attributes": {"ifccad::dimensionGraphics": {"origin": "sourceSnapshot"}},
    "children": {
      "0": "/cad/d1/e102/graphics/p0",
      "1": "/cad/d1/e102/graphics/p1"
    }
  },
  {
    "path": "/cad/d1/e102/graphics/p0",
    "attributes": {
      "ifccad::dimensionGraphicsPart": {},
      "ifccad::geom::lineSegment": {"start": [0, 10, 0], "end": [100, 10, 0]}
    }
  },
  {
    "path": "/cad/d1/e102/graphics/p1",
    "attributes": {
      "ifccad::dimensionGraphicsPart": {
        "purpose": "referencePoint",
        "layer": "/cad/d1/layer/0"
      },
      "ifccad::geom::point": {},
      "ifccad::geom::placement": {
        "origin": [0, 0, 0], "xAxis": [1, 0, 0], "yAxis": [0, 1, 0]
      }
    }
  }
]
```

## Composition and source retention

Existing WIRE-002 composition runs first. `children.graphics` and each numbered
child key merge by key; repeated attribute values replace whole values. The new
roles are interpreted on the composed graph, under LaterWins or RejectConflicts.
Deleting the third key from a later fragment is not a deletion of the earlier
third child. With unchanged links it still participates in the active list.
Moving a key without updating its canonical target is invalid, not a reorder.

No graphics child means no representation, provided no orphan graphics role is
left in the composed input. A linked group with `children: {}` means explicitly
empty. Null is neither absence nor a tombstone. Read/encode never reconcile the
definition with the graphics. Original IFCX downloads retain original bytes;
fresh encoding does not preserve fragment history or foreign context.

The source-aware converter must recognize these role/geometry/reference paths as
profile content. Canonical physical omission and part-index paths are not foreign
losses. Actual foreign content and lossy projection of original numeric tokens
remain audited, including numeric fields on graphics nodes. No f64 conversion of
IDs or fragment bypass is introduced by adding graphics.

## CBOR records and codes

Add optional field **7 = graphics** to `IfccadDimension`, after its current
fields 0..6. Absence is None; present null is invalid. Existing codes keep their
meaning. The new records are local code spaces:

| Record | Code | Field and mapping |
| --- | ---: | --- |
| IfccadDimensionGraphics | 0 | origin, required integer enum: SourceSnapshot=0, Generated=1 |
| IfccadDimensionGraphics | 1 | parts, required ordered array of part records; empty array stays present |
| IfccadDimensionGraphicsPart | 0 | layer_id, optional uint64; absent inherits Dimension layer |
| IfccadDimensionGraphicsPart | 1 | kind, required unsigned variant code from the table below |
| IfccadDimensionGraphicsPart | 2 | geometry, required payload body selected by field 1 |
| IfccadDimensionGraphicsPart | 3 | appearance, existing IfccadEntityAppearance; default all ByLayer |
| IfccadDimensionGraphicsPart | 4 | visible, Boolean, default true |
| IfccadDimensionGraphicsPart | 5 | line_pattern_scale, binary64, default exact positive 1.0 |
| IfccadDimensionGraphicsPart | 6 | purpose, integer enum Display=0, ReferencePoint=1; default Display |

`geometry` is the selected body, not another `{0: kind, 1: body}` envelope. This
is an explicit contextual rule like the existing native entity kind/geometry
pair. Standalone encoding of the new graphics-kind enum uses the ordinary mixed
enum envelope. This rule is registered, not inferred from Rust declaration order.

The graphics kind codes are explicitly chosen to match the existing and prerequisite
primitive bodies; they are a separate closed enum, not a cast of EntityKind:

| Code | Kind | Body fields, with local codes |
| ---: | --- | --- |
| 0 | LineSegment | 0=start, 1=end, existing LinePoint mapping |
| 1 | Point | 0=placement |
| 2 | Circle | 0=placement, 1=radius |
| 3 | Arc | 0=placement, 1=radius, 2=start_parameter, 3=sweep_parameter |
| 4 | Ellipse | 0=placement, 1=semi_major_radius, 2=semi_minor_radius |
| 5 | EllipseArc | 0=placement, 1=semi_major_radius, 2=semi_minor_radius, 3=start_parameter, 4=sweep_parameter |
| 6 | PlanarPolyline | 0=placement, 1=vertices, 2=bulges, 3=closed, 4=line_pattern_generation |
| 7 | SpatialPolyline | 0=vertices, 1=closed, 2=line_pattern_generation |
| 8 | BlockInstance | 0=definition_id, 1=transform, existing IfccadBlockTransform mapping |
| 10 | Text | 0=value_0, existing IfccadText record |
| 11 | MText | 0=value_0, existing IfccadMText record; Box is transparent |
| 12 | Hatch | 0=value_0, existing IfccadHatch record, with graphics restrictions; Box is transparent |
| 14 | Solid | 0=value_0, IfccadFilledQuadrilateral: 0=placement, 1=corners, 2=thickness |
| 15 | Trace | 0=value_0, the same IfccadFilledQuadrilateral record |

Code 9 is unassigned in this enum and invalid; 13 and all other unlisted codes
are invalid as well. Existing ordinary Viewport=9 and Dimension=13 do not become
valid parts. Point/curve/text bodies retain their registered required fields and
defaults. The existing Hatch record codes are reused without a new Solid-only
binary dialect, then shared validation enforces the restricted value.

Codes 14/15 deliberately match the new ordinary kinds introduced by the native
SOLID/TRACE precursor. Their shared value retains four XY corner slots in storage
order, signed thickness and the existing placement. Geometry restrictions/defaults
and codes 0/1/2 are defined once in that slice; graphics does not narrow them to
zero thickness or add a source-kind flag. ReferencePoint purpose remains Point-only.

All floating fields use binary64 and preserve bits, including negative zero.
Optional fields are omitted for None, not null. Defaults are omitted only by the
existing exact rule; all-ByLayer appearance may be omitted as a whole or expanded
from its registered leaf defaults. Explicit layer ID zero remains present.
The origin and parts fields are required, including for an explicitly empty group.
Unknown/repeated keys, unknown origin/purpose codes and incompatible bodies fail.

For example, an explicitly empty generated representation is the CBOR data model
`{0: 1, 1: []}` in Dimension field 7. A default Display line part has fields
`{1: 0, 2: {0: [x1,y1,z1], 1: [x2,y2,z2]}}`; each coordinate slot follows the
existing LinePoint/binary64 rules. These notations explain maps, not JSON wire
syntax or permission to encode coordinate integers as CBOR integer values.

Revision remains the single mutable unreleased version 1. Implementation updates
the active code inventory, record registration and conformance/next in place.
No historical decoder or version bump is added. Optional Zstd transport revision
1 wraps the same bytes; it introduces no graphics codes or feature negotiation.

## Stored state, bounds and rendering

Ordinary load, validation and encoding retain the current definition and the
current graphics independently. They never compare a snapshot with its style,
measurement or associations, generate replacements, or remove a representation
because it might be stale. A placement change applies to the whole local
representation. An editor or future update engine decides when to replace/remove
graphics after other edits; no invalidation API or hidden regeneration is added.

Absent graphics keeps the current unavailable Dimension display-bounds result.
Present graphics derives its enclosure from stored Display parts, transformed
through the Dimension frame and any marker occurrences. Definition anchors and
ReferencePoint helpers do not enlarge that enclosure. Empty or helper-only
graphics is a known empty display, not missing graphics. Text/MText retains the
existing estimated/partial extent quality, with font contours unassessed.
Primitive enclosure does not become proof of filled area or identical pixels.

As with ordinary scope bounds, the assessment conservatively includes stored
Display geometry even when visibility is off; rendering independently applies
the stored visibility and layer context. ReferencePoint is the explicit exception.
Validation and numerical conversion still examine every part, including helpers
and hidden parts. Missing optional surface evidence never drops a SOLID/TRACE.

## Transient conversion identities and reports

Extend the existing generic occurrence proof graph with tagged internal keys for
ordinary entities, ordinary definitions, Dimension graphics groups, their local
parts and their placed occurrences. These are conversion-only keys, not native
entity/block IDs. A graphics group contributes its owning Dimension placement
once; a nested marker contributes its own transform once. Keep definition-anchor
proofs alongside graphics proofs instead of overwriting them under the same key.

Each graphics diagnostic/evidence source identifies owner, the owning Dimension
and part index, plus the actual CAD part handle when available. Use the real CAD
Dimension handle during import and the native Dimension ID during export; do
not pretend a source handle is an allocated native ID. Extend the existing
`IfccadGeometryEntitySource` with a graphics-part wrapper around its existing
CAD/native source. Do not synthesize ordinary handles/IDs to reuse scope conversion.
Full-width IDs remain integers throughout IFCX/CBOR and browser inspection.

Dimension transfer reports distinguish Absent, ExplicitEmpty, Transferred (with
part count) and NotTransferred graphics. This is derived operation evidence,
not a stored validity state. Display coverage remains Unassessed in all cases.
Re-use the existing primitive/curve/text/Hatch/filled-surface evidence in each
actual owner/occurrence; Paper physical tolerance still requires its qualified
plot mapping. Definition-only success cannot certify picture coordinates.

## CAD parts and exact substitutions

Capture uses the actual DIMENSION pictureblock relationship and source entity
order. It first prepares the complete candidate and resource closure, then binds
the value. Calling an ordinary importer that skips an unsupported sibling and
attaching its remaining output is forbidden. Non-displaying helpers are still
validated and explicitly classified. Every new part has a diagnostic source path
and transient source-handle mapping; it is not inserted in the ordinary native
entity-ID map as a synthetic scope entity.

| CAD source | Native graphics | Bounds of first profile / CAD output |
| --- | --- | --- |
| LINE | LineSegment | Existing zero-thickness geometry and appearance profile; output LINE |
| POINT | Point, Display or ReferencePoint | Existing finite position/normal/orientation and zero-thickness profile; output POINT under the purpose rules below |
| CIRCLE / ARC | Circle / Arc | Existing radius/sweep qualification; output corresponding primitive |
| ELLIPSE | Ellipse or EllipseArc | Existing full/partial parameter qualification; output ELLIPSE |
| LWPOLYLINE / supported classic 2D polyline | PlanarPolyline | Straight/bulged, non-fitted; native Constant/PerVertex/Defaulted widths via the separate width contract, zero extrusion thickness; wide stored pictures qualified at AC1032 |
| Supported 3D polyline | SpatialPolyline | Straight segments, no fitted/mesh state; ordinary nested geometry rules |
| TEXT / MTEXT | Text / MText | Existing typed content, stack, placement and styles; output corresponding text type, including wrap width zero |
| INSERT | BlockInstance | Local non-dynamic block, existing qualified base/transform/unit behavior and complete eligible closure; output INSERT |
| Solid HATCH | restricted Hatch | Existing boundaries/area semantics and appearance; output HATCH; association/source removal, if required, remains diagnosed loss |
| SOLID | Solid | Reuse all finite local corner combinations and signed thickness from the precursor, including crossing/collapse; output SOLID; coordinate proof and surface/display coverage remain separate |
| TRACE | Trace | Reuse the same geometry profile with a distinct kind; output TRACE |
| Other visible source, including width/thickness unsupported for its own primitive kind | No partial graphics | Located whole-representation loss under Allow, refused under Reject |

The [native SOLID/TRACE slice](../geometry/solid-trace-design.md) owns geometry,
validation, bounds, kind retention and ordinary conversion. Graphics calls that
same preparation after rebinding the qualified picture context. It does not
convert either type to Hatch or wide polylines, and does not add a parallel
special converter limited to arrow blocks.

Four corner slots remain stored, even for triangles. Quad boundary order is
1,2,4,3; adjacent duplicate-corner analysis is exact, never the codec's epsilon
shortcut, and never changes stored slots. A missing simple/common surface cover
does not invalidate, omit or make graphics partial: the entire descriptor is
represented while its surface/display guarantee remains separate. Signed
nonzero thickness is a native value and must be included in base/top and nested
occurrence proofs. The graphics CAD cache-frame profile can impose an explicit
target limitation, particularly for a nonplanar display sweep in a tilted planar
cache; it must not silently erase thickness to make an arrow fit.

Ordinary marker definitions use the same native Solid/Trace entities with their
normal IDs. Graphics parts use the shared payload without an ordinary entity ID.
The source codec's is_trace flag is mapped to/from the closed native kind at the
adapter boundary, not persisted as an extra provenance bit. Supported type retention
does not by itself cause a Modified diagnostic or Reject refusal; other source
metadata, geometric precision and unsupported profiles keep their existing rules.

## Helper points and source visibility

Direct POINT children of a linked dimension pictureblock on the drawing-local
CAD `Defpoints` layer are captured as ReferencePoint. They keep their real layer,
placement, visible flag and appearance. The qualification is specific to POINT
in that picture context; the adapter does not hide arbitrary geometry or whole
layers merely because their names resemble Defpoints. Other POINTs remain Display.
Native rendering uses the explicit purpose and does not infer it from layer names.
The source signature and non-displaying interpretation require literal fixture
and unchanged-host checks; unidentified conventions are not guessed into this role.

Export of ReferencePoint requires an explicit local Defpoints layer, as provided
by this capture profile. Otherwise it reports an unsupported target mapping for
the complete graphics value rather than inventing a layer or changing ordinary
native layer meaning. No point is reconstructed from the current definition;
stored helper positions can differ from current reference anchors. Removing or
altering helpers is never a substitute for fixing an unsupported visible part.

For Display parts, explicit visibility, effective layer visibility/frozen/plot
state and inherited Dimension visibility remain independent. Invisible source
objects are not a blanket permission to discard unknown source semantics. Their
content is either captured within the closed family or separately diagnosed.
ReferencePoints remain inspectable but contribute no graphical enclosure.

## Appearance context and marker blocks

The root graphics context is the owning Dimension's current layer, appearance,
visibility and line-pattern scale in its actual Model/Paper/block occurrence.
Parts on CAD layer 0 capture as inherited-layer choice; other source layers bind
explicitly. A native explicit layer whose CAD name is 0 cannot be exported as
that explicit choice when CAD would instead inherit a different owner layer:
this is an unsupported target context, not an excuse to freeze resolved colors.

ByBlock leaves resolve against the immediate graphics/block parent; ByLayer
uses the effective part layer. An INSERT establishes the next parent context.
Inside the ordinary marker block, CAD layer-0 inheritance is evaluated in that
graphics occurrence, while nonzero named layers remain explicit. The contextual
rule is part of this graphics contract; it does not globally reinterpret Model
entities as inherited-layer records or introduce a general appearance resolver.
If the native resource/target cannot represent that scoped context, the graphics
is not qualified. Pure geometric copying without its layer/appearance context
does not count as a successful capture.

The ordinary global/entity/block line-pattern scale rules continue, with the
Dimension and part entity scales each applied once. Dimension component style
settings are not applied again to already authored graphics parts. No per-part
semantic guesses such as 'first arrow' or 'extension line' are required. Native
Foreground, concrete color identity, ByLayer and ByBlock remain distinct.

## CAD picture frame and placement

A dimension pictureblock is not assumed to be an ordinary INSERT placed at the
Dimension origin. For example, the sample has absolute-looking XY coordinates
near 400 in a block whose base point is zero; adding the definition point again
would translate the picture twice. Tilted dimension caches can instead encode
LINE, MTEXT and POINT data in the Dimension OCS. This needs a dedicated adapter
before reusing ordinary primitive preparation.

The first qualified cache profile keeps the existing definition restrictions:
cloned insertion point is zero, insertion scale is [1,1,1], insertion rotation is
zero and root pictureblock base point is zero. Unsupported clone/base behavior
is reported; it is not inferred from a generic INSERT matrix. Ordinary nested
marker block base points and their already qualified transforms remain supported.

Define P as the stored native Dimension frame and C as the qualified source
picture-to-owner frame. Capture evaluates each effective source primitive q as
`P^-1(C(q))`; export uses the target inverse `C_target^-1(P(q_local))`.
P includes the exact cross-product third axis and is solved as a basis, not
projected by an assumed orthonormal dot product. Nested marker transformations
apply at their own level exactly once.

The source/target picture conventions are explicit:

- For the default +Z Dimension normal, use ordinary entity coordinates in the
  owner's frame after each entity's own OCS interpretation. Do not add the
  Dimension definition point as a block insertion translation.
- For a tilted or -Z Dimension, the initially qualified planar cache uses the
  Dimension's arbitrary-axis OCS, with elevation from its OCS text-midpoint Z.
  Direct picture LINE/POINT/MTEXT coordinates are rebound to that frame; placed
  curves, text and INSERTs use the Dimension normal/elevation before ordinary
  preparation. The profile requires canonical planar zero stored Z and compatible
  default child normals/directions. It never overwrites conflicting authored Z
  or an independent child normal to force the convention to fit.
- Rebinding applies to direct picture children, not recursively to the already
  local contents of ordinary marker definitions. Existing INSERT transforms
  carry those definitions into the picture frame.
- Export of a non-+Z cache requires the display geometry to fit that planar
  convention with existing tolerances. Native out-of-plane graphics stays valid
  in IFCX/CBOR but has a diagnosed CAD target restriction. Reference anchors with
  independent XYZ heights are not flattened and are proved separately.

For style-driven target text, the exporter may populate a CAD display-cache anchor
without setting the user-positioned flag or changing native text placement. It
uses the first Display Text/MText anchor in stored order, or the dimension-line
point when there is none, to supply the cache anchor and required elevation.
Explicit native text retains its authored anchor. A conflicting cache-plane
elevation is a target restriction; the exporter does not move stored graphics.
No text anchor is inferred by measuring glyphs or generating a layout.

These are qualification rules, not permission to trust an in-memory transform.
Literal zero/nonzero-elevation, tilted and -Z caches must pass independent
primitive-location proofs and real DXF/DWG readback. A codec or host which uses a
different cache convention receives a located restriction, not silently moved
geometry. The implementation must not certify tilted picture support by testing
definition anchors alone or by using matching reader/writer assumptions.

## Block accounting and failure policy

Inspect picture links and marker dependencies before ordinary block traversal.
A private pictureblock may be consumed into Dimension graphics only when every
source use is accounted for and its remaining record state is known bookkeeping.
Names beginning with `*D` are not evidence of private ownership. Shared pictures
give each Dimension an independent part value; ordinary reusable marker resources
may remain shared. If the picture definition also has ordinary INSERT, style,
foreign or other uses, keep/audit its ordinary definition separately. Native scope
entities are never also inserted merely to display the copied graphics.

A transient map `(source Dimension handle, part index) -> source part handle`
locates preparation, normalization and numerical evidence. It is separate from
ordinary entity allocation/association maps. Source records represented through
graphics must be accounted for by the source audit; their metadata must not
vanish because a block was marked consumed. A partial marker block or skipped
required resource makes the entire dependent representation unavailable.

Unsupported content or target expressiveness gives a located representation
loss under Allow and is refused under Reject. Invalid geometry, hard references
and numerical excess remain errors under Abort, including helper records.
Explicit SkipInvalidEntities may omit an affected Dimension as a whole only
within the existing recoverable entity-error boundary, retaining the original
cause and dependency locations. It never attaches a truncated graphics value.
Shared structural/resource errors retain their existing fatal classification.
The source is not mutated, and typed output must pass strict validation.

## Diagnostics and implementation acceptance

The design reserves these shared native rule families for both logical validation
and encoding-specific paths; implementation keeps existing typed report/error APIs:

| Rule | Failure family |
| --- | --- |
| IFCCAD-DIMENSION-GRAPHICS-001 | Invalid origin, purpose, geometry subset or incompatible combination |
| IFCCAD-DIMENSION-GRAPHICS-002 | Missing/wrong/cross-drawing resource reference or graphics render cycle |
| IFCCAD-DIMENSION-GRAPHICS-003 | IFCX role, canonical path, ownership, numbering, duplicate or forbidden placement |

CBOR physical unknown/missing/repeated keys and resource-budget failures retain
the existing CBOR error categories. CAD failures keep the relevant geometry,
text, Hatch or Dimension preparation cause; adding a graphics location must not
turn precision failures into unsupported metadata. Conversion loss codes are
`dimension-graphics-unsupported`, `dimension-graphics-target` and, for loss of source-only helper/editing
information, `dimension-graphics-helper-state`. Numerical rounding retains the
existing certified-rounding classification and hard owner/occurrence tolerance.

Required regressions include absent/empty/nonempty values in both encodings,
all listed part kinds, invalid code 9/13, no synthetic entity IDs, full-width
references, unknown/null rejection, exact float/default rules, orphan and
cross-Dimension nodes, reordered/wrong pI targets, 3-to-2 key composition,
ReferencePoint bounds/render suppression and source-aware precision/foreign audits.
Hatch restrictions and resource cycles apply to unused eligible marker definitions.

CAD regressions cover the entire candidate or its explicit rejection: separate
SOLID and TRACE kind retention for every finite local corner combination,
including crossing/retrace/line/point states; signed thickness and mandatory
base/top occurrence proofs with optional surface coverage; separate unequal-OCS-Z
refusal, with no graphics fallback merely because fill/display is unassessed;
marker blocks containing both kinds; Defpoints helpers versus ordinary points;
layer 0/ByBlock context; cloned/root-base restrictions; tilted/-Z/nonzero-elevation
cache coordinates; Text/MText with wrap zero, literal spaces and stacked fractions;
and the independent Allow/Reject and Abort/SkipInvalidEntities policy matrix.
Use real ASCII DXF, binary DXF and AC1032 DWG, plus unchanged-host visual checks
on the actual preview/download bytes. No OpenCADStudio patch is part of this slice.

The retained sample control DXF has *D8 contents LINE x3, INSERT x2, MTEXT x1,
POINT x3; its `_ArchTick` marker contains one LWPOLYLINE. Its *D3 also exercises
two SOLID arrows. The control intentionally removes the red Dimension's link,
so it is evidence for block contents, not a replacement positive link fixture.
Control SHA-256: `10fa89d584ab7e63d2fbf5242709ef71f529b3a875bd18670126febdb2a59b7b`.
The original source/link provenance remains in the earlier retained repair archive.
These read-only inventories are design evidence, not passing implementation tests.

The original `_ArchTick` entity `4A6` has LWPOLYLINE constant width 0.15.
The initial bounded graphics slice retained this as a documented zero-width
boundary. The separate [native width slice](../geometry/polyline-widths.md) now
qualifies the actual linked `527 -> *D8` source from both original DWG and ASCII
DXF, all nine picture parts and both marker INSERTs. Strict IFCX/CBOR/Zstd and
AC1032 ASCII/binary DXF/DWG readback preserve the marker and text semantics;
bounded unchanged-host observations match both red ticks and the stacked label.
No marker is zeroed, narrowed, substituted or replaced by a partial picture.
The earlier detached-link control remains historical inventory evidence only.

The independent zero-width/Solid/Trace/Text fixtures still exercise the original
bounded graphics profile. Unsupported nonzero extrusion or conflicting width
controls retain whole-picture loss/rejection and transactional rollback. Width
support does not broaden arbitrary cache transforms, regeneration or glyph claims.

At implementation time, update active IFCX/supplemental schemas, profile rules,
the single CBOR register and conformance/next, strict reader/writer readback,
source-aware audit, bounds/inspection and both directional converter coverage.
Qualify optional Zstd and the existing frontend/WASM paths as applicable, with
the final workspace gate and recorded input/dependency provenance. No measurement,
legacy work, commit, integration or publication follows from this design.

## CAD references and evidence limits

[Autodesk common Dimension codes](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-EDD54EAC-A339-4EBA-AEA6-EC8066505E2B.htm)
defines the pictureblock link and distinguishes definition WCS from text OCS.
[Autodesk SOLID](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
documents repeated third/fourth corners for triangles and thickness/normal fields.
[Autodesk definition points](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-LT/files/GUID-BDD8C619-AB31-49D8-AE55-465EE6FAB61D.htm)
describes their special Defpoints layer and non-plotting role.

The selected codec's four raw Solid corner fields and is_trace flag are handled
by the preparatory native slice. Its `Solid::boundary_corners` corroborates quad
order, but its epsilon triangle predicate is deliberately not the capture predicate. Its current
Dimension preparation rejects clone transforms. The primary
[ezdxf Dimension implementation](https://github.com/mozman/ezdxf/blob/master/src/ezdxf/entities/dimension.py)
provides independent evidence of the special OCS treatment of tilted picture
primitives. That observed convention motivates the bounded cache-frame profile;
it does not certify every producer, codec or host and does not select a new
runtime dependency. The actual fixtures/readback remain the implementation gate.

The SOLID/TRACE precursor is integrated on remote main at the baseline recorded
above. Graphics capture/reuse remains unimplemented. Open issues
[#2](https://github.com/OpenAEC-Foundation/ifccad/issues/2) and
[#13](https://github.com/OpenAEC-Foundation/ifccad/issues/13) were reviewed for this
plan: typed bounded entities fit #2; stable regeneration identity and dependency
updates remain #13's separate workflow. Neither selects a picture generator,
persisted validity test or generic preservation framework for this slice.
