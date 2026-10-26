# Experimental IFCCAD CBOR mapping, revision 1

This is a standalone encoding of `IfccadDocument`, not a general IFCX binary
serialization. It retains the complete supported CAD projection, including
opaque CAD preservation, but no original IFCX fragments, imports or foreign
nodes. JSON/IFCX remains a separate mapping through the same typed document.

The normative field/type inventory is [cbor-mapping-1.json](cbor-mapping-1.json).
It enumerates concrete records and generic instantiations reachable from the
document. Names in that registry are explanatory, not transmitted type names.
Record field codes are local to their record type. Codes and enum variants are
explicit, independent of Rust declaration order. Object IDs are separate uint64
values and never inferred from field codes, vector indices or content.

## Envelope and revision

Exactly one CBOR map contains `{0: "IFCCAD-CBOR", 1: 1, 2: Document}`.
All three fields are required. Other revisions are rejected; no migration,
forward-compatible skipping, feature negotiation or historical decoder is
provided. Version 1 is a mutable, unreleased experimental draft, like the active IFCCAD
profile. Update this single contract and its fixtures in place, including changes
to language, semantics, codes, defaults and normalization. No compatibility with
earlier experimental files is promised. Do not add historical mappings or bump
the version before the first publication. The [release policy](../../docs/releases.md)
defines freezing that published snapshot and incrementing subsequent contract
revisions without requiring historical readers. File names conventionally end in
`.ifccad.cbor`; contents determine validity. This raw mapping uses no compression
or extra binary prefix. The optional experimental
[Zstd transport](cbor-zstd-transport-1.md) wraps these unchanged payload bytes
in a separate standard frame.

## Physical values

- Definite-length maps/arrays/byte strings/text strings only; no tags, null,
  undefined, non-finite floats, trailing items or invalid UTF-8.
- Fixed record fields use unsigned integer keys. Unknown or repeated keys are
  errors, regardless of their values. Map order is incidental on reading.
- All logical f64 values use binary64 (initial byte `fb`), preserving every
  bit including negative zero and subnormals. Integers and shorter floats are
  not accepted in these slots. Other integers retain their logical ranges;
  IDs and watermarks cover uint64 without any float or signed intermediate.
- `char` is a text string of exactly one Unicode scalar. Text is not normalized.
- Fixed arrays and mathematical tuples are CBOR arrays. No packed numeric
  buffers, columns, shared pools, quantization or coordinate deltas.
- A transparent ID newtype is its scalar. Box is transparent. Lists preserve
  order; semantic maps keep their registered key type. Option fields are
  represented by field absence, never null. Some(empty/false/zero) remains
  present except an explicitly registered default of that exact field type.
- Enums with only unit variants use their integer variant code. All variants
  of a mixed enum use `{0: variantCode}` for unit variants and
  `{0: variantCode, 1: payloadRecord}` for payload variants. Named payload
  fields have their local codes; tuple payload components use 0, 1, etc.

## Native entities

Entity is `{0: variant, 1: body}`: Native=0 and Opaque=1.
Native body uses `0=id, 1=layerId, 2=kind, 3=geometry, 4=appearance,
5=visible, 6=linePatternScale`. Here `geometry` is the payload body selected by
`kind`, not a second tagged enum envelope. The registry describes that selection
through the IfccadEntityKind variants. This contextual rule overrides standalone
enum representation only for this native-body field pair.

If an opaque appearance is present, its `appearance` record uses the separate
`OpaqueAppearanceModes` mapping: all four mode fields are required. Native
ByLayer omission does not apply to it. An absent opaque appearance remains None,
not an invented inherited appearance.

Kinds 0..13 are LineSegment, Point, Circle, Arc, Ellipse, EllipseArc,
PlanarPolyline, SpatialPolyline, BlockInstance, Viewport, Text, MText, Hatch,
Dimension. Codes 14 and 15 are Solid and Trace. Their body is `{0: filledValue}`;
filledValue is `{0: placement, 1: corners, 2?: thickness}`. Corners is exactly
four XY binary64 pairs; thickness defaults to exact +0, so -0 stays present.
All finite corner topologies are allowed under the shared frame/enclosure rules.
No derived surface or display evidence is stored.
IDs, ownership vectors and draw order retain their original meaning.

The single Dimension kind has its own definition enum: Linear=0, Aligned=1.
Other DimensionKind values 2..8 are legal style override keys but do not make
their corresponding unimplemented entity definitions legal.

## Defaults, presence and output

The registry marks each field as required, optional or defaulted. Missing
required fields fail. Missing optional fields become None. A default is
expanded before shared semantic validation; no general Default fallback applies.
The register includes the [drawing defaults](../../docs/drawing-defaults.md)
and current main's explicit fraction/Hatch defaults. Fraction layout defaults to
Inline (code 0) and scale to binary64 1.0 in primary/alternate settings. Optional
overrides still distinguish absence from authored Inline or 1.0. Hatch association
and text-island flags default false; ordered source lists default empty. Present
provenance defaults each flag false, but an all-false provenance record fails
shared logical validation. No new fallback is inferred from Rust Default.
Numeric comparisons use exact f64 bits: +0 may be omitted, -0 may not.
Line endpoints alone accept XY as XYZ with +0 Z; other XYZ points and placements
stay complete. Opaque absence is never interpreted as native ByLayer appearance.

