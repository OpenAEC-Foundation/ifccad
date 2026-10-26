# IFCCAD Text and MText

IFCCAD supports authored Text/MText, drawing-local text styles, typed rich
content and strict native readback without a CAD or font runtime in the core.
The active schemas, code/tests and directional converter coverage define the
implemented contract. OCDraw is [parked](ocdraw-parking.md).

Current dependency qualification (2026-10-16): codec `325fa98d2b881f48e106a830d5cf161624f63131`
plus the explicit Dimension text-position repair; viewport repairs are upstream.
The [current audit](geometry/opencadcodec-update-2026-10-16.md) supersedes dated
pin/transport limitations below, including paper grid and DXF ShadePlot gates.

Text has four distinct layouts: anchored, whole-text middle, aligned and fit.
Placement, rotation, independent mirrors, shear and signed thickness remain
authored values. Literal runs retain their boundaries and decorations through
native IO; an empty sequence is valid. Native strings do not interpret CAD codes
or dynamic fields. MText stores ordered paragraphs with runs, tabs, intra-paragraph
line breaks, column breaks and typed fraction/tolerance stacks. Paragraph and
character overrides remain separate, including explicit false/zero and empty
tab-stop resets. Effective formats are derived without rewriting authored presence.

Native MText supports all nine attachments, horizontal/vertical/by-style flow,
optional wrapping, three closed column states and backgrounds with none/color/
canvas fill, opacity, frame and absolute/relative padding. Overflow is reported;
content, column count and authored heights are not silently changed or clipped.
Font requests are symbolic, including family/CAD/big-font selectors and face,
charset and pitch requests. Styles are not resolved or embedded. Unused records,
local zero-valid IDs and deletion watermarks survive native IO. Styles do not
introduce a current-style workspace requirement.

