# IFCCAD conversion proof

Experimental direct conversion between `ocdraw::ifccad` and pinned
opencadcodec `CadDocument`. The core remains independent of opencadcodec. No IFCDR
package, block explosion or appearance resolver is used.

The source organization follows `ocdraw-convert`: [`src/from_cad`](src/from_cad)
and [`src/to_cad`](src/to_cad) own conversion orchestration and entity construction;
[`src/mapping`](src/mapping) owns format-specific geometry, appearance, block,
layout, line-pattern and viewport adapters; [`src/source`](src/source) inspects
CAD structure, layout references and supported entity fields. Target layout
allocation lives in `src/to_cad/layouts.rs`. `geometry_context.rs` and
`geometry_assessment.rs` adapt tolerance domains and evidence to IFCCAD ownership.
Numerical geometry and occurrence proofs live in the shared
Numerical geometry lives in cad-geometry-convert; format-owned graph/preservation conditions remain independent. Byte snapshots use cad-preservation and scalar workspace adaptation uses cad-workspace-convert.

This incomplete adapter defaults to **Allow**: supported content is returned
with located diagnostics for omissions and modifications. **Reject** refuses
any diagnosed semantic loss. Errors distinguish invalid structure, unsupported
content, core validation, encoding, output readback, ID allocation and CAD
construction. It makes no losslessness claim for arbitrary CAD files or IFCX graphs.

## API

`IfccadDocument` is the central logical input/output between CAD conversion
and IFCX encoding. `cad_document_to_ifccad_document` constructs it directly
with diagnostics and mappings; `ifccad_document_to_cad_document` validates
and converts the supplied projection. Both require explicit direction-qualified options; use `Default::default()`
for the existing Allow policy.
They do not encode/parse an IFCX file as an intermediate step.

The encoded and source functions below are convenience routes. A loaded
`ValidatedIfccad` retains both the CAD document and its complete immutable
source graph. Its conversion route additionally diagnoses foreign graph content
and source numeric precision. Use this route to assess losses from a full IFCX
input. Projection-only conversion cannot assess discarded source context.

Core `encode_ifccad_document` creates a fresh CAD-profile file. It does not
merge edits into a source graph. Original native downloads use source bytes.
See the [document lifecycle](../../docs/experiments/ifccad-document-lifecycle.md)
for ownership, validation and output boundaries.

```rust
use ocdraw::ifccad::ValidatedIfccad;
use ifccad_convert::{ifccad_source_to_cad_document, cad_document_to_encoded_ifccad,
    IfccadTargetMetadata, IfccadConversionError};

fn convert(source: &ValidatedIfccad, metadata: IfccadTargetMetadata)
    -> Result<Vec<u8>, IfccadConversionError>
{
    let cad = ifccad_source_to_cad_document(source, Default::default())?;
    let back = cad_document_to_encoded_ifccad(cad.document(), metadata, Default::default())?;
    // Strict writer and production reader have validated these bytes.
    Ok(back.encoded().bytes().to_vec())
}
```

Target header and drawing ID are caller supplied. Outcomes expose diagnostics
and mappings; CAD outcomes also offer `into_document()`.

The experimental Rust API retains typed failure payloads:
`CoreValidation(IfccadReport)` replaces `CoreValidation(String)`;
`CoreEncoding(IfccadEncodeError)` distinguishes invalid input, serialization,
production-readback rejection and semantic readback mismatch;
`CoreReadback(IfccadReadError)` covers the converter's additional production load;
and `IdAllocation(IfccadIdAllocationError)` identifies the exhausted domain.
All four expose their underlying error through `std::error::Error::source()`.
The reader error retains its report through both `report()` and `source()`.
Update Rust error matches; native files and conversion policies are unchanged.
Diagnostic strings remain available for presentation.

Use `IfccadToCadOptions { loss_policy: IfccadLossPolicy::Reject, ..Default::default() }` for
source/document-to-CAD conversion and `CadToIfccadOptions` with the same
policy for conversion from CAD. These are distinct direction-specific types. `IfccadDiagnosticAction` distinguishes `Omitted`, `Modified` and
`Recovery` and `RoundedWithinTolerance`; `is_loss()` excludes uniquely established
structural cache repairs while `is_semantic_loss()` also excludes certified rounding.
Diagnostics are part of the result and should be presented to the caller.
The separate layer, layout, line-pattern, block-definition and entity mappings offer
`cad_handle(id)`, `ifccad_id(handle)` and read-only iteration. Layout mappings use
Layout object handles; block mappings use BlockRecord handles. IDs remain u64,
including values above JavaScript's safe integer range. Deterministic allocation
for a fixed enumeration does not promise persistence through CAD serialization.

Each fresh CAD import allocates all five IFCCAD ID domains from 1 with checked
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