Only collections explicitly registered as empty-default lists may be omitted as
empty, including Document collections and Hatch source lists. Existing definitions, IDs, watermarks, supplied bounds, rich
text resets, font options, dimension overrides and inactive gradient state are
not synthesized, repaired, sorted or discarded. Preservation payload/baseline
bytes use byte strings without Base64 or provider interpretation.

Writers use ascending numeric map keys, UTF-8 byte order for authored string
keys, shortest integer/length headers, and explicit binary64. Every vector
retains its supplied order, including otherwise unordered definition collections.
Repeated encoding of the same document under this mapping is byte-deterministic;
this is neither an across-format semantic fingerprint nor a claim of RFC 8949
core deterministic encoding with shortest floats.

## Validation and resource limits

Readers validate physical structure and registered types, then call the shared
`validate_ifccad_document`. Writers validate, encode, load through the production
CBOR reader, and compare the complete document including numeric bits before
returning encoded bytes. There is no JSON bridge, original graph snapshot or
source-graph audit in the binary route.

Configurable default read limits: 512 MiB input, nesting 128, 16,000,000 entries
per container and 32,000,000 items total. Root depth is one; map keys/values
count separately; string bytes are not separate items. Length claims are checked
against remaining bytes, budgets and usize before allocation. File input is
bounded during reading. These are reader resource limits, not drawing semantics.
Internal writer readback uses finite limits established from its actual output.

Malformed mapping, unsupported revision, resource limits and logical validation
remain distinct failures. This provisional contract does not publish a supported
version. Optional whole-file compression is defined separately by the Zstd
transport contract; columns and chunks remain future work. Measurements are
documented separately from this mapping contract.

Reference: [CBOR, RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html).

## Current logical model

The registry includes Dimension fraction layout/scale and explicit resets,
Hatch ordered source groups, association state, provenance flags and text islands.
HatchLoop code 1 is unassigned; ordered source_entity_ids use 2, provenance uses 3
and text_island uses 4. There is no associative geometry update engine.

The JSON reader's numeric preservation-record ordering is graph normalization.
CBOR preserves supplied record-vector order, along with payload bytes, IDs,
watermarks and opaque entity draw order. JSON export may normalize record order
when the document is explicitly passed through that separate mapping.

## Spatial Dimension reference anchors

`DimensionDefinition::Linear` and `Aligned` store first_point/second_point
as required fixed XYZ arrays of three binary64 values. A +0 Z is not omitted.
The dimension-line point remains fixed XY. Measurement uses the XY reference
projections; Z is retained native reference geometry along the exact cross
product of the stored placement X/Y axes. Explicit text remains XY in that plane.
XY reference tuples are rejected, never completed with an inferred Z.

## Dimension graphics

Dimension record field 7 optionally stores IfccadDimensionGraphics. Its required fields are 0 origin (SourceSnapshot=0, Generated=1), 1 ordered parts (including explicit empty). Each part has 0 optional layer_id, 1 required kind code, 2 required contextual geometry body, 3 appearance (all ByLayer default), 4 visible (true), 5 binary64 line_pattern_scale (1.0), 6 purpose (Display=0 default, ReferencePoint=1). Layer zero is present, not inherited; negative zero is preserved. The body is the existing primitive body, without a second enum envelope. Standalone graphics-kind values retain the ordinary tagged-map envelope. Allowed codes are 0..8, 10..12, 14/15; codes 9/13 are invalid. Origin/parts have no omission defaults. These records update the one experimental revision 1 in place. Shared logical validation applies the closed Hatch and marker/helper restrictions.

## Hatch editing and construction

IfccadHatch optional fields 7 classic_settings and 8 boundary_construction are
omitted for None. Classic record fields: 0 optional preferred_fill (Solid=0,
LinePattern=1), 1 optional creation_type (UserDefined=0, Predefined=1, Custom=2),
2 optional name, 3 optional description, 4 optional XY origin, 5 rotation (0),
6 scale (1), 7 double (false), 9 optional ordered definition_lines. Classic code
8 is unknown and rejected; it is not a historical mapping. Definition-line record
fields are 0 required angle, 1 required XY base_point, 2 required XY offset_vector,
3 signed dash_lengths (default []). These are direct effective values, independent
of the active normalized HatchLineFamily mapping. Construction has
0 ordered XY seed_points ([]), 1 pixel_size (0). Numeric slots use binary64;
exact defaults can be omitted, negative zero is retained. Option empty/zero/code0
remains present, null/unknown/duplicate keys fail. All-default present records
fail shared logical validation. The single unreleased revision 1 is updated in
place, with no historical mapping, revision bump or changed Zstd envelope.

`LinePatternRotation` uses variant codes 0 Relative, 1 Absolute and 2 Upright.
Each has the same required body field 0 containing a binary64 radian angle.
Upright zero is explicit and distinct from the default relative positive zero.
This updates the single mutable mapping revision 1 in place.
