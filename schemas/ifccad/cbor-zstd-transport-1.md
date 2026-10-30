# Experimental IFCCAD CBOR Zstd transport

This optional transport wraps the [active CBOR mapping](cbor-mapping-3.md)
without changing its logical model, field codes or payload bytes. Uncompressed
CBOR remains the default. This is an independent IFCCAD experiment, not a general
IFCX compression contract or a promise about future IFCX encodings.

## Physical contract

This is transport revision 1. No private transport-revision field exists in
the standard Zstd frame; the inner CBOR envelope identifies IFCCAD. After first
publication, changed framing acceptance advances both the documented transport
revision and the inner CBOR mapping under the [release policy](../../docs/releases.md).

A file contains exactly one ordinary Zstandard frame, beginning with the standard
magic bytes `28 B5 2F FD`. Its decompressed content is exactly one complete
revision-2 IFCCAD CBOR document in this development candidate. Transport revision
1 retains its physical frame contract; the fixed 2026.44.0 release carried inner
CBOR revision 1. The frame checksum is mandatory. Dictionary ID
fields and external dictionaries are prohibited. Skippable frames, concatenated
frames, trailing bytes and nested compression are rejected. Reserved frame bits
follow the Zstandard specification. The frame content size may be present or
absent; writers in this implementation emit it. Readers verify decompression,
the checksum, complete frame consumption and the entire inner CBOR contract.

The magic and standard frame header identify Zstd; no private container header
or compression-method field is added to CBOR. The inner CBOR envelope identifies
IFCCAD and its mapping revision. Compression level is a writer setting and is
not recorded in the standard Zstd header or needed for decoding. The conventional
extension is `.ifccad.cbor.zst`; selecting this transport is explicit.

The writer accepts integer levels 1 through 22, with default 3. It uses no
dictionary or parallel workers. Levels and library revisions may produce different
compressed bytes for the same document; compressed byte identity is not a
semantic identity contract. Writer output is loaded through the production
transport reader and compared with the input document, including float bits.

## Resource and error behavior

Default reader limits are 512 MiB compressed input, 512 MiB decompressed CBOR and
a 128 MiB frame window (`max_window_log = 27`). The decompressed bound is the
inner CBOR `max_input_bytes` option. Its structural limits and strict logical
validation still apply. Limits are configurable resource policies, not semantic
format maxima. Supported window-log settings are 10 through 31. Byte limits
must allow a detection byte in the platform's address space.

The reader checks declared content size and frame window before decoder
allocation, bounds actual decompressed output even when size is absent, and
bounds file input during reading. Invalid options, malformed framing, resource
limits, decompression/checksum failures, inner CBOR errors and filesystem errors
remain distinguishable. Inner validation reports are retained.

This whole-document transport offers no partial loading or incremental writing.
Chunking and independently compressed sections require a separate design.
The native Rust dependency is optional under `ifccad-cbor-zstd`, disabled by
default; enabling it for browser/WASM builds is outside this experiment.

Candidate interoperability fixtures are in
[`conformance/next/ifccad-cbor-zstd`](../../conformance/next/ifccad-cbor-zstd/README.md).
