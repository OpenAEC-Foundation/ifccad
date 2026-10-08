# OCDraw and IFCCAD Text and MText

OCDraw supports standalone authored Text/MText, local text styles, typed rich
content and strict native readback. Its core needs neither IFCX nor a CAD or font
runtime. IFCCAD independently supports the same authored text values through its
own identities, IFCX payloads, strict reader/writer and CAD adapters. Shared
helpers do not establish route coverage; each route has its own native and
actual-file tests.

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

`DrawingTextEntity`, `DrawingMTextEntity` and `OcdrawTextStyleId` have their own
OCDraw identities and reference checks; text does not masquerade as a curve.
`OcdrawBuilder::add_text_style`, `add_text` and `add_mtext` allocate atomically and
use the existing shared entity counter and ordered scope ownership. Reader
accessors expose typed records. The registered `textStream` and `mTextStream`
use ordinary row columns for common values and closed nested rows for layout and
content. The [logical contract](../schemas/ocdraw/logical-contract-0.1.0.md#text),
registry, JSON mapping and active schema define the provisional wire contract.

IFCCAD uses `IfccadTextStyleId(u64)`, `IfccadTextStyle`, `IfccadText` and
`IfccadMText`, with Text/MText variants in its own native entity enum. The style
counter is independent of every other domain; IDs include zero and full-width
values beyond 2^53. `/cad/dN/textStyle/N` nodes are drawing children, including
unused styles. A text payload contains a complete same-drawing `style` path,
alongside the existing entity appearance and separate geometry placement.
`ifccad::text`, `ifccad::mText` and `ifccad::textStyle` replace whole attribute
values on LaterWins composition. No run/paragraph graph nodes are introduced.
The [IFCCAD contract](../schemas/ifccad/experimental-contract-0.1.0.md#text-and-mtext)
and [closed text-value schema](../schemas/ifccad/text-values-0.1.0.schema.json)
define its independent provisional mapping.

`assess_ifccad_document_bounds` returns derived enclosing/estimated/partial
quality and independent enclosure evidence. IFCCAD stores absent bounds by
omission; `boundsQuality` requires a box, permits enclosing/estimated, and
defaults to enclosing when omitted with a box. Atomic explicit recomputation
updates boxes and qualities together, including unused/nested definitions.
Glyph estimates are never verified enclosures; known primitive subsets and
numeric failures remain checked even when bounds are absent or a neighbour is
opaque. Empty glyphs do not invent insertion-point geometry for a nonempty block.

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
`assess_ocdraw_document_bounds` returns independent coverage and enclosure evidence;
the Explorer displays that separately from the producer declaration. Estimated
or partial results cannot justify negative spatial queries or safe glyph culling.

Frame stroke uses declared drawing units or a fixed physical Paper plot mapping.
Paper does not borrow the drawing unit or infer a scale from medium dimensions.
Unitless, fit-to-area, pixel and unresolved ByBlock stroke contexts remain
unavailable when a frame needs physical extent. Explicit zero stroke is known
without a scale. The estimator neither loads fonts nor claims letter contours.

## Qualified CAD subset

Both OCDraw directions use the pinned `cad-text` helper and the same loss policy,
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
or full CAD/native parity. See both OCDraw converter coverage contracts and
[cad-text](../crates/cad-text/README.md). Evidence uses base revision
`063c10671fe7833d562f772159771318c7a0ebb9` with the explicitly selected existing
viewport-off-state repair; it is not evidence for an unmodified dependency.
No size/exchange measurement was run for this slice.

Generate the [Explorer example](../examples/ocdraw/text.ocdraw.json) with
`cargo run --example write_text -- <new-output.ocdraw.json>`. The producer strictly
reads its output and refuses overwrite. Fonts remain requested rather than bundled.

The independent [IFCCAD example](../examples/ifccad/hello-text.ifcx) includes
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
base `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` and its explicit viewport-off
repair; the earlier OCDraw slice evidence above records its original base.
See the two IFCCAD converter coverage contracts for limits.
