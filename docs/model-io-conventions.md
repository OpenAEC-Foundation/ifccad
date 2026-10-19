# Active model IO conventions

The root package is now `ifccad`. Import public model and IO types directly from
`ifccad`, replacing the previous `ocdraw::ifccad` path. The core owns construction,
validation, preparation, encoding and storage. `ifccad-convert` connects owned
or validated IFCCAD models to `CadDocument`; CAD runtimes stay outside the core.
See the [IFCCAD lifecycle](experiments/ifccad-document-lifecycle.md).

OCDraw APIs described in the historical conventions below are parked, not exported
by the active workspace. Restore their full checkpoint for legacy work; see
[parking](ocdraw-parking.md). No deprecated compatibility facade is provided.

## Historical conventions before parking


The provisional Rust API uses explicit model and conversion direction names.
OCDraw and IFCCAD remain independent models with independent encoding and
validation contracts. Naming and responsibility conventions do not by themselves
change logical schema definitions or conversion coverage. Each additional
physical encoding, including IFCCAD CBOR, has its own explicit mapping contract.

## Responsibilities

| Responsibility | OCDraw | IFCCAD |
| --- | --- | --- |
| Owned editable model and shared semantic validation | `logical/`, `OcdrawDocument` | `logical/`, `IfccadDocument` |
| Fresh typed construction | `build.rs`, `build/`, `OcdrawBuilder` | Public logical records and checked ID allocators |
| Production byte reading | `read.rs`, `load_ocdraw_bytes` | `read.rs`, `load_ifccad_bytes` |
| Immutable IFCX source composition | Not applicable | `source/`, `LoadedIfccadGraph` |
| Physical field mapping and validation | `codec/json/` | `codec/json/` |
| Validation, encoding and strict readback orchestration | `encode.rs` | `encode.rs` |
| Encoded bytes and file storage | `storage.rs`, `EncodedOcdraw` | `storage.rs`, `EncodedIfccad` |
| CAD conversion | `ocdraw-convert` | `ifccad-convert` |

Both converters use `from_cad` and `to_cad`, with `options`, `diagnostics` and
`outcome` modules. CAD source auditing stays in `source`; native mapping helpers
and numerical kernels remain separate responsibilities. File granularity need
not match where the implementations differ. Shared unit mapping belongs in
`units`, independent of either conversion direction. Coverage contracts live under each
converter's `docs/`: `FROM-CAD-COVERAGE.md` and `TO-CAD-COVERAGE.md`.

## Routes and ownership

The separate [IFCCAD CBOR route](experiments/ifccad-cbor.md) uses
`load_ifccad_cbor_bytes`, `encode_ifccad_cbor_document` and
`load_ifccad_cbor_file`, with `ValidatedIfccadCbor`/`EncodedIfccadCbor`.
It shares the logical validator and document converter, and has no IFCX graph.
Its direct typed mapping lives in `src/ifccad/codec/cbor`; JSON entrypoints
retain their original meaning. Float bits and supplied definition vector order
are retained by CBOR readback; JSON keeps its own documented normalizations.

Byte readers return `Result<ValidatedOcdraw, OcdrawReadError>` or
`Result<ValidatedIfccad, IfccadReadError>`. Invalid input yields no editable
document. OCDraw errors retain invalid/unsupported-version classification and
all structured diagnostics. IFCX errors retain the existing `IfccadReport`.
File readers distinguish IO failures from production read failures.

Validated wrappers expose `document()` and `into_document()`. Clone the borrowed
logical document when an independent editable copy is needed. Validate raw
edits before conversion; encoding validates them again and requires production
readback. Validation neither encodes nor saves files. OCDraw bound recomputation
remains an explicit, transactional operation; encoding retains valid supplied
bounds. `OcdrawBuilder::finish()` is construction plus encoding.

Encoders return owned `EncodedOcdraw` or `EncodedIfccad` values with `bytes()`,
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

Each model has four routes (replace `ocdraw` with `ifccad` for IFCCAD):

- `cad_document_to_ocdraw_document`: fresh logical construction.
- `cad_document_to_encoded_ocdraw`: logical construction followed by core encoding.
- `ocdraw_document_to_cad_document`: validation and conversion of an owned model.
- `ocdraw_source_to_cad_document`: conversion of a validated reader snapshot.

OCDraw keeps `_with_id` variants for caller-supplied drawing identity. IFCX takes
`IfccadTargetMetadata` for explicit header and drawing identity. Every route
requires direction-specific options; `Default::default()` selects the existing
policy. No `Direct`, bare `Import`/`Export`, or `_with_options` duplicate API is
needed. Outcomes expose `document()`/`into_document()` or
`encoded()`/`into_encoded()`, plus diagnostics and mappings.

