# Candidate IFCCAD CBOR Zstd transport fixtures

These fixtures exercise the [experimental transport contract](../../../schemas/ifccad/cbor-zstd-transport-1.md).
The production Rust reader consumes `cases.json` under feature `ifccad-cbor-zstd`.
They cover known/unknown content size, level-independent reading, signed-zero
preservation, required checksum, corruption/truncation, extra frames/bytes and
inner revision rejection. Raw payloads are the independently authored neighboring
CBOR fixtures. This collection does not modify any released conformance set.

`generate.py` reproduces the files using Python 3.14.7's `compression.zstd`
(libzstd 1.5.7), independently of the production Rust writer. Run it with
`python conformance/next/ifccad-cbor-zstd/generate.py`. Byte identity across
different compressor versions is not required. Valid cases are decoded through
the production reader and compared by re-encoding their raw documents, including
floating-point bits. Invalid cases name the expected public error category.
