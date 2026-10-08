# cad-text

Shared CAD text preparation for the independent OCDraw and IFCCAD adapters.
The library has no drawing identities, ownership, native codec, filesystem IO,
font resolver, renderer or conversion loss policy. The core `ocdraw::text`
module stays independent of opencadcodec. Both native/directional routes use
this bounded profile through their own adapters and qualification tests.
Shared helper success never establishes route parity.

`parse_text` and `parse_mtext` turn supported CAD control codes into typed values.
TEXT and MTEXT have different grammars: ordinary TEXT braces/backslashes are
literal, while MTEXT uses scopes. Unknown control codes, malformed escapes,
dynamic field syntax and runtime resource-limit failures remain explicit.

File-safe literal emission accounts for the codec's CIF decoding before MTEXT
parsing: percent uses the qualified `%%%%` spelling, caret controls are separated
by harmless scope boundaries, and a literal backslash before U+XXXX is fenced
from premature Unicode decoding. Actual DXF/DWG readback tests cover these cases;
raw grammar tests alone do not establish that boundary.

Parsed character-basis factoring is exposed as `character_basis_normalized`.
Prepared adapters report `CharacterBasisNormalized` or
`AuthoredFormattingNormalized` when CAD state-machine representation changes
authored override scope/presence, including a no-op entity-color reset. Current
effective formatting is not a certificate for future editing dependencies.
Route adapters enforce these issues through their own Allow/Reject policy.
Parsing retains significant whitespace, blank/trailing paragraphs, explicit
character overrides and column breaks without changing entity kind.

MTEXT source scopes may cross paragraph breaks. The parser resolves source
carry-over into independent paragraph/inline overrides and restores the source
context when a group closes. Identifiable global character properties remain
global; shared explicit paragraph properties may be factored locally. Current
style values never become inferred fixed overrides. Relative height codes are
accumulated in the source context, then reference the nominal native height.

Emitters escape literals, including strings that resemble formatting or field
syntax. TEXT uses its legacy percent controls. Unsupported target semantics
return errors rather than silently becoming plain strings. The simple default
profile is exercised through the pinned production DXF and AC1032 DWG codecs.
That evidence does not establish application rendering or a complete native
document roundtrip.

## Boundary and evidence

`import_text_style`/`prepare_text_style_to_cad` retain supported symbolic font
requests and creation metadata, including unused-style data provided by callers.
Missing installed fonts are not inspected. Annotative, external and shape-file
style contexts are outside this profile. Local identity/name/reference checks
remain the drawing adapter's responsibility.

`prepare_text_from_cad`, `prepare_mtext_from_cad` and their target preparations
select active anchors, typed layouts and placement. TEXT anchors are OCS;
MTEXT insertion is WCS. A coherent retained MTEXT direction avoids unnecessary
angle reconstruction. Native MTEXT independent mirror flags remain unsupported
at this CAD boundary rather than being guessed from a flipped normal.

Preparations return `GeometryPair` inputs for `cad-geometry-convert`'s hard
tolerance and nested-occurrence checks. These cover anchor/represented baseline
or extrusion parameters, **not** glyph contours or laid-out text geometry.
`glyph_geometry_unassessed` remains true. The caller resolves its actual
coordinate domain and checks the pairs; preparation does not accept a numerical
deviation on the caller's behalf.

`CadTextIssue` identifies represented current values whose authoring dependency
changes, or dormant source state omitted by the bounded native profile. Adapters
must locate/classify every issue and apply their independent Allow/Reject policy;
discarding this evidence does not produce a lossless conversion.

## Current restrictions

- Local paragraph before/after/exact-spacing units and decimal tab markers remain
  unqualified (opencadcodec issues 87 and 90). No unit factor or separator is guessed.
- Native intra-paragraph line breaks, decimal-tolerance stacks and nondefault
  stack size/position combinations need separate CAD file qualification.
- TEXT strikethrough has no qualified pinned target code; native support is separate.
- Font delimiter escapes, inline combined/big-font requests and style font-face
  flags absent from the pinned public table type remain explicit restrictions.
- Dynamic auto-height count cannot be recovered from cached extents. Manual
  count/list mismatch is invalid. The last manual column is automatic: its
  finite stored height is ignored, including zero DXF sentinels and negative
  DWG cache values. Earlier heights remain positive fixed distances. Native
  Auto tails export as zero; a native fixed final cap is unsupported in CAD.
  See the [source qualification](../../docs/text/mtext-manual-column-tail.md).
- Column reference width must agree with the supported derived-total profile.
- Background transparency packing is not yet qualified; non-opaque fill is
  restricted. Default-opacity explicit/canvas fill and frame-only are distinct.
- Expanding a native global paragraph basis into CAD paragraph properties,
  mapping absolute padding to a relative border factor, and mapping absolute
  spacing to nominal-height factors carry explicit dependency-change issues.

The dependency is pinned to `063c10671fe7833d562f772159771318c7a0ebb9`, matching
both converters and the shared CAD geometry helper after integration with the
codec update already on main. Its remaining viewport-off-state repair is
independent; patched results are not evidence for unmodified upstream. The text
slice adds no later codec upgrade, font engine or measurement.