IFCX source conversion additionally assesses foreign graph content and original
numeric precision. Its document route cannot assess discarded source context.
Both converters use the same public unit-aware hard tolerance and numerical
proof engine in `cad-geometry-convert`. IFCCAD evidence resolves each Paper coordinate
domain independently. Within-limit rounding is distinct from semantic loss;
raw-source projection and CAD setter checks remain exact guards.
Model-specific diagnostic identities and source inventories stay separate.
Recovery remains distinguishable from semantic loss. The core neutral primitive
validation lives in `ifccad::geometry_kernel`, independently of any CAD runtime or IFCX
schema. See [shared geometry](geometry/shared-geometry.md).

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
| `read_native_cad_ifcx`, `_with_policy` | `load_ifccad_bytes(bytes, IfccadReadOptions { composition_policy })` |
| `write_native_cad_ifcx` | `encode_ifccad_document`, returning `EncodedIfccad` |
| IFCX encoder's `Vec<u8>` | `EncodedIfccad::bytes()` or `into_bytes()` |
| `raw_ifcx()` | `graph().composed_ifcx()` |
| `cad_document_to_drawing` | `cad_document_to_encoded_ocdraw` |
| `ocdraw_to_cad_document` | `ocdraw_source_to_cad_document` |
| `cad_document_to_ifcx_cad` | `cad_document_to_encoded_ifccad` |
| `ifcx_cad_to_cad_document` | `ifccad_source_to_cad_document` |
| IFCX `_with_options` pairs | One canonical function with required options |
| `ExportOptions`, `ImportOptions`, `ConversionLossPolicy` / `ExportLossPolicy` | `CadToOcdrawOptions`, `OcdrawToCadOptions`, `OcdrawLossPolicy` |
| `IfcxCadConversionOptions` | Distinct `CadToIfccadOptions` / `IfccadToCadOptions`, with model-qualified metadata and loss policy |
| `DirectExportOutcome`, `OcdrawDocumentExportOutcome`, `DirectImportOutcome` | `CadToEncodedOcdrawOutcome`, `CadToOcdrawDocumentOutcome`, `OcdrawToCadOutcome` |
| `CadToIfcxCadOutcome` | `CadToEncodedIfccadOutcome` |
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

See the [OCDraw lifecycle](../legacy/ocdraw/docs/ocdraw-document-lifecycle.md) and
[IFCCAD lifecycle](experiments/ifccad-document-lifecycle.md) for model-specific
guarantees and the converter READMEs for executable examples.

## IFCCAD naming migration

The provisional IFCCAD API and all repository consumers use the project name
consistently. This is a breaking naming migration without deprecated aliases.

| Previous name | Current name |
| --- | --- |
| `ocdraw::ifcx_cad`, `src/ifcx_cad` | `ifccad`, `src/ifccad` |
| `ifcx-cad-convert`, `ifcx_cad_convert` | `ifccad-convert`, `ifccad_convert` |
| `IfcxCadDocument`, other `IfcxCad…` types | `IfccadDocument`, corresponding `Ifccad…` types |
| `ValidatedIfcxCad`, `EncodedIfcxCad` | `ValidatedIfccad`, `EncodedIfccad` |
| `LoadedIfcxGraph`, `IfcxCompositionPolicy` | `LoadedIfccadGraph`, `IfccadCompositionPolicy` |
| `load_ifcx_cad_bytes`, `load_ifcx_cad_file` | `load_ifccad_bytes`, `load_ifccad_file` |
| `validate_ifcx_cad_document`, `encode_ifcx_cad_document` | `validate_ifccad_document`, `encode_ifccad_document` |
| Conversion function segment `ifcx_cad` | `ifccad`, retaining explicit source/target direction |
| Mapping lookup `ifcx_id(handle)` | `ifccad_id(handle)` |
| Browser `open_ifcx`, `convert_cad_to_ifcx`, `export_ifcx` | `open_ifccad`, `convert_cad_to_ifccad`, `export_ifccad` |
| Browser request kind/drawing format/native export format `ifcx` | `ifccad` |
| `schemas/ifcx-native-cad`, `examples/ifcx-native-cad`, `conformance/next/ifcx-native-cad` | Corresponding `ifccad` directories |
| `experimental-profile-0.1.0.ifcx` | `ifccad-profile-0.1.0.ifcx` |
| `ocdraw-browser`, `ocdraw_browser`, `crates/ocdraw-browser` | `browser`, `crates/browser` |
| `ocdraw-viewer`, `ocdraw_viewer`, `crates/ocdraw-viewer` | `viewer`, `crates/viewer` |
| Generated `ocdraw_browser.js`, `ocdraw_browser_bg.wasm` | `browser.js`, `browser_bg.wasm` |

