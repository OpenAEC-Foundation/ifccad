# Experimental IFCCAD CBOR mapping, revision 7 (development)

This is a standalone encoding of `IfccadDocument`, not a general IFCX binary
serialization. It retains the complete supported CAD projection, including
opaque CAD preservation, but no original IFCX fragments, imports or foreign
nodes. JSON/IFCX remains a separate mapping through the same typed document.

The normative field/type inventory is [cbor-mapping-7.json](cbor-mapping-7.json).
It enumerates concrete records and generic instantiations reachable from the
document. Names in that registry are explanatory, not transmitted type names.
Record field codes are local to their record type. Codes and enum variants are
explicit, independent of Rust declaration order. Object IDs are separate uint64
values and never inferred from field codes, vector indices or content.

## Envelope and revision

Exactly one CBOR map contains `{0: "IFCCAD-CBOR", 1: 6, 2: Document}`.
All three fields are required. Other revisions are rejected; no migration,
forward-compatible skipping, feature negotiation or historical decoder is
provided. Revision 6 and profile 0.6.0 are the unpublished block-attribute development candidate.
Published revision 5 / profile 0.5.0 remains fixed in tag 2026.45.1.
Revision 3 and profile 0.3.0 were published in native source alpha 2026.45.0;
that tagged contract is fixed.
Revision 1 and profile 0.1.0 were published in native source alpha 2026.44.0;
that tagged contract is fixed. Subsequent normative
changes to language, semantics, codes, defaults or normalization advance the
appropriate identifiers under the [release policy](../../docs/releases.md).
Keep one active mapping; historical readers are not required. No compatibility
with earlier experimental files is promised. File names conventionally end in
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
fail shared logical validation. These fields were added to draft revision 1
before its first publication; no historical draft mapping is retained.

`LinePatternRotation` uses variant codes 0 Relative, 1 Absolute and 2 Upright.
Each has the same required body field 0 containing a binary64 radian angle.
Upright zero is explicit and distinct from the default relative positive zero.
This variant was added to draft revision 1 before its first publication.

## Primitive thickness fields (mapping 3)

Ordinary and graphics kind codes stay unchanged. Their contextual body additions
are LineSegment 2=thickness, 3=extrusion_direction; Point 1=thickness; Circle
2=thickness; Arc 4=thickness; PlanarPolyline 6=thickness (5 remains widths).
Thickness scalars and the direction triple use binary64, with exact +0 and
[+0,+0,1] defaults respectively; signed zero is not discarded. Profile 0.6.0 is
required. Release 2026.44.1's profile 0.2.0/mapping 2 remain fixed in its tag.

## Annotation storage in the development candidate

Document field 20 is the annotation-scale registry (omitted empty); allocation
counter field 11 is required next_annotation_scale_id. Scale record fields are
0 id, 1 name, 2 paper_units, 3 drawing_units, 4 unit_scale (default false).
Text field 9 and MText field 14 optionally contain a saved annotation descriptor.
Descriptor fields are 0 mode (0 Disabled, 1 Enabled, 2 ByDefinition), 1 optional
reference key, 2 required context list. Each context has 0 key and 1 optional
presentation; each key has 0 scale ID. Optional values omit their field; a
present null is malformed. Family records and exact enum codes are declared in
[cbor-mapping-7.json](cbor-mapping-7.json). Shared profile rules govern semantic
validity in both encodings. Floating-point payloads retain their exact bits;
no record field calculates a scale quotient or normalizes a direction.

Dimension sizing replaces the previous positive scalar with a tagged record:
kind 0 Fixed with body field 0 factor, kind 1 ByViewport without a body, or kind 2
ByAnnotationScale with a required body containing optional field 0 dormant factor.
Both factor alternatives require positive finite values when present. Dimension
Style field 4 retains its independent annotative bool (default false).

