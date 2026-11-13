# Embedded host inputs

`ocs-source.json` selects the official Studio, Document, acadrust, IFCCAD and
kernel revisions. `Cargo.lock` freezes the resolved graph. `native-tables.patch`
retains the Explorer host's IFCX opening, MText editing, annotation scale and
leader picture fixes. It pairs worker protocol 10 with the corresponding
worker asset URLs.

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

The current untagged `leader-slice` candidate selects `leaders.patch`, which
includes the earlier attribute patch, adds bounded leader-picture invalidation
for changed properties/grips and uses worker protocol7 with matching cache URLs.
Saving also materializes the host's selected annotation scale through its
existing scale-list helper before writing CANNOSCALE; an empty native scale
registry must not turn into a dangling reference after opening in the host.
It uses the explicit relative local
overrides in `ocs-source.json`. The preparation tool applies all codec URL
aliases to that one selected model and verifies the resolved graph. Its recorded
lock is specific to this development pair, not a portable upstream release pin.
`leaders_smoke.rs` is copied only for this candidate and exercises opening from
IFCX and DWG, shared Leader targets, a MLeader text edit and grip, undo/redo,
stored-picture invalidation and strict native save/reopen. Its source describes
the intended check; only a recorded successful execution qualifies that behavior.
