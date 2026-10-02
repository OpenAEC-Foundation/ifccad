# Model IO and conversion conventions

The provisional Rust API uses explicit model and conversion direction names.
OCDraw and IFCX-CAD remain independent models with independent encoding and
validation contracts. This cleanup changes Rust call sites and module paths;
it does not change native files, schemas, format versions or conversion coverage.

## Responsibilities

| Responsibility | OCDraw | IFCX-CAD |
| --- | --- | --- |
| Owned editable model and shared semantic validation | `logical/`, `OcdrawDocument` | `logical/`, `IfcxCadDocument` |
| Fresh typed construction | `build.rs`, `build/`, `OcdrawBuilder` | Public logical records and checked ID allocators |
| Production byte reading | `read.rs`, `load_ocdraw_bytes` | `read.rs`, `load_ifcx_cad_bytes` |
| Immutable IFCX source composition | Not applicable | `source/`, `LoadedIfcxGraph` |
| Physical field mapping and validation | `codec/json/` | `codec/ifcx_json/` |
| Validation, encoding and strict readback orchestration | `encode.rs` | `encode.rs` |
| Encoded bytes and file storage | `storage.rs`, `EncodedOcdraw` | `storage.rs`, `EncodedIfcxCad` |
| CAD conversion | `ocdraw-convert` | `ifcx-cad-convert` |

Both converters use `from_cad` and `to_cad`, with `options`, `diagnostics` and
`outcome` modules. CAD source auditing stays in `source`; native mapping helpers
and numerical kernels remain separate responsibilities. File granularity need
not match where the implementations differ. Shared unit mapping belongs in
`units`, independent of either conversion direction. Coverage contracts live under each
converter's `docs/`: `FROM-CAD-COVERAGE.md` and `TO-CAD-COVERAGE.md`.

## Routes and ownership

Byte readers return `Result<ValidatedOcdraw, OcdrawReadError>` or
`Result<ValidatedIfcxCad, IfcxCadReadError>`. Invalid input yields no editable
document. OCDraw errors retain invalid/unsupported-version classification and
all structured diagnostics. IFCX errors retain the existing `IfcxCadReport`.
File readers distinguish IO failures from production read failures.

Validated wrappers expose `document()` and `into_document()`. Clone the borrowed
logical document when an independent editable copy is needed. Validate raw
edits before conversion; encoding validates them again and requires production
readback. Validation neither encodes nor saves files. OCDraw bound recomputation
remains an explicit, transactional operation; encoding retains valid supplied
bounds. `OcdrawBuilder::finish()` is construction plus encoding.

Encoders return owned `EncodedOcdraw` or `EncodedIfcxCad` values with `bytes()`,
`into_bytes()` and `write_file()`. Both implement `AsRef<[u8]>`, `PartialEq` and
`Eq`. Equality compares the exact encoded bytes; differing encodings can still
represent equivalent logical documents. Only validated encoding constructs them.
`write_file()` creates a new file and refuses overwrite. Replacement, backup
and editor-session policies belong to applications.

IFCX additionally retains immutable exact source bytes, the composed graph and
its composition policy through `graph()` and `into_parts()`. Editing its CAD
projection does not update this source. Original downloads use
`graph().source_bytes()`; profile encoding produces a fresh CAD-profile file.
It receives no original fragments or foreign content and provides no merge.

## Canonical conversion API

Each model has four routes (replace `ocdraw` with `ifcx_cad` for IFCX-CAD):

- `cad_document_to_ocdraw_document`: fresh logical construction.
- `cad_document_to_encoded_ocdraw`: logical construction followed by core encoding.
- `ocdraw_document_to_cad_document`: validation and conversion of an owned model.
- `ocdraw_source_to_cad_document`: conversion of a validated reader snapshot.

OCDraw keeps `_with_id` variants for caller-supplied drawing identity. IFCX takes
`IfcxCadTargetMetadata` for explicit header and drawing identity. Every route
requires direction-specific options; `Default::default()` selects the existing
policy. No `Direct`, bare `Import`/`Export`, or `_with_options` duplicate API is
needed. Outcomes expose `document()`/`into_document()` or
`encoded()`/`into_encoded()`, plus diagnostics and mappings.

IFCX source conversion additionally assesses foreign graph content and original
numeric precision. Its document route cannot assess discarded source context.
OCDraw retains its unit-aware hard tolerance and geometry assessment. IFCX
retains exact translation/projection and CAD setter checks. These necessary
differences remain explicit; no shared tolerance or generic diagnostic model is
introduced. Recovery diagnostics remain distinguishable from semantic losses.

## Migration from the previous provisional API

