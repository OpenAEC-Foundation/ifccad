# Shared text value and extent preparation

`ocdraw::text` provides format-independent authored Text/MText values, intrinsic
validation, property-wise inheritance and fontless extent preparation. This is
an implementation building block used by the native OCDraw model and its bounded
CAD route. Independent IFCCAD integration is deferred until the OCDraw slice has
been tested. Models and codecs stay independent; see [text support](text.md).

The values carry no drawing IDs, scope ownership, CAD runtime, loaded font or
physical encoding. Explicit `false`, zero where valid, and empty tab-stop reset
lists remain distinct from omitted overrides. Font requests remain symbolic;
resolving a format neither loads a font nor rewrites authored records.

The character chain is style → MText basis → paragraph → inline. Properties
replace instead of multiplying inherited values. Relative heights reference
nominal MText height; absolute heights retain their scope distances. Paragraph
layout uses a separate basis and has no dependence on the preceding paragraph.
Structural tabs/breaks/stacks never hide inside literal strings.

Full-context validation includes the authored entity-wide paragraph basis and
the actual effective character formats. Empty lines use their paragraph basis;
fully overridden unused values do not invent geometry. Independent document
adapters must consume that same intrinsic validation rather than duplicate it.

## Fontless estimates

`estimate_text_extent` and `estimate_mtext_extent` return a local estimated box,
an owning-scope box, status and reasons. Nonempty font-dependent results are
always `Estimated`; missing contributions such as an unknown physical frame
stroke make the result `Unavailable` with only a known subset box. These boxes
are navigation aids and cannot certify glyph enclosure or negative spatial queries.

The explicit initial heuristic counts Unicode scalars, including combining and
bidi characters, at two effective heights per scalar advance. It uses roomy line
bands, accounts for width/tracking/shear, greedy word/atomic-tab-field wrapping,
stack sizes, explicit breaks, paragraph spacing and stored column state. It never
clips overflow content, inserts authored breaks or changes the column count.
Column counts do not cause allocation of an array of generated columns.

Alignment, mirrors, separate rotation and Text thickness preserve the active
anchor. Placement uses the existing shared outward transform kernel. Vertical
flow is an explicit estimate profile, never an authored 90-degree rotation.
Background padding uses nominal height when relative; frame stroke must already
be expressed in scope coordinates. Medium size alone cannot supply that conversion.
Arithmetic overflow/positive-height underflow yields a typed error rather than
an unknown-font explanation.

`CoordinateFrame3::try_from_normal_arbitrary_axis` is explicit construction from
a normal. It reuses the previous block arbitrary-axis construction, with robust
scaled normalization and the strict normalized-normal 1/64 branch. Existing
`try_new` and native loading still preserve supplied axes without normalization.
This constructor is not a CAD conversion accuracy certificate.