Both directions accept `geometry_tolerance` and expose `geometry_assessment()`.
The default is exactly 1e-9 in each domain's own coordinates, independent of physical meaning;
use `IfccadGeometryTolerance::exact()` for zero residual or
`IfccadGeometryTolerance::drawing_units(1e-6)?` for a unitless coordinate budget.
Physical requests require a known drawing unit or fixed physical Paper mapping in
every retained domain, including empty layouts, unless `with_coordinate_fallback(value)`
explicitly supplies a coordinate limit for unknown domains. Model/definitions use drawing units;
Paper uses plot units and ratio. Bounds/media never establish a mapping.
See the [shared geometry and accuracy contract](../../docs/geometry/shared-geometry.md)
for nested occurrence proof, conservative bulge limits and separate file checks.

- One Model layout and multiple Paper layouts with explicit tab order, optional physical media, unused layers and definitions. Model units support all 25 tokens; Paper coordinates have output meaning through plot mapping, with independent medium dimensions.
- Points, XYZ lines, circles/signed arcs, full/partial ellipses, straight/bulged
  planar and straight spatial paths in valid oriented planes. Local-origin
  decomposition may change while the shape meets the configured hard limit.
- True RGB, named signed-length patterns (including unused/empty definitions), supported CAD hundredth-mm line weights and
  exactly byte-representable opacity. ByLayer/ByBlock/Explicit are independent
  for each entity property and remain stored. RGB text becomes uppercase.
- Drawing/entity pattern scales and polyline per-segment/continuous generation.
- Shared/nested local blocks, base points, insertion units and owner-relative
  order; valid oriented insert placement, rotation and signed nonuniform scale.
  CAD setter changes, including the tiny-scale clamp, cause rejection.

Under Allow, incompatible geometry is omitted as a whole entity. Unsupported target medium factors omit medium/dependent plot values with located evidence; Paper geometry and layout ownership remain. Ordinary local definitions retain supported
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

Widths, fitted/mesh curves, named/indexed color identity, complex text/shape patterns,
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
references, non-finite known geometry, geometric exceedance/incomplete proof and
CAD scale clamping fail under both policies. Certified within-limit geometric
rounding is reported and accepted independently of semantic loss policy.

## Exchange evidence

Historical baseline at unmodified opencadcodec revision `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`, AC1032:

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
regressions failed on that unmodified baseline; their clipping and angle repairs
have since merged upstream. The separate viewport-off repair remains local. Historical measurements
retain their recorded revisions.

No marker repair or guessed anonymous-block rename is applied. A stale DXF
model cache can be recovered only through unique consistent block/layout
agreement, with `model-cache-recovered` diagnostic. Ambiguity fails.

Run `cargo test -p ifccad-convert`. Fixtures compare geometry, units,
appearance and named owner/definition relations, not regenerated IDs or bytes.
Generated IFCX passes the strict production reader. This bounded corpus does
not establish broad CAD conformance or any file-size/performance conclusion.
The API boundary is CadDocument; diagnostics do not promise exactness through a
subsequent external DWG/DXF writer. Such outputs need independent readback.

### Paper viewport development configuration

Authored Paper viewports now map Model targets, Paper frames, orthographic and
perspective cameras, signed depth clipping, render/display controls and frozen
layers. Active clips support Circle, full Ellipse and closed straight/bulged planar
geometry; stored dormant boundaries retain their identity. Forward boundaries
are resolved without changing authored order. Unsupported boundaries omit the
whole viewport with located loss, retaining supported sibling geometry.

CAD exchange currently uses
[`patches/opencadcodec-viewports.toml`](../../patches/opencadcodec-viewports.toml)
and its [documented local checkout/repairs](../../patches/opencadcodec-viewports/README.md).
The default upstream dependency lacks required status support and diagnoses
viewport omissions. Run converter exchange tests with
`cargo test --config patches/opencadcodec-viewports.toml -p ifccad-convert`.
Core native viewports remain independent of opencadcodec. Direction-specific
coverage documents define whole/partial loss, scalar mapping, runtime numbering
and output-handle rules. Full plot configuration and viewport rendering remain
outside this slice; each drawing format keeps its independent clip contract.

The serde feature enables field inspection, not a new file encoding. No Cargo
cache code is edited. Baseline benchmark code is unchanged; no size experiment
is part of this proof. Cargo.lock follows the repository's existing ignore rule.

## OCDraw boundary

This adapter remains separate from `ocdraw-convert`. Its root dependency is the
`ocdraw` crate, whose independent IFCCAD module lives in `src/ifccad`.
Both readers use the neutral core geometry validation and unit registry;
IFCX nodes, composition, schema imports and profile validation stay separate
from the standalone OCDraw model and JSON encoding. Conversion is direct to
`CadDocument`, without an intermediate OCDraw file.

Both converters use the `cad-geometry-convert` companion for CAD preparation, tolerance
resolution and paired primitive/nested occurrence proof. Model identity and
source coverage stay in their own adapters. See [shared geometry](../../docs/geometry/shared-geometry.md). Browser support and main integration are implemented as an independent route.

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

Both converters pin upstream `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` and select
only the [remaining viewport-off repair](../../patches/opencadcodec-viewports/README.md).
The earlier exchange table retains its historical provenance. IFCCAD geometry,
clipping and source classification remain separate from OCDraw.
See the [2026-10-07 dependency audit](../../docs/geometry/opencadcodec-update-2026-10-07.md).
## Paper layout conversion boundary

