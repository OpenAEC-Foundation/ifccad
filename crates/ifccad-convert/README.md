# ifccad-convert

`ifccad-convert` connects the validated, typed IFCCAD model from the core
`ifccad` crate to cadcodec's `CadDocument`. It is a companion crate: the core
format implementation remains usable without cadcodec.

IFCCAD import is drawing-centric:

```rust,no_run
use ifccad::package::load_directory_package;
use ifccad_convert::drawing_to_cad_document;

# fn example(package_directory: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
let inspected = load_directory_package(package_directory)?;
let package = inspected
    .validated_package()
    .ok_or("the IFCCAD package is not strictly valid")?;
let drawing = package.drawings().next().ok_or("the package has no drawing")?;
let outcome = drawing_to_cad_document(drawing)?;

let document = outcome.document();
for diagnostic in outcome.diagnostics() {
    eprintln!("{diagnostic}");
}
# let _ = document;
# Ok(())
# }
```

The crate re-exports its pinned cadcodec dependency as
`ifccad_convert::cadcodec`. Consumers should use this re-export for
`CadDocument`, handles, entities, and DXF/DWG writers to avoid mixing cadcodec
revisions.

The direction names describe the boundary around `CadDocument`:

```text
IFCCAD -- import --> CadDocument -- cadcodec DXF writer --> DXF
DXF -- cadcodec DXF reader --> CadDocument -- export --> IFCCAD
```

Both directions are implemented. Import converts one validated IFCCAD drawing
to a `CadDocument`; export converts a complete `CadDocument` to one encoded,
in-memory IFCCAD directory package. Encoding and filesystem writing remain
separate operations. A direct IFCCAD-to-DXF or IFCCAD-to-DWG application can
use file-format-oriented names and does not need to expose this internal
import/export terminology.

## Current scope

- exactly one model layout on export;
- finite planar lines and straight lightweight polylines (`z = 0`), with
  unsupported entities diagnosed rather than approximated;
- IFCDR draw order and source-entity-to-target-handle mapping;
- IFCDR length units;
- layers, visibility, color (including named layer colors), line pattern, line
  weight, and ByLayer, ByBlock, or explicit opacity;
- structured, aggregated diagnostics for partially exported and skipped
  content, plus source-to-target entity mappings in both directions.

The converter accepts only a `DrawingRef` from a strictly validated package.
It does not load package paths or raw JSON and does not repeat package
validation.

The active package reader/writer contract is IFCDR 0.7.0 with IFCX overlay
0.9.0. `DrawingRepresentationRef` exposes the drawing resource through
`representation().resource()`; model and paper layouts share their Drawing's
representation and select scopes within it. The writer currently emits one
model layout and defaults to `resources/drawing.ifcdr.json`; callers can select
`DrawingResourceStorage::Inline` on the drawing builder. Validated inline and
external drawings use the same conversion API. Inline IFCPR reading does not
add preservation transfer to the converter. Unsupported IFCDR
versions or entity schemas block strict loading before import; the former
`IfcdrEntityRef::Unmodeled`, `UnmodeledEntityRef`, and
`ImportDiagnostic::UnmodeledEntitiesSkipped` APIs have been removed. Existing
line-pattern fallback and line-weight rounding diagnostics remain. Export
continues to diagnose unsupported CadDocument entities and properties under
its existing loss policy. See the
[compatibility matrix](../../conformance/next/COMPATIBILITY.md) for the separate
limits of reading, conversion, and IFCPR validation.

Multiple layouts, paperspace export, blocks, 3D geometry, other native export
entity kinds, and preservation transfer are deliberately deferred. The pinned
cadcodec coverage contract is documented in
[`src/export/COVERAGE.md`](src/export/COVERAGE.md): every public source-model
area must be represented, diagnosed, classified as non-semantic scaffolding,
or rejected as structurally invalid.

## Exporting a `CadDocument`

