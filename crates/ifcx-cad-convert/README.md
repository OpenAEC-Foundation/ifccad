# IFCX-CAD conversion proof

Experimental direct conversion between `ocdraw::ifcx_cad` and pinned
cadcodec `CadDocument`. The core remains independent of cadcodec. No IFCDR
package, block explosion or appearance resolver is used.

This strict, incomplete adapter returns located diagnostics instead of successful
partial output. Errors distinguish invalid structure, unsupported semantic
content, core validation and CAD construction. It makes no losslessness claim
for arbitrary CAD files or IFCX graphs.

## API

```rust
use ocdraw::ifcx_cad::ValidatedIfcxCad;
use ifcx_cad_convert::{ifcx_cad_to_cad_document, cad_document_to_ifcx_cad,
    IfcxCadTargetMetadata, IfcxCadConversionError};

fn convert(source: &ValidatedIfcxCad, metadata: IfcxCadTargetMetadata)
    -> Result<Vec<u8>, IfcxCadConversionError>
{
    let cad = ifcx_cad_to_cad_document(source)?;
    let back = cad_document_to_ifcx_cad(cad.document(), metadata)?;
    // Strict writer and production reader have validated these bytes.
    Ok(back.ifcx_bytes().to_vec())
}
```

Target header and drawing ID are caller supplied. Outcomes expose diagnostics
and mappings; CAD outcomes also offer `into_document()`.
The separate layer, layout, block-definition and entity mappings offer
`cad_handle(id)`, `ifcx_id(handle)` and read-only iteration. Layout mappings use
Layout object handles; block mappings use BlockRecord handles. IDs remain u64,
including values above JavaScript's safe integer range. Deterministic allocation
for a fixed enumeration does not promise persistence through CAD serialization.

## Supported slice

- One Model layout, 25 length-unit tokens, unused layers and definitions.
- XYZ lines, standard-XY circles and straight planar polylines. Polyline XY
  translation folds into vertices only for exact finite binary64 sums; Z becomes
  elevation. Position/path/plane survive, local-origin decomposition does not.
- True RGB, Continuous pattern, supported CAD hundredth-mm line weights and
  exactly byte-representable opacity. ByLayer/ByBlock/Explicit are independent
  for each entity property and remain stored. RGB text becomes uppercase.
- Shared/nested local blocks, base points, insertion units and owner-relative
  order; standard-XY insert placement, rotation and signed nonuniform scale.
  CAD setter changes, including the tiny-scale clamp, cause rejection.

This slice requires explicit IFCX layer `0`. CAD layer appearance must be
concrete: indexed color and Default weight are rejected rather than silently
resolved. Paper/viewports, oblique frames, arcs/bulges/widths, named/indexed
color identity, custom patterns, changed CAD settings, XDATA, arrays, attributes,
anonymous/dynamic blocks and XREFs remain outside the slice. Extra fields on
supported types are also checked. See the direction contracts:
[to CAD](docs/TO-CAD-COVERAGE.md), [from CAD](docs/FROM-CAD-COVERAGE.md).

Foreign IFCX information remains in the source reader's graph. The adapter
rejects differences from the canonical profile envelope, including extra nodes,
attributes, relations, imports and schemas. This conservative comparison can
also reject equivalent alternate envelopes; it is not graph equivalence.

## Exchange evidence

Unmodified cadcodec revision `5b682ed66ea2c89be8142c8dd83d83774fc3de08`, AC1032:

| Probe | Result |
| --- | --- |
| Primitives through DXF and DWG | Semantic comparisons pass in both directions |
| Nested/shared blocks with nonzero base through DXF | Pass |
| Nested/shared blocks with zero base through DWG | Pass |
| Nonzero block base through DWG | Expected rejection: decoded BLOCK marker disagrees with BlockRecord (known codec issue #52) |

No marker repair or guessed anonymous-block rename is applied. A stale DXF
model cache can be recovered only through unique consistent block/layout
agreement, with `model-cache-recovered` diagnostic. Ambiguity fails.

Run `cargo test -p ifcx-cad-convert`. Fixtures compare geometry, units,
appearance and named owner/definition relations, not regenerated IDs or bytes.
Generated IFCX passes the strict production reader. This bounded corpus does
not establish broad CAD conformance or any file-size/performance conclusion.

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
with parity tests when both adapters need the same contract. Browser support
and main integration remain separate follow-ups.
