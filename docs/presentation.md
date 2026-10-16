# Drawing presentation and CAD exchange

Current dependency qualification (2026-10-16): codec `325fa98d2b881f48e106a830d5cf161624f63131`
plus the explicit Dimension text-position repair; viewport repairs are upstream.
The [current audit](geometry/opencadcodec-update-2026-10-16.md) supersedes dated
pin/transport limitations below, including paper grid and DXF ShadePlot gates.

IFCCAD and OCDraw retain independent models, schemas, identities and converters.
Presentation values describe authored display and plot intent; they do not certify
glyph contours, point-symbol extents, projected curves or renderer output.
The core remains independent of opencadcodec. The CAD-only
`cad-presentation-convert` companion shares scalar decisions and strict XRecord
grammar; each converter owns native reference binding and located loss policy.

## Native contract

| Meaning | IFCCAD | OCDraw |
| --- | --- | --- |
| Color | `IfccadColor`: Foreground or concrete RGB with optional indexed/named identity | `DrawingColor`: the same independent value meaning |
| Layer presentation | Description, visible, frozen, locked, plottable, frozen in new viewports | Existing corresponding layer fields |
| Entity visibility | Common native `visible`, default true | Common entity `visible` |
| Point symbols | Optional drawing `PointDisplay` | Existing optional drawing `PointDisplay` |
| Block metadata | Description, anonymous, explodable, uniform scaling | Existing corresponding definition fields |
| Per-viewport layer state | `layerOverrides` rows referencing native layers/patterns | Corresponding typed relational rows |
| Viewport plot shading | Optional mode only | Optional mode only |
| Paper camera | Orthographic or perspective | Orthographic or perspective, now mapped by the converter |

IFCCAD layer visibility/plottability default true; frozen/locked/new-viewport
freeze default false. Description is optional. Entity visibility defaults true.
Block description defaults empty, anonymous/uniform scaling false and explodable
true. A uniform definition requires exactly equal signed insertion scales:
`[-2,-2,-2]` is uniform; `[-2,2,2]` is not. Ordinary anonymous definitions remain
definitions, including unused ones. Native names are hints; reserved Model/Paper
names are restricted when allocating CAD blocks, not used to infer native roles.

For concrete colors, RGB is the fallback. Indexed identity is `(system, u64 index)`;
named identity is `(catalog, name)`. Identity strings must be nonempty. Native
identity ranges are broader than the qualified CAD palette. IFCCAD's physical
color shape is `{rgb:[r,g,b], indexedColor?:{system,index},
namedColor?:{catalog,name}}`. Concrete Text/inline/background colors and editable opaque
common appearance use this shape; Foreground uses the separate closed choice
below. Provider payload bytes remain untouched.

Point display stores dot/hidden/plus/cross/shortLine with independent circle and
square enclosures. Size is default five percent, a positive absolute Model length,
or a positive viewport percentage. CAD PDSIZE zero is the default choice;
positive/negative values denote absolute/percentage sizes. Glyph settings do not
change the Point anchor or add bounds for the display symbol. Invalid CAD display
settings receive loss while supported Point geometry survives.

Viewport rows have one layer reference and optional color, opacity, line pattern
and lineweight, plus `frozen`. Duplicate layer rows, dangling references and empty
no-op rows are invalid. Writers order rows by native numeric layer identity.
`frozen:false` does not thaw a globally frozen layer. Common entity visibility,
viewport on/off and view locking are separate meanings.

Viewport `plotShadingOverride` contains only `AsDisplayed`, `Wireframe`, `Hidden`
or `Rendered`. Absence has the qualified AsDisplayed meaning. Quality/DPI stays
on effective layout plot settings; viewport rendering mode is independent.


## Foreground color

Both independent native color types have an exclusive Foreground choice. Its
closed wire value is `{"kind":"Foreground"}`, with no RGB, indexed or named
metadata. Concrete colors keep the existing shape. Concrete indexed ACI 7
(ASCII-case-insensitive system name) is rejected; use Foreground. Literal RGB
black and white remain fixed colors, and readers never infer Foreground from RGB.
The provisional migration updates active examples and candidate conformance;
there is no compatibility reader for intermediate ACI-7 objects.

Resolve appearance inheritance and viewport overrides first, then resolve
Foreground against the occurrence's effective composited opaque sRGB background.
Select pure black or white with the larger luminance contrast ratio, choosing
black on a tie. For normalized sRGB component c, linearize as c/12.92 for
c <= 0.04045, otherwise ((c+0.055)/1.055)^2.4. Let
L = 0.2126 R + 0.7152 G + 0.0722 B; compare (L+0.05)/0.05 for black with
1.05/(L+0.05) for white. Black backgrounds yield white and white backgrounds
yield black. The CAD-independent `ocdraw::color::foreground_srgb` helper applies
this native rule. It is not AutoCAD's complete adaptive palette algorithm.

Do not store resolved theme RGB in the drawing. Foreground is distinct from
MText's canvas-background-fill choice and from ByLayer/ByBlock modes. The existing
top-level ByBlock resolution boundary remains unchanged. Inspection reports the
symbolic value; this slice adds no renderer or theme controls.

Qualified CAD index 7 imports as Foreground before palette lookup and exports as
index 7, including CTB key 7. This applies to layer/entity appearance, qualified
inline/background text color and the C3/index-7 viewport override payload. Fixed
RGB white and black stay true color. A source named identity accompanying index
7 is unsupported on the exclusive Foreground value: Allow reports located loss
and Reject refuses it. Editable opaque common appearance uses native Foreground
without rewriting provider payload bytes. An unrepresentable source common
appearance may remain in its existing preserved snapshot instead of claiming an
editable native value.