The shared `browser` and `viewer` adapters serve both IFCCAD and OCDraw.
Their crate names, imports, workspace paths and generated WebAssembly assets
use these format-neutral names. The web application package is `format-explorer`.

The root `ocdraw` crate and its OCDraw APIs keep their domain names. The
underlying IFCX envelope uses `ifcxVersion: "ifcx_alpha"`, `.ifcx` files and
the existing `ifccad::` attributes and `urn:example:ifccad:0.1.0` import.
`graph().composed_ifcx()` continues to identify the raw IFCX envelope. Model
semantics, identity, allocation state, composition, conversion coverage and
numerical rules are unchanged. Native IFCCAD downloads use format identifier
`ifccad` with `.ifcx` filenames and preserve exact source bytes.

## Typed error causes

IFCCAD conversion retains `IfccadConversionError` with typed validation,
encoding, readback and allocation causes. Core encoding distinguishes invalid
documents, encoding and strict readback failures through `IfccadEncodeError`.
Callers can inspect variants and error source chains without parsing display
text. Located diagnostics, Recovery classification and Allow/Reject behavior
retain their existing contracts.

## Layout output revision

Both models retain layout media without complete plot settings. Plot unit and
fixed mapping determine Paper output meaning; IFCCAD no longer stores an independent
Paper coordinate unit. Effective plot settings, limits and layout PSLTSCALE have
separate native/CAD coverage. The provisional field/API migration, strict physical
scalar conversion limits, raster restrictions and per-domain accuracy reports are
specified in [layout output](layout-output.md). No new workspace state, renderer, release or controlled measurement is implied.

## IFCCAD opaque entity migration

`IfccadEntity` is now an ordered sum type. Wrap previous native construction in
`IfccadEntity::Native(IfccadNativeEntity { ... })`; use `id()` for shared identity
and `as_native[_mut]()` / `as_opaque[_mut]()` to inspect variants. Required native
layer/appearance/kind fields remain strong. `IfccadDocument.preservation` is an
optional independent collection; `IfccadIdCounters.next_preservation_record_id`
and `allocate_preservation_record_id()` preserve reservations/deletions.
Library capture remains opt-in through direction-qualified options. Loaded graphs
stay immutable and fresh encode/load retains generic byte payloads; source graph
writeback is not implied. This is a provisional API/contract change, not a release.
## Workspace lifecycle

IFCCAD UCS and Model-window IDs use separate persistent uint64 watermarks and canonical full references. Whole workspace attributes compose as IFCX values. Native readers/writers preserve omitted choices and dormant values; legacy missing canvas activation means true. The optional viewport snapshot is boxed in Rust to keep ordinary entity variants compact. Format-owned converters allocate/bind identities before references and strictly read back native output.


## Native color API migration

`DrawingColor` and independent `IfccadColor` are enums with `Foreground` and
`Concrete { rgb, indexed, named }` variants. `rgb(r,g,b)` still constructs a fixed
color. Metadata builders `with_indexed` and `with_named` now return `Result`;
they refuse metadata on Foreground instead of silently dropping it. Existing
callers must propagate or handle `DrawingColorMetadataError` or
`IfccadColorMetadataError`. Shared native validation rejects concrete ACI 7.

Use variant matching to distinguish symbolic meaning. `concrete_rgb`,
`indexed_identity` and `named_identity` expose optional values without resolving
Foreground against an invented background. Rendering consumers can use
`ocdraw::color::foreground_srgb` with their effective sRGB background; the
drawing stores no theme-specific resolved color. See [presentation](presentation.md#foreground-color).

## Complex line pattern values

Public pattern arrays now contain typed `LinePatternElement<Marker>` values. Use
`LinePatternElement::simple(length)` for existing numeric patterns; marked entries
carry a native text-style or symbol-source reference plus a shared transform.
The separate source table has its own persistent allocation watermark. Both
production codecs retain plain numbers for unmarked entries and write closed
objects for markers. IFCCAD references use complete profile-local paths. See
[line patterns](line-patterns.md) for the logical/physical distinction, estimates
and conversion limits.

## Native Hatch gradient API

`HatchFill::Gradient` selects `DrawingHatchEntity.gradient` or
`IfccadHatch.gradient`; the types are DrawingHatchGradient/IfccadHatchGradient
aliases over neutral HatchGradient with each model's concrete color.
With the Foreground/Concrete API, endpoints require Concrete; no background is
invented to turn Foreground into a gradient endpoint.
HatchEntityDefinition also requires gradient:None/Some explicitly. Existing
callers migrate with None; Gradient requires type/colors, while inactive state
may be partial. No compatibility bridge for unpublished intermediate schemas.
OCDraw outgoing Hatch failures now retain entity_id and the original typed
cause in OcdrawToCadError::HatchPreparation. Model and actual-file guarantees
remain separate; caller options lossPolicy=reject also enforce physical losses.
