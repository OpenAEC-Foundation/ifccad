# Complex line patterns

OCDraw and the independent IFCCAD profile support ordered line pattern elements
with optional text or symbol markers. The core model stores requests and local
references; it does not load fonts, resolve filesystem paths or render SHX glyphs.
See the active schemas and conformance cases for the normative serialized contract.

## Elements and progression

A plain number keeps the existing meaning: positive is a stroke, negative is a
gap, and zero is a dot. A marked element has `length` and `marker`. Its signed
length retains stroke/gap progression, but zero places the marker without an
additional dot. An empty definition remains a separately named continuous line.
A nonempty pattern starts with a nonnegative length and has a finite positive
sum of absolute lengths. A simple definition needs at least two elements; a
complex definition may have one. Marker presence does not make a zero period valid.

For example, an OCDraw pattern can contain:

```json
[
  10,
  {"length":0,"marker":{"kind":"text","text":"GAS","textStyleId":1}},
  -4,
  {"length":0,"marker":{"kind":"symbol","symbolSourceId":2,"shapeNumber":42}},
  -4
]
```

Text is a literal string and references an ordinary drawing-local text style.
Symbols reference a separate drawing-local source and a nonnegative `u32` shape
selector. The two identity domains are independent. Unused styles, sources and
definitions remain meaningful records. Duplicate source filenames do not merge IDs.

Every marker has a positive finite `scale`, a finite two-component `offset`
(along and perpendicular to the line), and `rotation` with `relative` or
`absolute` mode and a finite angle in radians. Defaults are scale 1, offset
`[0,0]` and relative angle 0; omission applies those defaults, while explicit
null is invalid. Writers omit exact default transform fields, preserving signed
zero and absolute-zero rotation; see [drawing defaults](drawing-defaults.md).
Relative rotation follows
the tangent; absolute rotation requests a fixed drawing orientation. Upright
rotation is deferred because the pinned codec public model cannot represent it.

## Symbol sources and identities

The first source kind is SHX:

```json
{"id":2,"kind":"shx","fileName":"custom.shx"}
```

The mandatory filename is a nonempty basename, preserving case. It cannot be
`.` or `..`, contain a slash, backslash or NUL, or be replaced by a URI.
Availability is unknown until an editor resolves it. A missing external file
does not invalidate the drawing; a missing local source or text-style reference
does. An editor that attempts resolution can report a missing asset.

OCDraw stores `symbolSources` alongside the other local tables and requires
`header.nextSymbolSourceId`, including when the table is absent or empty.
Its source IDs use the existing `u32` table identity convention. IFCCAD stores
independent `ifccad::symbolSource` roles below
`/cad/d<drawing>/symbolSource/<id>` and requires the drawing's own
`nextSymbolSourceId` watermark. IFCCAD markers use complete `symbolSource` and
`textStyle` paths, rather than OCDraw's numeric reference fields. Zero and
full-width `u64` identities remain exact; foreign or noncanonical paths fail.

The source is a closed tagged value, leaving room for a future approved source
kind. Optional URI/hash locators, embedding and non-SHX sources are deferred to
[issue 15](https://github.com/OpenAEC-Foundation/ifccad/issues/15).
Unknown core fields are rejected today; no speculative extension payload is added.
The Explorer offers source inspection and links, with no SHX upload input.

## Bounds and appearance

The host curve retains its geometric enclosure and numerical conversion proof.
Marker outlines contribute separate evidence. ByLayer, ByBlock and layer `0`
inside a definition are resolved per occurrence, including nested inserts; a
definition-only cache cannot substitute for those contexts. Point, Hatch,
Text/MText and viewport display geometry do not acquire line pattern markers.
Unused complex definitions do not change a resolved simple curve's bounds.

Text markers use the existing fontless text estimator. Nominal height is the
positive fixed style height multiplied by marker scale, or marker scale when
the style height is variable. The offset is not multiplied by marker scale.
Global and entity pattern scales apply to the entire marker contribution.
A conservative rotation-independent radius inflates the curve enclosure, then
uses the existing outward block transforms. This is an estimate, not a certified
glyph envelope. Empty text contributes no footprint.

SHX outlines are unavailable in the native core. Their presence makes the
derived scope assessment Partial. A valid stored producer box and its
Enclosing/Estimated claim remain intact, separately from this evidence. The
validator still rejects known curves or block occurrences outside that box;
estimated text outlines are not a hard containment requirement. Derived Partial
is never stored as a producer quality. Explicit bounds recomputation clears a
partial box and marks wholly estimated text bounds Estimated. Numerical failure
still fails atomically. Existing stricter opaque/unknown-geometry rules remain.

Inspection reports `linePatternReasons`, `enclosureVerified` and
`safeForNegativeQuery` alongside the stored claim. An unresolved symbol envelope
must not become a verified enclosure or support negative spatial queries.

## CAD conversion and qualification

Converters classify ordinary and shape STYLE roles before binding markers.
Text resolves to ordinary STYLE handles; SHX sources resolve to shape-file
STYLE handles. Separate source IDs produce separate anonymous shape records,
including equal filenames. Missing, contradictory or wrong-role internal
references are fatal. Source paths retain their basename with one located
locator-loss diagnostic per source; Reject refuses that loss.

Qualified marker values map through shared identity-free CAD scalar helpers.
The current proven selector range is 0..32767 on the CAD target. Negative CAD
selectors, larger native selectors and unsupported active style/source features
replace the complete definition with named continuous under Allow and receive a
located diagnostic. Reject refuses even unused unsupported definitions. No
dash-only partial approximation or unsigned reinterpretation is used. Nonfinite
values and invalid periods remain fatal, independently of loss policy.

AC1032 DXF/DWG tests check resolved STYLE relationships, marker content,
progression, scale, offsets and relative/absolute rotation through production
native readers. A manually authored DXF fixture independently specifies marker
flags and degree values. This qualification uses the dependency pin and explicit
patch documented in each converter's coverage contracts. Upright flags remain
outside this public-model boundary; see
[codec issue 108](https://github.com/HakanSeven12/opencadcodec/issues/108).

The Explorer entity catalogue contains continuous, dashed and GAS text patterns.
The retained IFCCAD `examples/ifccad/overview.ifcx` regression asset also contains
`ltypeshp.shx` shape 132, custom external shape 42, duplicate/unused sources,
mixed and empty definitions, and Model/Paper/block contexts. It is outside the
chooser and remains unchanged for historical evidence. The embedded editor may
resolve its own libraries; a successful preview does not certify every glyph.
See [example reproduction and extension](../examples/README.md) for the current
catalogue. Focused standalone authoring programs `write_line_patterns` and
`write_ifccad_line_patterns` remain available.