Both format routes have production-reader tests and actual DXF/AC1032 DWG
roundtrips for these color locations. This proves data exchange, not external-app
visual output or CTB printing. Method C5, ByPen, None/257 and broader palette or
CTB/STB contents remain outside this slice; see
[color and plot-style follow-up](https://github.com/OpenAEC-Foundation/ifccad/issues/16).

## Provisional-format migration

The active provisional IFCCAD profile replaces legacy hexadecimal color strings
with concrete color objects, viewport-local visibility with common entity
visibility, and `frozenLayers` with `layerOverrides`. Both active formats replace
the combined viewport shading/quality object with a mode value. Removed fields,
unknown core fields, explicit null for non-null optional IFCCAD values, and invalid
values are rejected. These are active-schema changes; released numbered
conformance collections were not rewritten. Preserved provider payloads retain
their own schema and byte contract.

## CAD scalar policy

Qualified explicit CAD colors are RGB and ACI 1–255. ACI 7 maps to symbolic
Foreground and back to ACI 7, without a stored RGB fallback or index identity.
Other consistent ACI identity can
be emitted as ACI; incompatible or unsupported indexed metadata falls back to
authored RGB with located loss. Complete named identity can use the qualified
layer/entity fields. Layer/entity catalog delimiters and unqualified text/color-book
representations retain their specific restrictions. Unrepresentable required
layer/entity appearance is diagnosed and the dependent item skipped; no invented
white color is substituted.

Opacity is finite in `[0,1]`. A CAD transparency byte is decoded as
`1 - byte/255`; exact recognition returns every one of the 256 bytes unchanged.
Other authored opacities use CAD's upward transparency rounding and receive
semantic modification diagnostics when their value changes. Lineweight is a
finite nonnegative number in millimeters; CAD output selects the nearest of the
24 standard weights, with the lower binary64-distance tie inside the table and
the nearest endpoint outside it. Nonstandard values remain valid native values
but quantization is diagnosed. These presentation losses are independent of
geometric tolerance. Reject refuses semantic substitutions.

## Viewport override transport

| Field | CadDocument | DXF / AC1032 DWG |
| --- | --- | --- |
| Per-viewport freezing | Frozen layer handles | Qualified |
| Concrete RGB / ACI color | Layer extension XRecord, 335 viewport + 420 value | Qualified method-tagged RGB/ACI |
| Opacity | 335 + packed alpha 440 | Qualified explicit alpha |
| Line pattern | 335 + typed pattern handle 343 | Qualified late-bound reference |
| Lineweight | 335 + integer 91 | Qualified explicit hundredths of a millimeter |
| Named/custom override identity | Native value is broader | Not qualified; located loss remains |
| Viewport ShadePlot | Independent mode 0–3 | DXF/DWG qualified on the current pin; PR #107 is merged |
| Paper grid behavior flags | Stored public fields | Both file routes pending codec PR #106 |

XRecord 420 differs from ordinary entity true color: the high byte is the
AcCmColorBase method. RGB uses C2; concrete ACI uses C3 and a valid palette index.
Inherited/custom/unknown method tags and reserved payload bits are refused rather
than reinterpreted as RGB. Alpha methods 2 (DXF) and 3 (the codec DWG form) carry
an explicit alpha byte. Code 91 carries hundredths of a millimeter, not a DWG
lineweight table index. See Autodesk's
[packed-color API](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcCmColorBase.html),
[color methods](https://help.autodesk.com/cloudhelp/2026/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_Colors_ColorMethod.html)
and [lineweight units](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-Core/files/GUID-21DF5F82-4F3A-4F93-8FD6-89A942799468.htm).

Strict known-section parsing avoids the codec pair scanner crossing unknown
sections. Incomplete streams, extra values, conflicting duplicate values and
unavailable targets remain loss or invalid structure. Equal duplicate values are
consumed without duplicate native rows. Only completely consumed XRecord and
dictionary containers with qualified metadata are exempted from residual source
inventory; mixed containers retain their original container-level diagnostic.
Supported siblings still map. Target handles bind after construction, and skipped
viewports receive no emitted override or workspace references.

All eight codec-dependent companions select the audited 325fa98 pin and the
explicit Dimension text-position repair. Upstream #103, #106 and #107 are
included; viewport off-state, paper grid flags and DXF ShadePlot are qualified.
The browser still compares requested ShadePlot with physical readback and
reports changed or unconfirmed values. Older CAD versions remain separately limited.

## Perspective qualification

Both converters preserve target, unnormalized target-to-camera direction,
DCS view center/height, twist, lens length in millimeters and depth clips.
Paper frame and clip coordinates stay in their Paper domain. Model insertion
unit codes 0, 1 and 4 do not reinterpret the lens or normalize camera distance.
Shared native camera validation rejects invalid lens, zero/overflowing direction
and invalid clip order; dormant orthographic zero lenses remain valid.

The hand-authored DXF/JSON camera references contain axial and oblique cases and
independently specified projected landmarks. Each route uses production native
readback and actual DXF/DWG exchange. Physical examples establish the overall
orthographic Paper canvas separately from authored cameras. This preserves the
existing qualified canvas profile; it does not infer active Paper selection or
repair an incomplete codec graph. These tests are not external-application
certification or proof of projected-curve/rendering accuracy. Existing geometry
tolerance and numerical assessment contracts remain unchanged.

The inspector exposes these native values and relational references. It adds no
CAD editing controls or new rendering behavior. For related boundaries see
[workspace state](workspace-state.md), [layout output](layout-output.md),
[text](text.md) and each converter's FROM/TO coverage contracts.
