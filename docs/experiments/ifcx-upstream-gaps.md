# IFCX alpha assumptions used by the native CAD experiment

Checked against [buildingSMART's IFCX alpha TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp), examples and linked issues on 2026-09-30. This is a living record of choices the [native CAD profile](../../schemas/ifccad/experimental-contract-0.1.0.md) relies on but IFCX has not yet specified sufficiently for independent interchange. It is not a list of accepted buildingSMART proposals. Update an entry when the experiment starts relying on a new rule, upstream clarifies it, or an independent implementation disagrees. Keep the exact CAD rule in the profile contract; record the general IFCX question and evidence here.

Each entry records: **current IFCX surface**, **our tested rule**, **open question**, **broader use**, and **evidence / upstream overlap**. `Profile-only` means the rule may never belong in IFCX core. `Candidate` means a general IFCX clarification or mechanism could help other domains. `Interoperability risk` means current implementations may produce different results.

## G1 — Local, compact node paths (`Candidate`)

- **Current IFCX surface:** `IfcxNode.path` is a string constrained by a TypeSpec pattern. The [path issue #63](https://github.com/buildingSMART/IFC5-development/issues/63) notes that this pattern and published UUID examples disagree; its maintainer discussion calls for removing the earlier USD-style angle brackets and allowing `/` and `-`. The current TypeSpec pattern has not yet caught up. It also does not define the uniqueness domain or cross-file reference resolution for local paths. The [composition probe](ifcx-composition-probe.md) found that the upstream root finder excluded paths containing `/`, so the then-current complete fixture produced no roots until its paths were temporarily replaced with slashless aliases.
- **Our tested rule:** Paths such as `/cad/d1/e42` use a drawing-local canonical `uint64` segment. The complete path is a node's identity within this experiment, and references use that complete string. A drawing ID distinguishes drawings in the file. The reader rejects the earlier bracketed CAD paths.
- **Open question:** Can a dataset identifier plus a local path form a stable globally referencable identity? How do imports, merging, renaming, and cross-file references resolve it? Which path characters and normalization rules are normative?
- **Broader use:** Compact, human-readable identifiers for repeated model elements, sensor points, assembly members, and external annotations without a UUID in every node. This needs an explicit collision and reference model before proposing it upstream.
- **Evidence:** [profile paths](../../schemas/ifccad/experimental-contract-0.1.0.md#file-and-identity), [fixture](../../examples/ifccad/hello-cad.ifcx), [composition probe](ifcx-composition-probe.md), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

### Native ID lifecycle (2026-10-02)

The [persistent allocation design](ifccad-id-management.md) keeps existing
paths stable during native edits and records per-domain watermarks on the
drawing. This is implemented and verified locally on `ifccad-id-management`.
The counters are a CAD-profile choice;
they do not establish global uniqueness, resolve cross-file identities or
support independently allocating editors. A future collaboration design must
address node identity and ordered relationships together. G1 remains open.

The schema describes these counters as IFCX `Integer`; exact unsigned 64-bit
range and allocation rules are profile constraints. Rust/native WASM readback
and original-byte browser download preserve full-width values. Full-width
Integer interchange through the upstream composer or other JavaScript
consumers still needs an independent probe; no generic IFCX precision guarantee
is claimed. The broader question also applies to exact counters and identifiers
outside CAD, independently of compact paths.

## G2 — Ordered relationships through `children` (`Candidate`)

- **Current IFCX surface:** `children` is a keyed record of references or `null`; the TypeSpec does not give its keys a general ordering meaning.
- **Our tested rule:** Model-layout, Paper-layout and block-definition children named `"0"`, `"1"`, etc. are contiguous and define CAD draw order separately per scope. Exactly one layout or block definition owns each drawable entity. JSON object member order is irrelevant; drawing children do not define layout-tab order.
- **Open question:** Should IFCX offer an explicit ordered relationship, or should ordering remain a profile convention? If a generic mechanism is added, how are insertion, removal, and concurrent edits handled without renumbering every later relation?
- **Broader use:** Order of assembly steps, presentation items, ordered spatial sections, and other graph traversals where order changes meaning. CAD draw order and single ownership remain profile-specific.
- **Evidence:** [profile ownership and order](../../schemas/ifccad/experimental-contract-0.1.0.md#scope-and-order), [nested-block fixture](../../examples/ifccad/hello-nested-blocks.ifcx), [paper-layout fixture](../../examples/ifccad/hello-paper-layouts.ifcx), [strict order tests](../../tests/ifccad_native.rs), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G3 — Composition of repeated path fragments (`Interoperability risk`)

- **Current IFCX surface:** `data` is an array of nodes; upstream composition code implements layered merging, while the TypeSpec alone does not define collision, deletion, or duplicate-key behavior. [Issue #132](https://github.com/buildingSMART/IFC5-development/issues/132) reports differing `null` deletion behavior for children/inherits and attributes in the current implementation.
- **Our tested rule:** By default, repeated fragments with the same path merge in file order, with the later value winning per `children`, `inherits`, or `attributes` key on CAD and non-CAD nodes alike. An explicit reader option rejects differing repeated values for development diagnostics. CAD constraints are checked on the composed result in both modes. Duplicate JSON object keys in one object still fail. In the default mode, the parser removes an `inherits` key when overwritten by `null` and retains `null` markers for children and attributes, following upstream flattening; graph expansion and attribute deletion remain unresolved. The [composition probe](ifcx-composition-probe.md) measured the upstream behavior that motivated this change.
- **Open question:** What is the normative result of repeated paths, conflicting values, duplicate keys, and explicit `null` across one file or imported layers? Is source order meaningful?
- **Broader use:** Predictable federation, extensions, diffs, and collaboration for all IFCX nodes. File-order overwrites are useful for layered changes, while CAD validation can still reject an invalid final drawing. We need concrete CAD cases before arguing that last-wins causes unacceptable ambiguity or should be non-normative in IFCX. A domain-specific format such as OCDraw remains an alternative if the IFCX graph or composition rules prove unsuitable.
- **Evidence:** [profile fragment rule](../../schemas/ifccad/experimental-contract-0.1.0.md#file-and-identity), [composition probe](ifcx-composition-probe.md), [upstream composer](https://github.com/buildingSMART/IFC5-development/blob/main/src/ifcx-core/composition/compose.ts), [upstream issue #132](https://github.com/buildingSMART/IFC5-development/issues/132).

## G4 — Node profiles and cross-attribute requirements (`Candidate`)

- **Current IFCX surface:** `schemas` describes individual attribute values. The alpha TypeSpec does not provide a normative way to say that a node with one role must also have specified attributes, exactly one of several payloads, and valid references. [Issue #72](https://github.com/buildingSMART/IFC5-development/issues/72) discusses how class properties could be described, but does not establish this validation contract.
- **Our tested rule:** An owned drawable must have `ifccad::entity` and exactly one supported geometry payload. A circle also needs placement. A Paper layout requires a unique name, explicit tab index and coordinate unit. Its optional physical medium has positive dimensions and a physical unit; Model requires tab zero and forbids Paper metadata. Exactly one Model layout is required; Paper layouts are optional. These conditional value and graph requirements, references, ownership, draw order and appearance modes are checked by the strict profile reader.
- **Open question:** Is there a reusable IFCX profile or node-type mechanism for required attribute sets, alternatives, reference targets and cardinalities? How does it coexist with unknown extension attributes?
- **Broader use:** Independent validation of domain-specific nodes, including building elements, infrastructure and linked observations. The CAD primitive set and its exact geometry rules remain profile-specific.
- **Evidence:** [profile validation](../../schemas/ifccad/experimental-contract-0.1.0.md), [strict reader and tests](../../tests/ifccad_native.rs), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G5 — Versioned schema imports and reproducible resolution (`Candidate`)

- **Current IFCX surface:** `imports` contains a URI and optional `integrity`, but the TypeSpec does not define a complete resolver, version policy, or offline behavior.
- **Our tested rule:** The drawing imports `urn:example:ifccad:0.1.0`; the reader resolves only its bundled experimental schema module offline. Missing definitions fail. The `example` namespace and version are provisional: schema meaning currently follows the reader revision, without an immutable-publication promise.
- **Open question:** How are imports located, verified, cached and kept stable across versions? Can a file carry both an immutable schema identity and a resolvable location without assuming network access?
- **Broader use:** Reproducible exchange and validation of any domain extension, especially archived models and disconnected workflows.
- **Evidence:** [profile import rule](../../schemas/ifccad/experimental-contract-0.1.0.md#file-and-identity), [schema module](../../schemas/ifccad/ifccad-profile-0.1.0.ifcx), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp).

## G6 — Reusable value schemas and unions (`Candidate`)

- **Current IFCX surface:** The alpha TypeSpec describes nested arrays and objects inline. It has no defined union of alternative value schemas or clear named-schema reference for a nested value. Its `IfcxValueDescription` also lists restriction fields independently of `dataType`. [Schema wishlist issue #51](https://github.com/buildingSMART/IFC5-development/issues/51) proposes datatype-specific restrictions, schema references and unions; these are discussion points, not adopted rules. Value-schema inheritance here is separate from `IfcxNode.inherits`.
- **Our tested rule:** The experimental schema repeats three-real arrays for line endpoints and placement vectors, while planar-polyline vertices use two-real arrays. The profile reader checks the supported value shapes and CAD semantics without inventing IFCX syntax for schema references or unions.
- **Open question:** Can a value schema refer to another named value schema, and can an array element be one of several named shapes? How are restrictions tied to their declared datatype, including validation of mismatched or unused restriction fields?
- **Broader use:** One reusable point or placement definition and a typed union of curve segments would improve independent validation of procedural geometry. The same mechanisms would serve non-CAD structured values. They would not by themselves require `ifccad::entity` and geometry attributes on the same node; that remains G4.
- **Evidence:** [upstream issue #51](https://github.com/buildingSMART/IFC5-development/issues/51), [upstream TypeSpec](https://github.com/buildingSMART/IFC5-development/blob/main/schema/ifcx.tsp), [experimental schema module](../../schemas/ifccad/ifccad-profile-0.1.0.ifcx).

## Profile choices to keep separate

`ifccad::geom::*`, layer 0, ByLayer/ByBlock/explicit appearance, block transforms, drawing and paper units, Paper layout metadata, and the right-handed local XYZ convention are currently CAD-profile semantics. Shared block coordinates use drawing units, while direct paper coordinates use the paper unit; instance scale explicitly includes any conversion. These choices demonstrate the need for reusable geometry and unit vocabulary, but do not by themselves justify changing IFCX core. Compare their meaning with published IFCX geometry definitions when available. The [IFCCAD benchmark issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6) overlaps future physical-encoding comparisons; it does not settle this node-model experiment. CBOR is only a proposed probe, not a feature used by the current reader or writer.

## Suggested order of investigation

1. Follow up the [completed G1/G3 composition probe](ifcx-composition-probe.md) with a full upstream expansion check for G2 using the current unwrapped CAD paths. The TypeSpec still requires old-style brackets while the issue discussion recommends removing them; record actual implementation differences before drafting an upstream proposal.
2. Nested blocks and two Paper layouts now have semantic readback evidence for G2/G4, including distinct units, shared definitions, draw order and rejected duplicate ownership. Extend this to a paper viewport referencing model space before claiming complete layout exchange; define its view, clipping and scale semantics explicitly.
3. Test one non-CAD reference to a local CAD path across a second IFCX document. This turns the broader G1 motivation into evidence and exposes import/identity requirements in G5.
4. Only after the semantic model survives these cases, compare matched JSON, compressed JSON, and an implemented binary candidate. Keep the physical-encoding work aligned with [IFCCAD issue #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6).

Before contacting buildingSMART, split each candidate into a minimal domain-neutral question, a reproducible IFCX example, observed upstream behavior, and an IFCCAD use case. Link any resulting upstream issue or decision back to its G-number.

## Line-pattern evidence (2026-10-01)

- G1: drawing-local `/cad/dN/linePattern/N` string references resolve to named
  definitions, including unused definitions, without UUIDs or duplicated IDs.
  Target role and drawing scope remain profile checks, not IFCX guarantees.
- G3: a later `ifccad::linePattern` attribute replaces the entire earlier object
  and ordered element array. Tested composition does not append array elements.
- G4: the schema describes an object/real array; the reader separately enforces
  target roles, drawing membership, folded name uniqueness, signed-length rules,
  positive scale/defaults and mode-dependent references. No unadopted reference
  datatype or node-profile syntax is introduced.

Evidence: [line-pattern tests](../../tests/ifccad_line_patterns.rs),
[conversion tests](../../crates/ifccad-convert/tests/line_patterns.rs) and
[profile contract](../../schemas/ifccad/experimental-contract-0.1.0.md#named-line-patterns).
