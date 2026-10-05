# IFCX-CAD conversion proof

Experimental direct conversion between `ocdraw::ifcx_cad` and pinned
opencadcodec `CadDocument`. The core remains independent of opencadcodec. No IFCDR
package, block explosion or appearance resolver is used.

This incomplete adapter defaults to **Allow**: supported content is returned
with located diagnostics for omissions and modifications. **Reject** refuses
any diagnosed semantic loss. Errors distinguish invalid structure, unsupported
content, core validation, encoding, output readback, ID allocation and CAD
construction. It makes no losslessness claim for arbitrary CAD files or IFCX graphs.

## API

`IfcxCadDocument` is the central logical input/output between CAD conversion
and IFCX encoding. `cad_document_to_ifcx_cad_document` constructs it directly
with diagnostics and mappings; `ifcx_cad_document_to_cad_document` validates
and converts the supplied projection. Both require explicit direction-qualified options; use `Default::default()`
for the existing Allow policy.
They do not encode/parse an IFCX file as an intermediate step.

The encoded and source functions below are convenience routes. A loaded
`ValidatedIfcxCad` retains both the CAD document and its complete immutable
source graph. Its conversion route additionally diagnoses foreign graph content
and source numeric precision. Use this route to assess losses from a full IFCX
input. Projection-only conversion cannot assess discarded source context.

Core `encode_ifcx_cad_document` creates a fresh CAD-profile file. It does not
merge edits into a source graph. Original native downloads use source bytes.
See the [document lifecycle](../../docs/experiments/ifcx-cad-document-lifecycle.md)
for ownership, validation and output boundaries.

```rust
use ocdraw::ifcx_cad::ValidatedIfcxCad;
use ifcx_cad_convert::{ifcx_cad_source_to_cad_document, cad_document_to_encoded_ifcx_cad,
    IfcxCadTargetMetadata, IfcxCadConversionError};

fn convert(source: &ValidatedIfcxCad, metadata: IfcxCadTargetMetadata)
    -> Result<Vec<u8>, IfcxCadConversionError>
{
    let cad = ifcx_cad_source_to_cad_document(source, Default::default())?;
    let back = cad_document_to_encoded_ifcx_cad(cad.document(), metadata, Default::default())?;
    // Strict writer and production reader have validated these bytes.
    Ok(back.encoded().bytes().to_vec())
}
```

Target header and drawing ID are caller supplied. Outcomes expose diagnostics
and mappings; CAD outcomes also offer `into_document()`.

The experimental Rust API retains typed failure payloads:
`CoreValidation(IfcxCadReport)` replaces `CoreValidation(String)`;
`CoreEncoding(IfcxCadEncodeError)` distinguishes invalid input, serialization,
production-readback rejection and semantic readback mismatch;
`CoreReadback(IfcxCadReadError)` covers the converter's additional production load;
and `IdAllocation(IfcxCadIdAllocationError)` identifies the exhausted domain.
All four expose their underlying error through `std::error::Error::source()`.
The reader error retains its report through both `report()` and `source()`.
Update Rust error matches; native files and conversion policies are unchanged.
Diagnostic strings remain available for presentation.

Use `IfcxCadToCadOptions { loss_policy: IfcxCadLossPolicy::Reject }` for
source/document-to-CAD conversion and `CadToIfcxCadOptions` with the same
policy for conversion from CAD. These are distinct direction-specific types. `IfcxCadDiagnosticAction` distinguishes `Omitted`, `Modified` and
`Recovery`; `is_loss()` excludes uniquely established structural cache repairs.
Diagnostics are part of the result and should be presented to the caller.
The separate layer, layout, line-pattern, block-definition and entity mappings offer
`cad_handle(id)`, `ifcx_id(handle)` and read-only iteration. Layout mappings use
Layout object handles; block mappings use BlockRecord handles. IDs remain u64,
including values above JavaScript's safe integer range. Deterministic allocation
for a fixed enumeration does not promise persistence through CAD serialization.

Each fresh CAD import allocates all five IFCX-CAD ID domains from 1 with checked
core allocators and writes the resulting persistent watermarks. The layer
named `0` has no reserved numeric ID. Native read/edit/write preserves authored
IDs, including legacy numeric zero, and advanced watermarks; deletion never
resets them. Reimporting DWG/DXF creates a new allocation history, and conversion
mappings do not themselves persist that history inside the CAD file.

The source inventory accepts a layer's canonical `AcCmTransparency` XDATA only
when its single Integer32 exactly duplicates the mapped explicit layer transparency.
This is an alternate encoding of supported opacity, with no source mutation or
repair. Other owners, values, applications and undecodable payloads remain losses.

## Supported slice

