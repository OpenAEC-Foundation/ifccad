# Drawing-resource terminology and representation consistency

Date: 2026-09-08

Status: Approved, implemented and verified. The implementation includes the
0.8.0 overlay and normative relationship contract, reader/writer vocabulary,
cross-layout validation and renewed candidate conformance cases.

Branch: `drawing-resource-terminology`, based on `main` at `83fcdb8`.

## Purpose and scope

Milestone 2 requires IFCX and Rust terminology to describe an IFCDR resource as
complete drawing content. A resource can contain model and paper scopes; its
name and descriptor must not suggest that it contains only model-space geometry.

The encoding-neutral resource model is already implemented. This step changes
the IFCX vocabulary, package relationship validation, public representation
type, default writer filename, and corresponding contract evidence. It does
not change IFCDR geometry, encoding, resource construction, or supported CAD
entity families.

The scope follows open issue #8. It builds on #7's codec boundary, leaves #1's
inline resource implementation for the next step, and does not implement #3's
physical chunking or #6's size measurements. `ROADMAP.md` remains authoritative
for milestone status and sequencing.

## Approved terminology

| Existing name | New name |
| --- | --- |
| `openaec:DrawingGeometryRepresentation` | `openaec:DrawingRepresentation` |
| `attributes.geometry` on that node | `attributes.resource` |
| Rust `GeometryRepresentationRef` | `DrawingRepresentationRef` |
| IFCDR descriptor role `modelspace` | `drawing` |
| Writer filename `resources/model-space.ifcdr.json` | `resources/drawing.ifcdr.json` |

`Drawing` is the drawing with its layouts and relationships.
`DrawingRepresentation` connects it to the drawing content in an IFCDR
resource. `DrawingLayout` selects a scope from that content. The relation
`children.Representation` and public contextual methods `representation()`
and `resource()` retain their names.

Rename internal geometry-representation helpers and lookup maps consistently,
including `geometry_representation(s)` and `geometry_ifcdr_by_path`. Do not
rename actual geometric concepts such as geometric bounds or CAD geometry.
The writer resource URI constant becomes `DRAWING_RESOURCE_URI`.

The resource descriptor contains `format`, `version`, `resourceId`, `uri`,
`checksum`, and `role`. Its role is `drawing`. `version` describes the IFCDR
content, not the version of the IFCX overlay. Resource IDs are caller-supplied
opaque identities: examples and renewed fixtures use drawing-oriented names,
but readers must not reject or rewrite an ID because its spelling contains
`geometry` or `modelspace`. URIs likewise need not match the writer default.

Example representation (checksum abbreviated; this is not a complete package):

```json
{
  "path": "drawing-representation-0",
  "type": "openaec:DrawingRepresentation",
  "attributes": {
    "resource": {
      "format": "openaec.ifcdr",
      "version": "0.7.0",
      "resourceId": "drawing-main",
      "uri": "resources/drawing.ifcdr.json",
      "checksum": "sha256:...",
      "role": "drawing"
    }
  }
}
```

## One drawing resource, selected by layouts

Each Drawing has one `children.Representation` reference to a
DrawingRepresentation. Each layout listed by that Drawing references the
same representation node through its own `children.Representation`. The
representation identifies one IFCDR resource; each layout's `scopeId` must
resolve in that resource. Different layouts may select different scopes.

The equality rule compares representation node paths, not file locations or
merely equal resource IDs. A second representation node pointing to the same
resource does not satisfy a layout's obligation to reference its Drawing's
representation. Existing resource identity checks still apply independently.

The new rule does not add package-wide ownership restrictions for resources,
representations or layouts. It does not require exactly one model scope, a
particular numeric scope kind, or a unique scope per layout. Those additional
rules are outside this naming step. Existing model/paper layout kinds and
scope metadata remain available.

The package graph validator checks representation equality after establishing
that the Drawing and its listed layouts have references of the expected kind.
A mismatch prevents a strict validated package, with a diagnostic identifying
the Drawing, layout, expected representation and actual representation. Missing
or wrong-kind targets retain the existing reference diagnostics; do not add
dependent equality errors when the necessary graph relationship is invalid.

Public navigation then guarantees that `drawing.representation()` and each of
its `layout.representation()` views refer to the same node. Entity iteration
continues to select content by scope, not by resource role or filename.

## Version and compatibility proposal

Introduce IFCX overlay `0.8.0`, with the new node and descriptor names and a
descriptor role of `drawing`. Keep IFCDR and its JSON mapping at `0.7.0`, IFCPR
at `0.2.0`, and the candidate collection at unpublished suite `1.1.0`.

The drawing-core schema stays at `0.2.0`: its local reference shapes and
`Representation`/`Layouts` fields do not change. The cross-node equality rule
belongs in a normative package drawing-resource contract next to the new
overlay, backed by package validation and conformance tests.

The new reader does not migrate the retired representation vocabulary. This
continues the project's choice to renew development fixtures instead of
maintaining old-file compatibility. A recognized retired
`openaec:DrawingGeometryRepresentation` node must explicitly block strict
loading; it must not disappear into the generic unknown-node extension path.
Use a contextual unsupported-vocabulary diagnostic. A new representation node
with only `attributes.geometry` fails the required `resource` shape check.

Unrelated unknown IFCX node types and permitted extension fields remain open.
Historical schemas and the frozen `conformance/1.0.0` collection are retained
unchanged. Replace the active candidate overlay copy and migrate its ordinary
fixtures; retain retired names only in explicit compatibility tests. Do not
relabel IFCDR content as 0.8.0 to match the overlay version.

## Writer and conversion behavior

The writer still builds one Drawing with one model layout and one external
IFCDR resource. It writes the new representation name, resource descriptor,
role and default filename. The representation's generated display name, if
retained, describes drawing content rather than model space. Model layout
names and the actual model scope may still be named `Model`/`ModelSpace`.

The reader's existing multiple-scope capability is demonstrated by a package
fixture containing model and paper layouts referencing one resource. This does
not add paperspace export, viewport semantics, block support or rendering.
The converter retains its existing source coverage and loss classifications;
only representation API names and fixture references change.

## Required evidence

- Schema tests accept the new node/descriptor/role and reject missing resource
  descriptors and invalid roles. Historical schemas retain their old behavior.
- Graph tests accept one Drawing whose model and paper layouts reference the
  same representation, and reject a layout referring to another valid
  representation, even if both representations name the same IFCDR resource.
- Existing missing-node, wrong-node-kind and missing-scope tests remain valid;
  failures do not produce misleading dependent mismatch diagnostics.
- A complete production-reader fixture loads two layouts/scopes from one
  IFCDR resource. Typed navigation agrees on the representation and resource;
  entity order remains independently correct within each selected scope.
- Explicit retired-vocabulary cases cannot produce a strict package. Unrelated
  unknown-node and open-extension tests continue to pass.
- Writer output uses the new vocabulary and filename and reloads through the
  production reader. Existing semantic and deterministic converter roundtrips
  remain covered. Supplied resource IDs and arbitrary valid URIs remain valid.
- Candidate copies match active contract assets. Recompute checksums whenever
  fixture resource bytes change, including identity changes, while preserving
  intentional negative checksums and existing IFCPR fixture meaning.
- README, compatibility/provenance material, API examples and ROADMAP describe
  the new terminology. Milestone 2 remains Current with inline resources,
  richer reporting and initial measurements still outstanding.

Implementation verification requires formatting, Clippy with warnings denied,
workspace tests and focused doc tests. Work stays in this task without agents;
commit, push and merge require separate user authorization.