| Previous API | Replacement |
| --- | --- |
| `DrawingBuilder`, `DrawingOptions`, `DrawingBuildError` | `OcdrawBuilder`, `OcdrawBuildOptions`, `OcdrawBuildError` |
| `ValidatedDrawing`, `DrawingDiagnostic`, `EncodedDrawing` | `ValidatedOcdraw`, `OcdrawDiagnostic`, `EncodedOcdraw` |
| `load_drawing_bytes`, `DrawingLoadOutcome` | `load_ocdraw_bytes`, `Result` with `OcdrawReadError` |
| `DrawingLoadStatus` | `OcdrawReadStatus`; successful `Result` means Valid |
| `load_drawing_file`, `DrawingOpenError` | `load_ocdraw_file`, `OcdrawOpenError` with IO/Read variants |
| `DrawingWriteError` | `OcdrawWriteError` |
| `validate_document`, `recompute_document_bounds`, `encode_document` | Model-qualified `validate_ocdraw_document`, `recompute_ocdraw_document_bounds`, `encode_ocdraw_document` |
| `read_native_cad_ifcx`, `_with_policy` | `load_ifcx_cad_bytes(bytes, IfcxCadReadOptions { composition_policy })` |
| `write_native_cad_ifcx` | `encode_ifcx_cad_document`, returning `EncodedIfcxCad` |
| IFCX encoder's `Vec<u8>` | `EncodedIfcxCad::bytes()` or `into_bytes()` |
| `raw_ifcx()` | `graph().composed_ifcx()` |
| `cad_document_to_drawing` | `cad_document_to_encoded_ocdraw` |
| `ocdraw_to_cad_document` | `ocdraw_source_to_cad_document` |
| `cad_document_to_ifcx_cad` | `cad_document_to_encoded_ifcx_cad` |
| `ifcx_cad_to_cad_document` | `ifcx_cad_source_to_cad_document` |
| IFCX `_with_options` pairs | One canonical function with required options |
| `ExportOptions`, `ImportOptions`, `ConversionLossPolicy` / `ExportLossPolicy` | `CadToOcdrawOptions`, `OcdrawToCadOptions`, `OcdrawLossPolicy` |
| `IfcxCadConversionOptions` | Distinct `CadToIfcxCadOptions` / `IfcxCadToCadOptions`; metadata and `IfcxCadLossPolicy` retain their names |
| `DirectExportOutcome`, `OcdrawDocumentExportOutcome`, `DirectImportOutcome` | `CadToEncodedOcdrawOutcome`, `CadToOcdrawDocumentOutcome`, `OcdrawToCadOutcome` |
| `CadToIfcxCadOutcome` | `CadToEncodedIfcxCadOutcome` |
| `DirectExportError`, `DirectImportError`, `ExportDiagnostic`, `DirectImportDiagnostic` | `CadToOcdrawError`, `OcdrawToCadError`, `CadToOcdrawDiagnostic`, `OcdrawToCadDiagnostic` |
| `ConversionGeometryTolerance`, `ConversionToleranceError`, `ConversionGeometryAssessment` and related lifecycle evidence types | `OcdrawGeometryTolerance`, `OcdrawToleranceError`, `OcdrawGeometryAssessment` and model-qualified evidence types |
| Encoded outcome `drawing()` / `into_drawing()` | `encoded()` / `into_encoded()` |
| IFCX outcome `ifcx_bytes()` / `validated_ifcx()` | `encoded().bytes()` / `validated_source()` |
| Dependency/reexport `cadcodec` | `opencadcodec` |

This is a deliberate breaking cleanup of the provisional Rust API, without a
deprecated parallel facade. Repository consumers and examples migrate together.
The upstream package, fixed revision and features are unchanged. Historical
reports keep their original dependency alias and provenance where applicable.
IDs, allocation watermarks, ordered membership, loss messages and numerical
guarantees retain their contracts. New geometry, editor context, source writeback
and merging edited copies remain outside this task.

See the [OCDraw lifecycle](ocdraw-document-lifecycle.md) and
[IFCX-CAD lifecycle](experiments/ifcx-cad-document-lifecycle.md) for model-specific
guarantees and the converter READMEs for executable examples.

## Retained error-boundary follow-up

IFCX conversion retains its shared `IfcxCadConversionError` and existing
`CoreValidation(String)` payload in this cleanup. A future API refinement should
retain validation reports and encoding/readback errors as typed causes, allowing
callers to distinguish failures without parsing text. It must preserve located
loss diagnostics, Recovery classification and Allow/Reject behavior. A shared
conversion error can remain appropriate; separate direction errors are not a
prerequisite for typed causes. This refinement is not implemented here.