- One Model layout and multiple Paper layouts with explicit tab order, optional physical media, unused layers and definitions. Native coordinate units support all 25 tokens; CAD Paper export supports unitless, inch and millimetre coordinates.
- XYZ lines, standard-XY circles and straight planar polylines. Polyline XY
  translation folds into vertices only for exact finite binary64 sums; Z becomes
  elevation. Position/path/plane survive, local-origin decomposition does not.
- True RGB, named signed-length patterns (including unused/empty definitions), supported CAD hundredth-mm line weights and
  exactly byte-representable opacity. ByLayer/ByBlock/Explicit are independent
  for each entity property and remain stored. RGB text becomes uppercase.
- Drawing/entity pattern scales and polyline per-segment/continuous generation.
- Shared/nested local blocks, base points, insertion units and owner-relative
  order; standard-XY insert placement, rotation and signed nonuniform scale.
  CAD setter changes, including the tiny-scale clamp, cause rejection.

Under Allow, incompatible geometry is omitted as a whole entity. Paper layouts with unsupported target coordinate units and their entities are omitted with located evidence. Ordinary local definitions retain supported
content; every instance of a partial definition receives a loss diagnostic,
including through nested blocks. Anonymous/reserved names, XREF/external flags
and directly owned dynamic-block objects cause definition omission; referring
inserts are also omitted. Mappings contain only emitted objects.

Appearance adaptations are explicit and diagnosed: ACI color becomes canonical
RGB; unavailable layer color becomes white; inherited/default layer opacity
becomes opaque; unavailable/default weight becomes 0.25 mm. Numeric weights use
the closest standard CAD weight (ties prefer the lower entry); opacity uses the
closest decoded transparency byte (ties prefer greater transparency). Complex
text/shape patterns become an empty pattern under their original name and ID,
with one Modified diagnostic per definition, including unused definitions. Missing IFCX layer `0` generates an explicit white,
opaque, Continuous, 0.25 mm CAD layer without a source mapping. Entity
ByLayer/ByBlock modes remain independent and stored. These adaptations are
policy-controlled losses, not an appearance resolver.

Oblique frames, arcs/bulges/widths, named/indexed color identity, complex text/shape patterns,
changed CAD settings, XDATA, arrays, attributes and source preservation remain
outside the slice. Unsupported common metadata may be omitted while keeping
geometry. Extra fields on supported types are also checked. See the contracts:
[to CAD](docs/TO-CAD-COVERAGE.md), [from CAD](docs/FROM-CAD-COVERAGE.md).

Unresolved ownership of unsupported non-Layout objects, such as SUN lighting
metadata, and optional reactor/extension relationships receive located loss
diagnostics. Allow omits them; Reject refuses the loss. Entity and Layout
ownership and typed drawing references remain structural requirements. Source
objects are never repaired or removed from the caller's CAD document.

Foreign IFCX information remains in the source reader's graph. A conservative
canonical-envelope comparison reports extra nodes, attributes, relations,
imports and schemas as losses; it can also diagnose equivalent alternate
envelopes and is not graph equivalence. Allow projects the CAD subset; Reject
refuses these differences. Exact geometry value checks separately reject integer
to binary64 rounding. Invalid structure, cycles, dangling essential drawing
references, non-finite known geometry, translated-coordinate rounding and CAD scale clamping fail under
both policies. No geometric tolerance kernel is introduced.

## Exchange evidence

Unmodified opencadcodec revision `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`, AC1032:

| Probe | Result |
| --- | --- |
| Named simple/dot/fractional/empty patterns, scales and polyline generation through DXF and DWG | Semantic comparisons pass; no pattern-name whitelist |
| Primitives through DXF and DWG | Semantic comparisons pass in both directions |
| Nested/shared blocks with nonzero base through DXF | Pass |
| Nested/shared blocks with zero base through DWG | Pass |
| Nonzero block base through DWG | Pass, including nested/shared definitions; codec issue #52 is resolved |
| Near-quarter-turn rotation through DXF | External codec degree/radian roundtrip can change one binary64 step; separately diagnosed in the practice probe |

The [2026-10-05 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-05.md)
records the public-model review and fresh verification. The viewport development
regressions still fail on unmodified upstream; the explicit local patch passes
those tests and remains separate from this default pin. Historical measurements
retain their recorded revisions.

No marker repair or guessed anonymous-block rename is applied. A stale DXF
model cache can be recovered only through unique consistent block/layout
agreement, with `model-cache-recovered` diagnostic. Ambiguity fails.

Run `cargo test -p ifcx-cad-convert`. Fixtures compare geometry, units,
appearance and named owner/definition relations, not regenerated IDs or bytes.
Generated IFCX passes the strict production reader. This bounded corpus does
not establish broad CAD conformance or any file-size/performance conclusion.
The API boundary is CadDocument; diagnostics do not promise exactness through a
subsequent external DWG/DXF writer. Such outputs need independent readback.

