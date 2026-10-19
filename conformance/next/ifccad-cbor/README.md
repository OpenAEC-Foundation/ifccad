# Candidate IFCCAD CBOR conformance

This independent candidate exercises standalone IFCCAD CBOR mapping revision 2.
`cases.json` lists valid and invalid binary files, expected error classes and
readable diagnostic sidecars. The test uses the production byte reader and
strict encoder readback. These fixtures were authored independently of the
Rust writer to pin actual field codes and catch mutually consistent codec errors.

Valid cases cover a minimal complete drawing, XY and signed-zero line points,
incidental root-map order and unused high watermarks. Invalid cases cover
unknown/duplicate/missing fields, wrong physical types, logical references,
version mismatch, forbidden tokens, truncation and excessive length claims.
Broader typed content is also exercised by encoding all positive current IFCX
candidate cases and by rich-content/preservation/conversion tests.

See [the physical contract](../../../schemas/ifccad/cbor-mapping-2.md).
This does not modify a numbered conformance collection, publish a supported
version, or provide a binary serialization of arbitrary foreign IFCX nodes.