TextStyle field 3 and BlockDefinition field 11 retain definition annotation
intent, omitted when disabled/fixed. Definition record fields 0 enabled (default
false) and 1 orientation (0 Fixed, default; 1 MatchLayout) preserve inactive
orientation independently of enabled. No entity membership is synthesized from a
TextStyle flag. See shared profile rules for block inheritance and bounds limits.

Descriptor field 3 optionally retains explicit object orientation (0 Fixed,
1 MatchLayout). Absence is unspecified and differs from a saved Fixed value.
This does not alter the separate definition policy defaults or apply a view
transform. Null is malformed.

Document field 21 stores the annotation-view registry, omitted empty. Counter
field 12 stores next_annotation_view_id, omitted only for exact default 1. The
stored-view record fields are 0 id, 1 name, 2 space (Model 0, Paper 1), 3 width,
4 ordinary viewport camera, 5 optional annotation_scale_id. Domain IDs, names and
watermark are validated independently of annotation scales.


BlockInstance body code8 adds optional field2 annotation, using the shared record
with IfccadBlockAnnotationContext presentation (field0 transform). Definition_id
and base transform keep fields0/1. Context transform encoding reuses the ordinary
IfccadBlockTransform record, including exact-default rules and independent values.


Dimension record adds optional field8 annotation. Its presentation fields are
0 text_location (XY), 1 user_text_location, 2 text_rotation, 3 layout, 4 geometry,
5 optional graphics (the ordinary owned picture record). Layout fields0..4 are
line_outside, suppress_arrows_outside, fit_switch, force_text_inside and
text_movement_switch; each StoredBoolOverride requires0 active and1 value,
including inactive values. Layout fields5..7 are second_arrow_present, flip_first
and flip_second. Context geometry codes0..8 match DimensionKind; each body uses
fields0.. in the order listed in the profile rules. All geometry points are XYZ.

Document field22 optionally stores annotation settings: field0 optional
insert_as_annotative bool; field1 optional model_linetype_scaling (0 Unscaled,
1 ByAnnotationScale). The record must not be empty. Model field6 and Paper field8
optionally store annotation state: field0 optional current_scale_id, field1
optional visibility (0 CurrentScaleOnly, 1 All). Viewport field10 and stored-view
field5 optionally store annotation_scale_id. ModelWindow adds no scale field.
All references resolve against this document's scale registry; omitted values
remain unspecified, independently of explicit false/zero enum values.

## SHX request extension in mapping 5

SHX variant code 0 retains body field 0 (`file_name`: required string). Optional
body field 1 (`cad_file_name`: string) defaults to None and is omitted only when
absent. Explicit null, duplicate fields and unknown fields are rejected. Present
strings, including a request equal to the basename, survive exact native IO.
Shared source validation applies the logical basename/request rule.

## Revision 6 attributes

The single active registry includes AttributeDefinition (entity-kind code 16),
BlockInstance attributes (local field 3), and drawing attribute_visibility (local
field 23, integer enum Normal=0, HideAll=1, ShowAll=2). All attribute records,
annotation contexts and field graph/cache/codepart variants have explicit local
codes in the registry. Signed field metadata uses i32; signed Integer caches use
i64. Object caches and Object codeparts carry global entity IDs, while Child
codeparts and field record IDs are local to the owning attribute graph. JSON
path spelling is not embedded in CBOR. Physical CAD limitations do not weaken
native exact-equality readback or graph validation.


Leader-family extension (unreleased profile0.7/CBOR7): kinds17/18/19 use the same closed wrapper0 as other boxed native values. Document field24 is the MultiLeaderStyle registry; counter13 allocates its IDs. DimensionStyle field5 is the independent leader arrow. The JSON register enumerates all nested command, branch, label, settings and presentation records. Ordered command streams retain state/transform order and bounded CadCompatibility51Zero; no second primitive list is stored. Sets of override properties use unique arrays in canonical encoded-key order. Source indices and positive part/label keys remain separate domains. ToleranceContent physical CAD payload qualification remains open; native storage does not claim that qualification.