```rust,no_run
use ifccad::package::PackageOptions;
use ifccad::PackageId;
use ifccad_convert::cadcodec::CadDocument;
use ifccad_convert::{
    cad_document_to_package, ExportError, ExportLossPolicy, ExportOptions,
};

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let document = CadDocument::new();
let metadata = PackageOptions {
    package_id: PackageId::new("drawing-export")?,
    data_version: "1".into(),
    author: "Example application".into(),
    timestamp: "2026-09-04T10:00:00Z".into(),
};

// Allow is the default: supported content is returned together with every loss.
let outcome = cad_document_to_package(&document, metadata, ExportOptions::default())?;
for diagnostic in outcome.diagnostics() {
    eprintln!("{diagnostic:?}");
}
for (source_handle, target_entity_id) in outcome.entity_mapping().iter() {
    println!("{source_handle} -> {target_entity_id:?}");
}
let encoded_package = outcome.into_package();
encoded_package.write_directory("drawing-export")?;

// Reject performs the complete scan too, but returns no package when loss exists.
let strict_metadata = PackageOptions {
    package_id: PackageId::new("strict-export")?,
    data_version: "1".into(),
    author: "Example application".into(),
    timestamp: "2026-09-04T10:00:00Z".into(),
};
let strict = cad_document_to_package(
    &document,
    strict_metadata,
    ExportOptions {
        loss_policy: ExportLossPolicy::Reject,
    },
);
if let Err(ExportError::LossRejected { diagnostics }) = strict {
    eprintln!("strict export rejected {} losses", diagnostics.len());
}
# Ok(())
# }
```

`ExportError` covers invalid source structure, rejected source loss, package
construction, and internal conversion invariants. Once export succeeds,
`EncodedPackage::write_directory` performs the separate storage step. It never
overwrites an existing directory, and path or filesystem failures are reported
as `PackageWriteError`, not `ExportError`.

The initial exact native export subset is one model-space drawing, the drawing
unit, all representable layers, appearances, finite 2D `LINE` entities, and
straight finite 2D `LWPOLYLINE` entities. Layer/entity order is stable. CAD
handle numbers are technical identifiers and may change; `ExportEntityMapping`
records emitted source handles against their new IFCDR entity IDs. Semantic
relationships carried by handles are still diagnosed when they cannot be
represented.


## Assessment of an executed conversion

Both outcomes expose `transfer_assessment()` alongside their detailed diagnostics
and entity mapping. `conclusion()` is `LossDetected`, `NoLossDetected` or
`NotFullyAssessed`; `scope()`, `coverage()` and `limitations()` explain its reach.

Export assesses the pinned public CadDocument model according to
[export coverage](src/export/COVERAGE.md), excluding private runtime state and
original raw CAD bytes. With no recorded losses, this permits `NoLossDetected`
within that scope. Import assesses the selected drawing and currently has
[coverage gaps](src/import/COVERAGE.md), including metadata and opacity
quantization. Without recorded losses it returns `NotFullyAssessed`.

Recorded loss takes precedence while coverage limitations remain visible.
`Allow` still returns output with loss diagnostics; `Reject` still returns
`LossRejected` with those diagnostics and no package. Other typed errors also
describe failed attempts, not completed transfers. Summary access does not
rescan input, and the existing `into_parts()` tuples remain unchanged.
Package validation and transfer fidelity are separate: even a successfully
loaded IFCPR resource is not restored by this converter.

## Controlled size and exchange checks

The development example `size_baseline` compares controlled line/polyline
drawings across external/inline IFCCAD, text DXF and normal DWG. It measures
complete file bytes, checks exact recipe semantics and executes separate
conversion chains with Reject. The current report uses cadcodec 0.5.4 at
`2f2cd25832db298524fb5eb36ced5a438a877e95` with two explicit local DWG fixes;
normal Cargo commands use the unmodified pin and still encounter the known
DWG return-conversion failures. See the [patch setup](../../patches/cadcodec-upstream/README.md)
and the [complete experiment report](../../docs/benchmarks/size-baseline-v1.md). Failed checks
produce an explicitly incomplete report; they do not change converter policy.
