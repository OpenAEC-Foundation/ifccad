# IFCCAD candidate conformance

This independent experimental collection exercises the active IFCCAD profile,
not the standalone OCDraw contract or released numbered IFCCAD collections.
`cases.json` lists filenames and expected validity. The production reader uses
strict profile validation; valid cases also undergo production encoding and
strict readback in `tests/ifcx_conformance_next.rs`.

The valid examples contain an analytic circular clip and a concave closed straight
polyline clip. Invalid cases cover a zero perspective lens, a missing enabled
boundary, incomplete circle enclosure, an unknown viewport field and a duplicate
frozen-layer reference. Diagnostic wording is not the language-neutral contract.
Use the active schema and experimental contract as the authoritative requirements.
