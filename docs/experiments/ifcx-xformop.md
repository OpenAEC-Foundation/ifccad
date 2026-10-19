# IFCCAD xformop placement reader experiment

This experiment reads ordinary IFCCAD geometry and drawing structure with
`usd::xformop` replacing native geometry placement. It tests the intent to reuse
published IFCX schemas against actual native readback and CAD exchange.
**Our own placement remains the default.** Matrix input requires an explicitly
selected reader; neither the default reader nor the logical drawing model changes.

## Active workspace basis

The experiment worktree was updated on 2026-10-19 to main commit `40b63d2`.
The core package is `ifccad`, and public model/IO types are exported directly:
`use ifccad::{load_ifccad_xformop_bytes, IfccadReadOptions};`.
The reader, example and CAD exchange tests use only the active IFCCAD workspace;
they do not import parked OCDraw APIs, packages, helpers or fixtures.
All three drawing examples include the current required symbol-source ID counter
and represent their fixed white layer color as RGB components.
The selected CAD codec is the current main pin with its documented
[Dimension repair](../../patches/opencadcodec-dimension/README.md).

The reader experiment applies to IFCX JSON. CBOR revision 2 and optional native
Zstd transport retain their existing native-document semantics; no matrix
placement decoder was added to those routes. The nested block example remains
a schema-only proposal, outside the experimental reader's supported mapping.

## Example and implemented route

The [matrix example](../../examples/experiments/ifcx-xformop/standard-xformop.ifcx)
is the same drawing as the [native reference](../../examples/experiments/ifcx-xformop/native-placement.ifcx).
It keeps our planar-polyline and line geometry, layers, line patterns, layouts,
blocks, IDs, allocation state and drawing order. Only the two planar-polyline
placements are replaced with `usd::xformop`. There is no USD mesh or curve
geometry in this example. Its native block transforms are unchanged.

The public `load_ifccad_xformop_bytes(bytes, IfccadReadOptions)` function composes
the IFCX fragments, maps supported matrix placements to native frames, and runs
the ordinary projection and shared validation. It returns a `ValidatedIfccad`
whose `document()` is an ordinary `IfccadDocument`. The original matrix-based
source bytes and composed graph remain available through `graph()`.

This is a **codec experiment**, not a matrix variant in `IfccadDocument`.
`encode_ifccad_document()` writes the projected native placement and its output
passes the default reader. It does not re-emit xformop or merge document edits
back into the original source graph. The original source snapshot can be retained
independently. No second complete placement is stored in the matrix input file.

Point, Circle, Arc, Ellipse, EllipseArc and PlanarPolyline geometry placements
use the experimental mapping. Matrices on block instances, direct XYZ geometry,
layouts, block definitions or other CAD roles are rejected rather than ignored.
Foreign nodes, including unowned geometry attributes, stay outside this mapping.
Block rotation, scale and base point therefore keep their existing native rules;
flattening them into a matrix is not implemented by this slice.

## Nested block placement schema example

The separate [nested block example](../../examples/experiments/ifcx-xformop/nested-block-xformop.ifcx)
demonstrates reuse inside a custom attribute value. Its inline
`ifccad::experiment::blockInstanceXformop` declaration defines
`transform.placement` as:

```json
"placement": {
  "dataType": "Object",
  "inherits": ["usd::xformop"]
}
```

The three experimental block instances store their matrices at
`ifccad::experiment::blockInstanceXformop.transform.placement.transform`, retaining separate native
rotation, scale, definition references and definition base points. There is no
top-level xformop attribute on those block instances. Geometry and the original
block hierarchy stay IFCCAD. In this example `inherits` is schema-value reuse;
it is not node inheritance or a chain of geometric placements.

The example uses a unique experimental attribute identifier. It imports the
native schema unchanged and does not redefine `ifccad::blockInstance` or any
USD identifier. Schema inheritance of a field is independent of schema override:
the inherited USD value definition stays unchanged.

