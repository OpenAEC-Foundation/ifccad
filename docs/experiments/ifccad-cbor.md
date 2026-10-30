# Experimental standalone IFCCAD CBOR

IFCCAD has two independent physical mappings of its editable `IfccadDocument`:
the existing IFCX JSON CAD profile and this standalone CBOR candidate. The
binary file is not a general IFCX serialization and is not automatically
readable by IFCX JSON applications. OCDraw is parked as a frozen reference outside the active workspace.

The [development revision-3 contract](../../schemas/ifccad/cbor-mapping-3.md) and
[complete field register](../../schemas/ifccad/cbor-mapping-3.json) define the
language-neutral physical mapping. Conformance is in
`conformance/next/ifccad-cbor`; published alpha 2026.44.0 fixes the previous
revision-1 snapshot; 2026.44.1 fixes revision 2 and profile 0.2.0. The active
revision-3 candidate adds primitive extrusion thickness alongside profile 0.3.0.
This remains experimental, with no stable-format or compatibility promise.

## Structure and evolution

The complete file is one definite-length CBOR map:
`{0: "IFCCAD-CBOR", 1: 3, 2: Document}`. File names conventionally use
`.ifccad.cbor`. Each record has its own small unsigned field codes; entity
kind codes and document identities have separate meanings. Source order in
Rust does not assign codes. The registry explicitly fixes every field/variant.

The candidate reader accepts only mapping revision 3. Unknown fields, variants or
revisions are errors; there is no compatibility migration or silent skipping.
Revision 1 is published with native source alpha 2026.44.0. Later normative
changes advance the single active mapping and fixtures under the
[release/version policy](../releases.md). There are no mandatory historical
mappings, compatibility promises for earlier experimental files or migration
routes. Dimension references are required XYZ; the dimension-line point is XY in its placed plane.

All supported logical content survives: IDs/counters, ownership/draw order,
definition vector order, native geometry, Paper/Model state, text, patterns,
Hatch, all nine supported Dimension kinds with bounded selectors and saved
graphics, and typed opaque preservation.
Binary readback does not reorder otherwise unordered definition collections.

Defaults are registered explicitly. Every stored float is binary64, including
negative zero and subnormals; integers retain their logical ranges. Only
line-point +0 Z may become an XY tuple. Authored option/reset presence and
inactive settings remain intact. Preservation payload/baseline bytes use byte
strings without Base64 conversion. No provider payload is interpreted here.

Raw CBOR remains the default. An optional experimental Zstd transport is described
below. Columns, numeric buffers, dynamic dictionaries, chunks, partial loading,
external URI integration and quantization remain outside this candidate.

## Next investigation

