# OCDraw Text and MText

OCDraw supports standalone authored Text/MText, local text styles, typed rich
content and strict native readback. Its core needs neither IFCX nor a CAD or font
runtime. The independent IFCCAD integration is deferred until the OCDraw slice
has been tested on main; it does not acquire native text support from these helpers.

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