The example is a proposal, separate from the supported reader fixture. On
2026-10-08 the upstream schema-validation functions at commit
`1a63082ada967c683cfacee2005f8f749c8e1b79` (blob
`a430a1a82b3b64566db5a78025e1f25591e5dd28`) accepted its attribute values against
the native/USD definitions and the explicit local alternative declaration.
The schema identifiers were also checked for collisions with both imported
module definitions. The same functions rejected both a native O/X/Y value in the inherited field
and a nonnumeric matrix component. The isolated check used Node 24.20.0 with
TypeScript type stripping and removed the two type-only imports; validation
function bodies were unchanged. This checks value inheritance, not complete
IFCX import resolution, graph semantics or CAD conversion.

On 2026-10-19 the same pinned functions were rerun for the two top-level xformop
values and three experimental block attributes, with the current native schema
module loaded unchanged. All five values passed, both negative controls failed
as expected, and the local schema identifier still had no imported collision.
This rerun deliberately checks only these attributes: the old upstream validator
cannot process unrestricted arrays used by current native line-pattern schemas.
It therefore cannot certify the complete current drawing. Production reader and
CAD tests cover the supported native/matrix pair independently.

Our reader does not recognize the experimental block attribute or decode its
nested matrices. Its extension must explicitly recognize the new schema and map
the nested value. The inner instance also retains the `(0.6,0.8)` frame discussed
below, so the exact-Z mapping boundary needs a separate decision.
[Upstream pull request 58](https://github.com/buildingSMART/IFC5-development/pull/58)
was an open, unmerged draft about disallowing imported-schema overrides when
reviewed on 2026-10-08. The
first illustration reused the native identifier and an isolated validation map
selected that replacement explicitly; this proved field inheritance, not the
legality of duplicate declarations. The example now avoids that ambiguity.
A future supported profile still needs an explicit module/version decision.
The native default remains unchanged.

## Shared schema and mapping contract

The matrix example imports `https://ifcx.dev/@openusd.org/usd@v1.ifcx` alongside
the native CAD module. The reviewed [USD module](https://github.com/buildingSMART/ifcx.dev/blob/36010c0e1069043f964a7dff8367e3e910255ebb/%40openusd.org/usd%40v1.ifcx)
is recorded in [cases.json](../../examples/experiments/ifcx-xformop/cases.json):
repository commit `36010c0e1069043f964a7dff8367e3e910255ebb`, Git blob
`7a3301c9dce41e9cc87993a3fef1040c00015915`. Its mutable import URI is not an
immutable upstream version guarantee.

The experimental codec requires that import for mapped CAD nodes and checks the
reviewed xformop value shape locally. A conflicting inline declaration fails.
It does not fetch remote modules or provide general USD/IFCX schema validation.
The active native schema module and profile remain unchanged; this reader is
an explicit provisional alternate mapping, not ordinary profile compatibility.

The value contains exactly `transform`, a finite 4 by 4 affine matrix. Following
[OpenUSD's row-vector convention](https://openusd.org/release/api/class_gf_matrix4d.html),
X/Y/Z occupy the first three rows, O the final row, and the final column is
`[0,0,0,1]`. A local point transforms as `[x,y,z,1] * M`.

Mapping copies O, X and Y without normalization and validates them using the
existing native frame predicate. Z must equal the exact rational cross product
of the stored binary64 X/Y values. This deliberately strict boundary prevents
silently dropping an independently authored Z row. For example, X `(0.6,0.8,0)`,
Y `(-0.8,0.6,0)` and Z `(0,0,1)` pass an ordinary rounded cross-product check,
but do not satisfy exact equality for the stored scalars. This experiment rejects
that matrix even when its planar point geometry could be represented. A wider
mapping needs explicit reparameterization/accuracy rules or a matrix-valued
logical model; no such approximation is selected here.

Matrices outside the shared native frame constraints, perspective/non-affine
matrices, malformed values and simultaneous native-plus-matrix placements fail.
Ownership, bounds, primitive validation, reference integrity and other CAD rules
remain with shared native validation. Fragment composition occurs before mapping;
repeated xformop attributes follow the selected composition policy, not matrix
multiplication. This example keeps native CAD paths and makes no claim that
upstream viewers understand the native geometry or root/ownership conventions.

## Verified evidence

| Check | Result and scope |
| --- | --- |
| Experimental example versus native reference | Same complete `IfccadDocument`, including geometry, layers, blocks, order and IDs |
| Original source context | Matrix attributes and original bytes remain in the immutable graph |
| Native encoding/readback | Projected document passes ordinary encoding and strict production readback |
| Placed families | Point, Circle, Arc, Ellipse, EllipseArc and PlanarPolyline exercised through shared validation |
| Rejection boundaries | Matrix shape, affine column, invalid frames/Z, missing import, schema override, double placement and unsupported CAD roles rejected |
| Composition and foreign nodes | Later fragments map after composition; unrelated/unowned source nodes retained |
| DXF and DWG exchange | The example's projected document has the same semantic file readback as the native-reference route, through production codecs and strict native validation |

These checks are recorded in [reader tests](../../tests/ifccad_xformop.rs) and
[exchange tests](../../crates/ifccad-convert/tests/xformop_exchange.rs). The file
exchange uses existing hard tolerance and semantic Reject policy; it does not
claim bit-identical original CAD parameters or qualification of arbitrary matrices.
The example's block transforms are native, so their successful exchange does not
qualify matrix-based block decomposition. The [manual arithmetic check](../../examples/experiments/ifcx-xformop/check.py)
compares the two placed paths and unchanged block occurrences independently.

Historical verification on 2026-10-07 passed focused reader/exchange tests, workspace doc
tests, the runnable reader example, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
That evidence predates the active-crate migration and does not certify the
2026-10-19 candidate. Fresh verification on the updated `40b63d2` basis passed
the active-workspace boundary audit, formatting, workspace Clippy with warnings
denied, and the full workspace suite: 952 tests passed, with one manual performance
measurement intentionally ignored. The reader's eight tests, both CAD exchange
tests and the runnable example also passed focused checks. Input identities,
dependency/configuration provenance and logs are retained in the worktree's ignored
`.superpowers/verification/xformop-main-20261019/` directory.

For conversion, pass the explicit projection to
`ifccad_document_to_cad_document(source.document(), options)`. The ordinary
`ifccad_source_to_cad_document()` additionally compares the original graph with a
canonical native graph; it still diagnoses xformop/import information as outside
that ordinary source contract, and Reject refuses it. This distinction prevents
silently claiming preservation of the complete matrix source during CAD export.

## Remaining conversion questions

- A matrix does not uniquely retain the original CAD frame, rotation, signed
  scale or possibly nonunit normal. Explicit Z alone is not exact source-normal
  preservation. Adding a rotation field alone cannot recover the original frame
  from an already rounded effective matrix.
- Combining a block's base point and placement can lose arithmetic information.
  With base `2^54`, origin `1`, scale `1`, and a point at the base, separate
  operations yield `1`, while the combined binary64 translation yields `0`.
  The manual check retains this witness; this reader does not merge blocks.
- Nested nonuniform scales and rotations can produce effective shear. A future
  matrix/block mapping must distinguish hierarchy from flattening and avoid
  guessing original insert parameters.
- General native matrix authorship would require a logical matrix variant,
  validation/evaluation rules and converter coverage. A codec-only exact subset
  cannot store every otherwise valid affine matrix in the existing document.

This overlaps [placement research issue 11](https://github.com/OpenAEC-Foundation/ifccad/issues/11)
in separating stored parameters from evaluated geometry. It does not select an
arbitrary-axis representation or replace that investigation. Future standard
schema components need their own mappings, validation and exchange evidence.

## Reproduction

From the repository root:

```text
cargo run --example read_ifccad_xformop -- examples/experiments/ifcx-xformop/standard-xformop.ifcx
cargo test -p ifccad --test ifccad_xformop
cargo test -p ifccad-convert --test xformop_exchange
python examples/experiments/ifcx-xformop/check.py
```

The reader example optionally accepts a second path and writes a new native
IFCCAD file without overwriting an existing file. The normal viewer/browser
opening route remains native-only and rejects the matrix example's missing
native placements. No upstream viewer/composer qualification or size/exchange
measurement is claimed. The [roadmap](../../ROADMAP.md) continues to govern
feature sequencing; only a later explicit decision can change the default.
