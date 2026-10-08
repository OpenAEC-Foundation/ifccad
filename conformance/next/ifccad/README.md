# IFCCAD candidate conformance

This independent experimental collection exercises the active IFCCAD profile,
not the standalone OCDraw contract or released numbered IFCCAD collections.
`cases.json` lists filenames and expected validity. The production reader uses
strict profile validation; valid cases also undergo production encoding and
strict readback in `tests/ifccad_conformance_next.rs`.

The valid examples contain an analytic circular clip and a concave closed straight
polyline clip. Invalid cases cover a zero perspective lens, a missing enabled
boundary, incomplete circle enclosure, an unknown viewport field and a duplicate
frozen-layer reference. Diagnostic wording is not the language-neutral contract.
Use the active schema and experimental contract as the authoritative requirements.

The provisional 0.1.0 geometry expansion additionally covers all native primitive
families, optional conservative Model/definition bounds through nested blocks,
full-ellipse and two-semicircle bulged clips, and forward boundary references.
Negative cases cover bulge cardinality/coincident curved endpoints, zero/full
arc sweeps, unordered/undersized bounds, unknown primitive fields, forbidden
spatial placement, off-plane clips and whole-curve escape despite fitting vertices.
These cases live only in `next`; no numbered collection is revised.

## Layout output revision

Both models retain layout media without complete plot settings. Plot unit and
fixed mapping determine Paper output meaning; IFCCAD no longer stores an independent
Paper coordinate unit. Effective plot settings, limits and layout PSLTSCALE have
separate native/CAD coverage. The provisional field/API migration, strict physical
scalar conversion limits, raster restrictions and per-domain accuracy reports are
specified in [layout output](../../../docs/layout-output.md). No new workspace state, renderer, release or controlled measurement is implied.

Workspace candidates cover saved/current separation, unknown choices, disabled zero snap spacing, independent canvas frames and malformed references/scalars. IFCCAD also covers full-width UCS/window identities with explicit independent watermarks.
