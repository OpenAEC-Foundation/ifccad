# IFCX alpha assumptions used by the native CAD experiment

Checked against [buildingSMART's IFCX alpha TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp), examples and linked issues on 2026-09-30. This is a living record of choices the [native CAD profile](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md) relies on but IFCX has not yet specified sufficiently for independent interchange. It is not a list of accepted buildingSMART proposals. Update an entry when the experiment starts relying on a new rule, upstream clarifies it, or an independent implementation disagrees. Keep the exact CAD rule in the profile contract; record the general IFCX question and evidence here.

Each entry records: **current IFCX surface**, **our tested rule**, **open question**, **broader use**, and **evidence / upstream overlap**. `Profile-only` means the rule may never belong in IFCX core. `Candidate` means a general IFCX clarification or mechanism could help other domains. `Interoperability risk` means current implementations may produce different results.

## G1 — Local, compact node paths (`Candidate`)

- **Current IFCX surface:** `IfcxNode.path` is a string constrained by a TypeSpec pattern. The [path issue #63](https://github.com/buildingSMART/IFC5-development/issues/63) notes that this pattern and published UUID examples disagree. The TypeSpec does not define the uniqueness domain or cross-file reference resolution for local paths. The [composition probe](ifcx-composition-probe.md) also found that the current upstream root finder excludes paths containing `/`, so our complete fixture produced no roots until its paths were temporarily replaced with slashless aliases.
- **Our tested rule:** Paths such as `</cad/d1/e42>` use a drawing-local canonical `uint64` segment. The complete path is a node's identity within this experiment, and references use that complete string. A drawing ID distinguishes drawings in the file.
- **Open question:** Can a dataset identifier plus a local path form a stable globally referencable identity? How do imports, merging, renaming, and cross-file references resolve it? Which path characters and normalization rules are normative?
- **Broader use:** Compact, human-readable identifiers for repeated model elements, sensor points, assembly members, and external annotations without a UUID in every node. This needs an explicit collision and reference model before proposing it upstream.
- **Evidence:** [profile paths](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md#file-and-identity), [fixture](../../examples/ifcx-native-cad/hello-cad.ifcx), [composition probe](ifcx-composition-probe.md), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G2 — Ordered relationships through `children` (`Candidate`)

- **Current IFCX surface:** `children` is a keyed record of references or `null`; the TypeSpec does not give its keys a general ordering meaning.
- **Our tested rule:** Layout and block-definition children named `"0"`, `"1"`, etc. are contiguous and define CAD draw order. Exactly one layout or block definition owns each drawable entity. JSON object member order is irrelevant.
- **Open question:** Should IFCX offer an explicit ordered relationship, or should ordering remain a profile convention? If a generic mechanism is added, how are insertion, removal, and concurrent edits handled without renumbering every later relation?
- **Broader use:** Order of assembly steps, presentation items, ordered spatial sections, and other graph traversals where order changes meaning. CAD draw order and single ownership remain profile-specific.
- **Evidence:** [profile ownership and order](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md#scope-and-order), [strict order test](../../tests/experimental_ifcx_native.rs), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G3 — Composition of repeated path fragments (`Interoperability risk`)

- **Current IFCX surface:** `data` is an array of nodes; upstream composition code implements layered merging, while the TypeSpec alone does not define collision, deletion, or duplicate-key behavior. [Issue #132](https://github.com/buildingSMART/IFC5-development/issues/132) reports differing `null` deletion behavior for children/inherits and attributes in the current implementation.
- **Our tested rule:** Repeated fragments with the same path merge only disjoint keys or identical values. Conflicting values and duplicate JSON object keys fail, including on non-CAD paths because parsing precedes CAD-role validation. This experiment does not implement layered overwrites or `null` deletion. The [composition probe](ifcx-composition-probe.md) confirms that upstream instead lets a later conflicting value win.
- **Open question:** What is the normative result of repeated paths, conflicting values, duplicate keys, and explicit `null` across one file or imported layers? Is source order meaningful?
- **Broader use:** Predictable federation, extensions, diffs, and collaboration for all IFCX nodes. Our conservative reader may reject a valid future IFCX layer, so it must be tested against the upstream composer before an interoperability claim.
- **Evidence:** [profile fragment rule](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md#file-and-identity), [composition probe](ifcx-composition-probe.md), [upstream composer](https://github.com/buildingSMART/IFC5-development/blob/main/src/ifcx-core/composition/compose.ts), [upstream issue #132](https://github.com/buildingSMART/IFC5-development/issues/132).

## G4 — Node profiles and cross-attribute requirements (`Candidate`)

- **Current IFCX surface:** `schemas` describes individual attribute values. The alpha TypeSpec does not provide a normative way to say that a node with one role must also have specified attributes, exactly one of several payloads, and valid references. [Issue #72](https://github.com/buildingSMART/IFC5-development/issues/72) discusses how class properties could be described, but does not establish this validation contract.
- **Our tested rule:** An owned drawable must have `ifccad::entity` and exactly one supported geometry payload. A circle also needs placement. References, ownership, draw order and appearance modes are checked by the strict profile reader.
- **Open question:** Is there a reusable IFCX profile or node-type mechanism for required attribute sets, alternatives, reference targets and cardinalities? How does it coexist with unknown extension attributes?
- **Broader use:** Independent validation of domain-specific nodes, including building elements, infrastructure and linked observations. The CAD primitive set and its exact geometry rules remain profile-specific.
- **Evidence:** [profile validation](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md), [strict reader and tests](../../tests/experimental_ifcx_native.rs), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G5 — Versioned schema imports and reproducible resolution (`Candidate`)

- **Current IFCX surface:** `imports` contains a URI and optional `integrity`, but the TypeSpec does not define a complete resolver, version policy, or offline behavior.
- **Our tested rule:** The drawing imports `urn:example:ifccad:0.1.0`; the reader resolves only this bundled immutable schema module offline. Missing definitions fail. The `example` namespace is temporary.
- **Open question:** How are imports located, verified, cached and kept stable across versions? Can a file carry both an immutable schema identity and a resolvable location without assuming network access?
- **Broader use:** Reproducible exchange and validation of any domain extension, especially archived models and disconnected workflows.
- **Evidence:** [profile import rule](../../schemas/ifcx-native-cad/experimental-contract-0.1.0.md#file-and-identity), [schema module](../../schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## Profile choices to keep separate

`ifccad::geom::*`, layer 0, ByLayer/ByBlock/explicit appearance, block transforms, drawing units, and the right-handed local XYZ convention are currently CAD-profile semantics. They demonstrate the need for reusable geometry and unit vocabulary, but do not by themselves justify changing IFCX core. Compare their meaning with published IFCX geometry definitions when available. The [IFCCAD benchmark issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6) overlaps future physical-encoding comparisons; it does not settle this node-model experiment. CBOR is only a proposed probe, not a feature used by the current reader or writer.

## Suggested order of investigation

1. Follow up the [completed G1/G3 composition probe](ifcx-composition-probe.md) by checking whether any path convention satisfies both the current TypeSpec and composer, then perform a full upstream expansion check for G2. Record actual differences before drafting an upstream proposal.
2. Extend the fixture with nested blocks using layer 0 and ByBlock, then a paper layout, to test whether G2 and G4 still work under realistic reuse and multiple scopes. Compare semantic readback, not just JSON shape.
3. Test one non-CAD reference to a local CAD path across a second IFCX document. This turns the broader G1 motivation into evidence and exposes import/identity requirements in G5.
4. Only after the semantic model survives these cases, compare matched JSON, compressed JSON, and an implemented binary candidate. Keep the physical-encoding work aligned with [IFCCAD issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6).

Before contacting buildingSMART, split each candidate into a minimal domain-neutral question, a reproducible IFCX example, observed upstream behavior, and an IFCCAD use case. Link any resulting upstream issue or decision back to its G-number.