Native Paper names use full Unicode case folding; target CAD lookup additionally
rejects uppercase collisions before allocation. Source tabs must be distinct and
nonnegative, with Model at zero. Gaps normalize to contiguous Paper tabs with a
Recovery diagnostic, preserving relative order. Native IDs and watermarks are
independent of tab order. Only the fully default initial Layout1 scaffold is
excluded; additional empty sheets remain authored.

CAD physical dimensions, margins and offsets are millimetres independently of plot
unit selection. Both layouts and valid media survive when unsupported complete
plot state is omitted with loss. Effective plot settings and saved layout PSLTSCALE
are mapped independently of workspace state. Paper coordinates acquire output
meaning through fixed plot unit/ratio only; no implicit geometry rescaling occurs.

Physical scalar conversions must be exact in binary64; unsupported exact factors
omit medium/dependent plot under Allow, while inexact conversion is a typed failure
under both policies. Raster calibration, printable-relative offsets, transparency
and external style contents keep explicit limits. Definition-content losses still
propagate through shared/nested Paper instances. See the layout-output contract.

## Layout output revision

Both models retain layout media without complete plot settings. Plot unit and
fixed mapping determine Paper output meaning; IFCCAD no longer stores an independent
Paper coordinate unit. Effective plot settings, limits and layout PSLTSCALE have
separate native/CAD coverage. The provisional field/API migration, strict physical
scalar conversion limits, raster restrictions and per-domain accuracy reports are
specified in [layout output](../../docs/layout-output.md). The later workspace slice is documented separately; no renderer, release or controlled measurement is implied.

## Upstream pin update — 2026-10-07

Historical note: this records the previous base. The 2026-10-08 update below supersedes its pin and remaining spline/paperspace limitations.


Both converters use opencadcodec `063c10671fe7833d562f772159771318c7a0ebb9` (0.6.0).
Clipping activation/group 340 and VIEWPORT angle units now come from merged
upstream PRs #88/#89; only the independent viewport-off repair remains selected.
References above to clipping/angle defects of the previous unmodified pin are
historical evidence, not limitations of this new base. The [current dependency
audit](../../docs/geometry/opencadcodec-update-2026-10-07.md) records new public
fields and compatibility decisions. Native spline semantics remain absent;
new OCDraw snapshots use payload v2 and retain v1 read/restore support. Fit-only
spline DXF parameterization remains a target-codec limitation pending PR #99.
Unresolved source layer handles must not silently become a native layer 0.
Known resolved handles are relationship identity and are rebuilt from native
layer references. Canonical one-byte-per-INSERT count framing is derived;
unfamiliar/mismatched count storage retains loss evidence. Additional table,
associative/count and solid-history data remain at the existing unsupported
family boundaries. No benchmark evidence is extended by this update.

Opaque spline capture is available with `CadToIfccadOptions.preservation =
IfccadPreservationCapture::SupportedTyped` (library default Disabled). Export
uses `IfccadPreservationRestore::RestoreSupported` by default, or explicit Skip.
Outcomes expose `preservation_report()` independently from semantic diagnostics
and numeric evidence. `geometry_assessment().is_complete()` is false for opaque
curves/occurrences. See [preservation](../../docs/preservation.md).
## Workspace and upstream update — 2026-10-08

Both converters select opencadcodec 0.6.0 at `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` plus the explicit viewport-off repair [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103). Merged spline DXF parameterization and DWG Paper owner/overall-role repairs now come from upstream. See the [dependency audit](../../docs/geometry/opencadcodec-update-2026-10-08.md) and [workspace contract](../../docs/workspace-state.md) for the current field/transport boundary.

UCS definitions, Model windows, canvas frame/grid/snap/UCS and authored Paper viewport aids map through shared ID-free scalar helpers. Native identities, ownership, source classification and located losses stay format-specific. Disabled zero snap spacing and stored-UCS activation survive independently of current choices. Model-viewport aids retain Model coordinates inside Paper; canvas aids use Paper coordinates. Canvas frames do not enter geometry bounds or clip ownership.

A uniquely available active Model window may be selected; multiple unqualified windows survive with unspecified activation. Paper current viewport/UCS association remains unavailable on the codec surface. Model/Paper mode and the active Paper tab are retained through the unique reserved *Paper_Space block and its consistent LAYOUT association, including multiple sheets. Export synchronizes BLOCK_RECORD names, existing BLOCK begin names and the reserved header handle together; setting the header cache alone does not change the active role. Unknown choices remain omitted with located loss. Skipped viewports receive no workspace references.

Dot grid style and frequencies beyond CAD i16 receive field-specific substitutions. VIEWPORT grid beyond-limits/adaptive/subdivision/follow-workplane flags stay in CadDocument but its pinned DXF/DWG routes do not retain them; target diagnostics identify each nondefault field and Reject refuses that portability loss. Model VPORT grid flags survive both routes. Unrepresented display/icon/base/orthographic/plot/visual state remains diagnosed. Numeric and structural failures remain fatal under both policies.