The [provisional standalone CAD scope](ifccad-document-lifecycle.md#provisional-standalone-cad-scope--2026-10-21)
keeps this experiment focused on the supported drawing document. A full IFCX
source snapshot beside CBOR, a general fragment encoding and mixed-document
writeback are deferred. CAD semantic coverage and qualified exchange remain
the main development priority; each semantic slice keeps the single active
mapping, shared validation and production readback aligned, advancing identifiers
when the published contract changes.

A useful next optional measurement is a bounded whole-document comparison of
compact IFCX JSON, raw CBOR and the native Rust CBOR + Zstd route at default
level 3. Agree the exact inputs, retained content, dependency configuration and
reference hardware before a separately requested run. Record full opening and
writing, separating filesystem IO, decompression, parsing/composition, shared
validation and mandatory writer readback where observable, plus peak/resident
memory. Keep CAD conversion costs separate. JSON source-graph retention and
projection-only binary retention must be disclosed; an equal-content encoding
comparison must not treat discarded source context as a compression gain.

The existing external compression probes do not measure this native end-to-end
route. Retain their reports and matching results with their original scope;
they neither qualify later revisions nor authorize reruns. A production host's
full load/convert/save costs require separate evidence from codec-only timings.

This is a proposed follow-up, not an approved corpus, implementation task or
measurement run. It selects no chunks, columns, dictionaries or new revisions.
Reconsider those only against a concrete workload and measured remaining cost,
following the [roadmap](../../ROADMAP.md#later-physical-encodings-and-performance).

## Core API

```rust,no_run
use ifccad::{
    encode_ifccad_cbor_document, load_ifccad_cbor_bytes,
    IfccadDocument, IfccadCborEncodeError,
};

fn encode(document: &IfccadDocument) -> Result<Vec<u8>, IfccadCborEncodeError> {
    Ok(encode_ifccad_cbor_document(document)?.into_bytes())
}

let bytes: &[u8] = &[]; // Supply a complete revision-1 file.
let result = load_ifccad_cbor_bytes(bytes, Default::default());
```

`load_ifccad_cbor_bytes` returns `ValidatedIfccadCbor`, with `document()` and
`into_document()`. It does not invent an IFCX graph or source snapshot.
`encode_ifccad_cbor_document` validates, writes, loads through the production
CBOR reader and compares every document value including float bits. It returns
`EncodedIfccadCbor` with `bytes()`, `into_bytes()`, `AsRef<[u8]>`, byte equality
and `write_file()`. File writing creates a new file and refuses overwrite.

`IfccadCborReadError` distinguishes Malformed, UnsupportedRevision,
ResourceLimit and InvalidDocument; the latter retains `IfccadReport` through
`report()` and its error source. Encoding distinguishes InvalidDocument,
Mapping, Readback and SemanticMismatch. `load_ifccad_cbor_file` separately
reports IO failures through `IfccadCborOpenError`.

Default configurable read limits: 512 MiB input, depth 128, 16 million entries
per container and 32 million CBOR items total. A map key and value each count;
root depth is one. File input is bounded while reading, and claimed lengths
are checked before allocation. Limits are resource settings, not semantic
format maxima. Internal encoder readback uses finite limits derived from the
actual output. No public unbounded mode or skip-readback switch is provided.

## Explicit file conversion

```text
cargo run --example convert_ifccad_cbor -- to-cbor INPUT.ifcx OUTPUT.ifccad.cbor
cargo run --example convert_ifccad_cbor -- to-json INPUT.ifccad.cbor OUTPUT.ifcx
```

This converts the supported CAD projection. Original IFCX fragments, schema
imports and foreign nodes are outside `IfccadDocument` and are not copied into
CBOR or reconstructed on JSON export. Existing JSON source-download and source
audit behavior remains separate. Binary-to-CAD conversion uses
`ifccad_document_to_cad_document(validated.document(), options)`; no IFCX
source-graph audit can be inferred from a binary document.

The example never overwrites a file or guesses a format from its extension.
The Explorer download menu also offers CBOR and CBOR + Zstd explicitly, marked
experimental. These use the same production writers and strict readback. Ordinary
IFCX source download keeps the original bytes. Binary downloads contain the drawing
projection only, as noted in the menu. The Explorer opens `.cbor` and `.cbor.zst`
(or `.zst`) through explicit application-level reader selection. It bounds input
and decompressed bytes to 64 MiB, retains the binary document for conversion and
labels its generated inspection JSON/nodes as decoded content. This display
projection is not an original IFCX graph or source audit. DXF/DWG output uses
typed document conversion directly; JSON download reconstructs an IFCX profile
file. Same-encoding binary download preserves the original validated bytes.
New converter convenience entrypoints remain deferred.

## Optional experimental Zstd

Enable the Cargo feature `ifccad-cbor-zstd` to use this separate core route.
It is disabled by default in the core. The browser adapter explicitly enables
the feature for experimental downloads, using the existing Zstd implementation
and its WebAssembly allocator shim. Building that adapter requires a
WebAssembly-capable C compiler (Clang on CI); no server-side file upload or
JavaScript reimplementation of the encoding is involved.

```text
cargo run --features ifccad-cbor-zstd --example convert_ifccad_cbor -- to-cbor INPUT.ifcx OUTPUT.ifccad.cbor.zst --zstd
cargo run --features ifccad-cbor-zstd --example convert_ifccad_cbor -- to-cbor INPUT.ifcx OUTPUT.ifccad.cbor.zst --zstd --level 9
cargo run --features ifccad-cbor-zstd --example convert_ifccad_cbor -- to-json INPUT.ifccad.cbor.zst OUTPUT.ifcx --zstd
```

`encode_ifccad_cbor_zstd_document(document, options)` accepts
`IfccadCborZstdEncodeOptions { compression_level: 9 }`; `Default::default()` uses
3. Supported levels are 1 through 22. `EncodedIfccadCborZstd` exposes `bytes()`,
`into_bytes()` and create-new `write_file()`. The byte/file readers are
`load_ifccad_cbor_zstd_bytes` and `load_ifccad_cbor_zstd_file`, with explicit
`IfccadCborZstdReadOptions`. Neither the raw reader nor filenames auto-select
compression.

The [transport contract](../../schemas/ifccad/cbor-zstd-transport-1.md) requires
one standard, checksummed Zstd frame containing ordinary revision-1 CBOR, without
a dictionary. Standard magic/header bytes identify Zstd. Compression level is a
writer setting, is not stored in that header and is unnecessary for reading.
No private container or inner field codes are introduced. Writer output passes
strict production readback and exact document comparison, including float bits.

Default bounds are 512 MiB compressed input, 512 MiB decompressed CBOR and
a 128 MiB frame window; inner CBOR structural limits remain active. Bounds are
configurable. Checksums, frame completion and declared/actual sizes are checked;
concatenated frames, trailing bytes, dictionaries and nested compression are
rejected. Transport, resource, decompression, inner validation and IO errors
remain distinguishable. This is whole-document compression with no partial
loading or incremental save contract. Future IFCX compression remains independent.

## Verification boundary

Production readback, shared logical validation, explicit code/register checks,
independent hand-authored binary fixtures, malformed-input tests and semantic
CAD document-conversion tests qualify this candidate. The codec uses direct
typed minicbor token IO; it does not traverse JSON as a conversion bridge.
This proves correctness on the covered contracts, not a size advantage over
DWG, faster loading, incremental saving or a cross-format canonical fingerprint.

## Current main alignment

The current draft includes fraction layout/scale and explicit resets introduced
by the Dimension repair, Hatch ordered source groups, association state,
provenance flags and text islands. HatchLoop code 1 remains unassigned; the
single register specifies the current fields.

JSON graph loading canonicalizes preservation records by numeric ID. CBOR retains
the supplied vector order, including reverse/multi-digit record IDs. Both retain
record contents and opaque draw order. This physical difference does not alter
provider restore qualification or permit silent semantic loss.

The [measurement protocol](../benchmarks/ifccad-cbor-baseline-v1.md) and
[accepted results](../benchmarks/ifccad-cbor-results-summary-v1.md) record the
qualified baseline scope; `test.dxf` remains deferred. Separate external probes
record [compressed sizes](../benchmarks/ifccad-foundation-zstd-vs-gzip-v1.md) and
[compression timings](../benchmarks/ifccad-foundation-compression-speed-v1.md).
Those probes explain the choice of default level 3; they do not measure the
new Rust transport implementation or establish its end-to-end load/write speed.