### Paper viewport development configuration

Authored Paper viewports now map Model targets, Paper frames, orthographic and
perspective cameras, signed depth clipping, render/display controls and frozen
layers. Active clips support existing Circle and closed straight-polyline
geometry; stored dormant boundaries retain their identity. Forward boundaries
are resolved without changing authored order. Unsupported boundaries omit the
whole viewport with located loss, retaining supported sibling geometry.

CAD exchange currently uses
[`patches/opencadcodec-viewports.toml`](../../patches/opencadcodec-viewports.toml)
and its [documented local checkout/repairs](../../patches/opencadcodec-viewports/README.md).
The default upstream dependency lacks required status support and diagnoses
viewport omissions. Run converter exchange tests with
`cargo test --config patches/opencadcodec-viewports.toml -p ifcx-cad-convert`.
Core native viewports remain independent of opencadcodec. Direction-specific
coverage documents define whole/partial loss, scalar mapping, runtime numbering
and output-handle rules. Full plot configuration and viewport rendering remain
outside this slice; wider OCDraw clipping support remains a separate contract.

The serde feature enables field inspection, not a new file encoding. No Cargo
cache code is edited. Baseline benchmark code is unchanged; no size experiment
is part of this proof. Cargo.lock follows the repository's existing ignore rule.

## OCDraw boundary

This adapter remains separate from `ocdraw-convert`. Its root dependency is the
`ocdraw` crate, whose independent IFCX-CAD module lives in `src/ifcx_cad`.
The IFCX reader uses OCDraw's geometric validation types and unit registry;
IFCX nodes, composition, schema imports and profile validation stay separate
from the standalone OCDraw model and JSON encoding. Conversion is direct to
`CadDocument`, without an intermediate OCDraw file.

Small unit-code and scaffold helpers adapt existing converter reasoning; the
comprehensive numerical kernel is not copied. Extract narrow shared helpers
with parity tests when both adapters need the same contract. Browser support and main integration are implemented as an independent route.

### Line-pattern conversion boundary

Definitions are allocated before layer/entity references and exposed in the
line-pattern mapping. Actual element lengths are copied; the declared CAD
period is derived from them on output. A disagreeing finite source period is a
`line-pattern-period` modification, accepted only under Allow. Alignment other
than A, invalid/nonfinite values, invalid names, missing references and conflicting
name/handle targets fail under both policies. The target CAD uppercase lookup
is checked separately from the native Unicode case-folded name contract.
Canonical ByLayer/ByBlock table records are selection scaffolding, not definitions.
Changed metadata on them and XREF provenance are explicitly diagnosed.
If native Continuous is absent, the fresh CAD target retains its required
Continuous scaffold with `line-pattern-scaffold` modification; Reject refuses it.
Optional native scale/generation defaults are normalized for comparison only;
foreign graph data is still checked and exact numeric projection errors fail.

The OCDraw viewport worktree pins the shared upstream base `fe69506` and opts in
to [explicit local codec repairs](../../patches/opencadcodec-viewports/README.md).
The earlier exchange table retains its stated historical dependency provenance.
The current IFCX-CAD viewport slice uses the same repairs plus the independent
viewport-off repair; its geometry and clipping contract stays separate from OCDraw.
See the [2026-10-05 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-05.md).
## Paper layout conversion boundary

Native Paper names use full Unicode case folding; target CAD lookup additionally
rejects uppercase collisions before allocation. Source tabs must be distinct and
nonnegative, with Model at zero. Gaps normalize to contiguous Paper tabs with a
Recovery diagnostic, preserving relative order. Native IDs and watermarks are
independent of tab order. Only the fully default initial Layout1 scaffold is
excluded; additional empty sheets remain authored.

CAD paper dimensions are always millimetres. Positive finite dimensions map to
an optional medium; invalid or partial dimensions omit only that medium with
loss evidence. Explicit fixed 1:1 inch/mm mappings establish Paper coordinate
units. Fully unconfigured unsized defaults are unitless; other authored mappings
retain numeric coordinates as unitless with a loss diagnostic. Media never
establish coordinate units and block instances are never implicitly rescaled.

Native media convert through exact rational millimetre factors. Any binary64
rounding is fatal under both policies (5 inches maps exactly to 127 mm; 1 inch
cannot exactly map to binary64 25.4 mm). Unsupported factors such as parsecs omit
only media under Allow. Printer/media names, margins, rotation, plot limits,
authored overall canvases and workspace state remain deferred losses. Definition-content
losses propagate to Paper instances through shared and nested definitions.
