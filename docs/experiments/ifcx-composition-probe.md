# IFCX fragment composition interoperability probe

2026-09-30. This probe compares the original [IFCCAD experimental parser](../../src/experimental_ifcx/parse.rs) with the buildingSMART IFCX development composer at commit [`1a63082ada967c683cfacee2005f8f749c8e1b79`](https://github.com/buildingSMART/IFC5-development/commit/1a63082ada967c683cfacee2005f8f749c8e1b79). The table preserves the observed results before the reader adopted later-wins for repeated keys. Both reader revisions use the same provisional 0.1.0 profile identifier; this alpha experiment does not promise compatibility across revisions. The probe tests implementation behavior, not a settled IFCX interchange rule. The current [TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp) defines the node fields but not all of the composition outcomes below.

## Method

The upstream repository was shallow-cloned at the commit above and installed with its locked `npm ci` dependencies. A temporary TypeScript harness imported the unmodified `FlattenCompositionInput`, `Federate`, and `LoadIfcxFile` functions, bundled it with that repository's esbuild, and ran it with Node 24.20.0 on Windows. Each case used two fragments with `path: "node1"`, first and second in one `data` array; `Federate` also received them as two ordered IFCX files. The local results were checked by focused `experimental_ifcx::parse::tests` in this repository. The comparison observes the flattened node before `children`/`inherits` expansion, then checks upstream full loading separately. Ordinary upstream schema validation used String schemas for `test::a` and `test::b`.

| Second fragment relative to first | Local parser at probe time | Upstream flatten / federation | Upstream full load |
| --- | --- | --- | --- |
| New `test::b` beside `test::a` | Merge | Merge | Accepts |
| Same value for `test::a` | Merge | Merge | Accepts |
| `test::a: "A"` then `"B"` | Conflict error | Later `"B"` wins | Accepts `"B"` |
| `children.one: "node2"` then `"node3"` | Conflict error | Later `"node3"` wins | Accepts later child |
| `children.one: "node2"` then `null` | Conflict error | Keeps `null` marker at flatten/federation | Removes child during expansion |
| `test::a: "A"` then `null` | Conflict error | Keeps literal `null` | Without validation: retains `null`; with String schema: error |
| `inherits.one: "node2"` then `null` | Conflict error | Flatten removes entry; federation retains `null` marker | Accepts, no inherited entry |
| Duplicate `test::a` JSON key in one object | JSON parse error | JavaScript `JSON.parse` keeps later key before composer runs | Not separately detected by composer |

These cases deliberately isolate fragment rules; `node2` and `node3` are simple reference names, not a CAD document. The child deletion occurs during upstream expansion, not in its flattened or federated output. The upstream attribute `null` discrepancy is already reported in [buildingSMART issue #132](https://github.com/buildingSMART/IFC5-development/issues/132). Ordinary schema validation prevents the current example of a String attribute deletion from loading, so `null` behavior cannot be judged from a validating load alone.

## Complete CAD fixture check

At probe time, the then-current [hello-cad.ifcx](../../examples/ifcx-native-cad/hello-cad.ifcx) used bracketed paths such as `</cad/d1>` and passed our strict reader. Upstream `LoadIfcxFile` with schema validation disabled returned an artificial root with **zero** children. Its [root finder](https://github.com/buildingSMART/IFC5-development/blob/1a63082ada967c683cfacee2005f8f749c8e1b79/src/ifcx-core/composition/cycles.ts) admitted root paths only when they contained no `/`; those paths therefore never became roots. Replacing every fixture path and matching string reference *in memory* with slashless aliases `n0`, `n1`, etc. yielded one root with five direct children. This isolates a G1 path/composer issue; it does not prove every CAD child or attribute expands correctly. With upstream schema validation enabled, the original fixture failed earlier because its imported `ifccad::drawing` schema was not resolved by `LoadIfcxFile` from the empty local `schemas` map. Our reader resolves that one bundled import offline. That is a separate G5 issue. The current fixture uses unwrapped paths such as `/cad/d1`; this historical probe has not been rerun against that revision.

## Decision for the experiment

The experimental reader now accepts later-wins fragments for all paths by default and validates CAD meaning after composition. Its explicit `RejectConflicts` option provides a development diagnostic using the earlier strict rule. The canonical snapshot writer continues to emit one fragment per path. A later fragment can repair or invalidate geometry, ownership, or draw order; final CAD validation cannot infer whether an overwrite was accidental. Explicit `null` deletion and multi-file layer ordering remain open. If practical CAD cases show that later-wins causes unacceptable ambiguity, use those cases to discuss whether it should become normative IFCX behavior, and reconsider whether a separate CAD format such as OCDraw is preferable. The current upstream composer alone does not establish a production interchange rule.

The path/root mismatch is an obstacle to claiming compatibility with the upstream viewer/composer. The experimental profile now omits USD-style angle brackets in line with the discussion in [issue #63](https://github.com/buildingSMART/IFC5-development/issues/63), while the published TypeSpec still has the earlier pattern. The slash-related root behavior needs a fresh check with the current fixture. Physical-encoding comparisons remain separate from these semantic questions.
