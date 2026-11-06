# Embedded host inputs

`ocs-source.json` selects Studio81bbb05e and the exact public Document, codec,
native and kernel revisions. `Cargo.lock` freezes that portable dependency graph.
The thickness codec fix remains in the explicitly named user fork.

`block-attributes.patch` applies the bounded IFCX opening, active MText edit,
Map3D enum compatibility and main/worker protocol6 changes. The preparation tool
accepts a clean application or an already applied patch and rejects source drift.
It also aliases both kernel codec Git URLs to the one selected codec model.

`block_attributes_smoke.rs` and `block-attributes.ifcx` are regression-test sources
for opening, editing, committed grips, undo/redo, save/reopen and existing ATTSYNC.
The fixture preserves the accepted gallery's Paper setup and exact IDs used by
the tests; it is intentionally independent of later public example edits.
They are copied into the pinned host with the test module enabled only under
`cfg(test)`. These are source fixtures, not packaged manual-review deliverables.
The frozen local review files and their provenance remain outside Git.

Preparation and bundle provenance hash every file here. Local path overrides
are not part of the released pins or lockfile. A source patch is distinct from an
upstream Studio contribution; it must not be attributed to unmodified upstream.
