# Candidate IFCCAD conformance

The [IFCCAD candidate](ifccad/README.md) exercises the provisional JSON/IFCX
profile through strict production native reading. The [base-CBOR candidate](ifccad-cbor/README.md)
qualifies its separate mapping revision and production reader. [Optional native
Zstd transport](ifccad-cbor-zstd/README.md) retains the same inner CBOR contract.
This migration changes none of those fixture bytes or semantics.

The unpublished OCDraw candidate and its case index are parked under
`legacy/ocdraw/conformance/next/ocdraw` at the repository root. They are outside
ordinary test discovery. Restore their complete frozen checkpoint for legacy work;
see [parking](../../docs/ocdraw-parking.md).

Numbered released collections remain immutable historical material. Their package
readers and schemas remain available from Git history and do not become requirements
of the active IFCCAD reader. No supported release is published by this migration.