IFCCAD uses `IfccadTextStyleId(u64)`, `IfccadTextStyle`, `IfccadText` and
`IfccadMText`, with Text/MText variants in its own native entity enum. The style
counter is independent of every other domain; IDs include zero and full-width
values beyond 2^53. `/cad/dN/textStyle/N` nodes are drawing children, including
unused styles. A text payload contains a complete same-drawing `style` path,
alongside the existing entity appearance and separate geometry placement.
`ifccad::text`, `ifccad::mText` and `ifccad::textStyle` replace whole attribute
values on LaterWins composition. No run/paragraph graph nodes are introduced.
The [IFCCAD contract](../schemas/ifccad/experimental-contract-0.1.0.md#text-and-mtext)
and [local-value supplement](../schemas/ifccad/supplemental-values-0.1.0.schema.json)
define its independent provisional mapping.

`assess_ifccad_document_bounds` returns derived enclosing/estimated/partial
quality and independent enclosure evidence. IFCCAD stores absent bounds by
omission; `boundsQuality` requires a box, permits enclosing/estimated, and
defaults to enclosing when omitted with a box. Atomic explicit recomputation
updates boxes and qualities together, including unused/nested definitions.
Glyph estimates are never verified enclosures; known primitive subsets and
numeric failures remain checked even when bounds are absent or a neighbour is
opaque. Empty glyphs do not invent insertion-point geometry for a nonempty block.

## Shared text value and extent preparation

`ifccad::text` provides format-independent authored values, intrinsic validation,
property-wise inheritance and fontless extent preparation. These values carry
no drawing IDs, ownership, CAD runtime, loaded font or physical encoding. Native
adapters use the same intrinsic checks rather than duplicating them.

The character chain is style -> MText basis -> paragraph -> inline. Properties
replace inherited values rather than multiplying them. Relative heights refer
to nominal MText height; absolute heights retain their scope distances.
Paragraph layout uses a separate basis independent of the preceding paragraph.
Tabs, breaks and stacks remain typed structural content. Explicit false, valid
zero and empty tab-stop resets remain distinct from absent overrides.

Full-context validation uses the authored entity-wide paragraph basis and the
actual effective character formats. Empty lines use their paragraph basis;
fully overridden unused values do not invent geometry. Resolving a format
neither loads fonts nor rewrites authored records.

Finite indentation may exhaust or exceed the wrap/column width, including the
first-line width. These are authored overflow states, not invalid numbers.
Positive nominal height and explicit wrap/column width, finite indent products
and finite derived arithmetic remain required. Storage and CAD conversion retain
the content and factors; no width/indent clamping or text omission is implied.
Fontless estimates report actual overflow without clipping, independently of
unassessed glyph layout. The active-next paragraph-overflow fixture covers strict
JSON/CBOR storage; bounded synthetic AC1032 DWG/DXF tests cover conversion.

### Fontless estimates

`estimate_text_extent` and `estimate_mtext_extent` return local and owning-scope
estimated boxes, status and reasons. Nonempty font-dependent results are
`Estimated`; a missing contribution such as an unknown physical frame stroke
gives `Unavailable` with only a known subset box. They cannot certify glyph
enclosure or negative spatial queries.

The heuristic counts Unicode scalars, including combining and bidi characters,
at two effective heights per scalar advance. It includes width/tracking/shear,
greedy word/atomic-tab-field wrapping, stacks, explicit breaks, paragraph spacing
and stored column state. It neither clips overflow nor inserts authored breaks,
changes column counts or allocates an array of generated columns.

Alignment, mirrors, rotation and Text thickness preserve the active anchor.
Placement uses the shared outward transform kernel. Vertical flow has an
explicit estimate profile rather than an authored 90-degree rotation.
Relative background padding uses nominal height. Frame stroke must already be
expressed in scope coordinates; medium size alone supplies no conversion.
Arithmetic overflow or positive-height underflow returns a typed error.

`CoordinateFrame3::try_from_normal_arbitrary_axis` explicitly constructs a frame
from a normal using scaled normalization and the strict normalized-normal 1/64
branch. Native loading and `try_new` preserve supplied axes without normalization.
This construction helper does not itself certify CAD conversion accuracy.

## Bounds and evidence

Explicit recomputation produces enclosing primitive bounds, estimated bounds
when every contribution is available but text is estimated, or null bounds with
derived partial coverage when a contribution is unavailable. Empty text has no
extent. Quality propagates through shared/nested and unused block definitions;
opaque content retains its transitive null-bounds rule. Numerical failures remain
hard errors even beside unknown contributions. Preparation stages boxes and
qualities before mutating the document.

Stored `boundsQuality` is a producer declaration: enclosing or explicit estimated.
Missing quality with a box means enclosing. Partial is derived, never a stored
box quality; quality without a box is invalid. Encoding retains a valid supplied
declaration. A fontless reader cannot confirm enclosing glyph claims or require
them to match its heuristic box. Proven primitive contributions remain checked.
`assess_ifccad_document_bounds` returns independent coverage and enclosure evidence;
the Explorer displays that separately from the producer declaration. Estimated
or partial results cannot justify negative spatial queries or safe glyph culling.

Frame stroke uses declared drawing units or a fixed physical Paper plot mapping.
Paper does not borrow the drawing unit or infer a scale from medium dimensions.
Unitless, fit-to-area, pixel and unresolved ByBlock stroke contexts remain
unavailable when a frame needs physical extent. Explicit zero stroke is known
without a scale. The estimator neither loads fonts nor claims letter contours.

## Qualified CAD subset

Both IFCCAD directions use the pinned `cad-text` helper and the same loss policy,
strict native readback and hard per-coordinate-domain numerical checks as other
entities. They preserve mapped text styles, including unused definitions, and
mixed draw order in Model, Paper and blocks. Text uses active OCS anchors;
MText uses its WCS anchor/direction convention. Assessment covers qualified
anchors/active baselines and parameter preparation, not whole glyph geometry.
`text_assessment()` and `geometry_assessment().unassessed_sources()` expose that
boundary even when there are no semantic diagnostics. Handle mappings are
conversion-local; unallocated STYLE handles do not merge named records.

CAD is narrower than native IO. Unsupported active semantics skip the whole text
entity under Allow; Reject refuses the loss. Current restrictions include
annotative contexts, dynamic fields, attributes, TEXT strike-through, independent
MText mirrors, unqualified intra-paragraph line-break file mapping, advanced stack
position/size variants, decimal-tab marker/local spacing profiles, unsupported
font options and background transparency. Named/unsupported indexed inline or
background color metadata has no silent RGB fallback. Known inactive state,
global paragraph-basis expansion, spacing/padding dependency changes and authored
formatting normalization produce explicit diagnostics. Missing native last-height
history becomes CAD's required 2.5 default with a diagnostic. Native formatting
presence survives native IO; CAD state-machine normalization can change future
editing dependencies and is not described as exact authored roundtrip.

CAD dynamic manual columns end in an automatic column that takes the remaining
content. Its stored final height is a cache/sentinel, including negative DWG
values and zero DXF values; import uses the existing native `Auto` entry rather
than inventing a fixed height from its magnitude or cached extents. Earlier
heights retain positive fixed distances. Auto exports as zero. A native fixed
final cap remains valid native data but cannot be represented by CAD manual
columns, so CAD export diagnoses and skips that whole text under Allow; Reject
refuses the loss. See the [qualification and evidence](text/mtext-manual-column-tail.md).

Simple Text/MText pass production native IO and actual DXF/DWG exchange tests.
Placement/content/style/column/background tests qualify a bounded profile;
they do not establish a font engine, application rendering, every file version
or full CAD/native parity. See the converter's
[FROM-CAD](../crates/ifccad-convert/docs/FROM-CAD-COVERAGE.md) and
[TO-CAD](../crates/ifccad-convert/docs/TO-CAD-COVERAGE.md) inventories and
[cad-text](../crates/cad-text/README.md). Fonts remain requested rather than bundled.

The [IFCCAD example](../examples/ifccad/text.ifcx) includes
Text/MText in Model, Paper and a local block, explicit paragraph colour and
estimated bounds. Generate it with `cargo run --example write_ifccad_text --
<output.ifcx>`. That producer uses strict native readback before writing.
IFCCAD actual-file tests qualify these owners independently and cover manual
Auto-tail columns in DXF and AC1032 DWG. A DXF degree/radian roundtrip can change
a stored angle's last bit; native IO remains exact. Formatting scope changes
are located, rejectable `text-dependency` diagnostics rather than silently
claimed authored parity. IFCCAD accepts the exact version-1 AcadAnnotative flag
0 as disabled ordinary text; residual XDATA still receives common-metadata loss
evidence. Active/unknown annotation contexts remain outside the bounded profile.
Both IFCCAD directions expose `text_assessment()` and explicit unassessed glyph
geometry. Loaded-source conversion compares closed text payloads and their
defaults without changing source bytes; exact numeric projection loss is hard
under both policies. The integrated IFCCAD qualification uses the main dependency
base `325fa98d2b881f48e106a830d5cf161624f63131` and its explicit Dimension
text-position repair. Earlier OCDraw slice evidence remains historical.
See the two IFCCAD converter coverage contracts for limits.

IFCCAD concrete character/background colors use IfccadColor with RGB fallback
and optional indexed/named metadata. Qualified RGB/ACI text identity passes through the shared
CAD color rules; richer/unqualified text color metadata still obeys the bounded
whole-text restriction. Scalar color support does not certify glyph appearance.
See [presentation](presentation.md) for migration and numerical policy.


Explicit native Text/MText colors also support the symbolic Foreground choice,
including character/paragraph/inline overrides and explicit background fills.
Qualified CAD indexed color 7 retains that meaning through DXF/AC1032 DWG in both
routes; ordinary RGB white/black remain fixed. Foreground is different from the
existing canvas-background-fill choice. See [Foreground](presentation.md#foreground-color)
for resolution and the closed wire value; richer named/custom text colors retain
their existing bounded-profile restrictions.

## Text in line patterns

Complex line patterns reference the same ordinary local text-style domain but
store literal marker strings and their own scale, offset and relative/absolute
rotation. They do not introduce Text/MText entities or rich text markup. The
fontless marker estimate reuses the Text extent estimator, with nominal height
from fixed style height and marker scale. Marker numerical coverage remains
unassessed beside the host curve proof. See [line patterns](line-patterns.md).
