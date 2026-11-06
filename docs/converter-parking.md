# Direct converter archive and active exchange boundary

Following the Explorer's migration to shared `opencaddocument` conversion, the
ordinary workspace contains `ifccad`, `viewer` and `browser`. The core retains
native construction, semantic validation, source inspection/composition,
fragments, JSON/CBOR/Zstd and neutral kernels. Explorer retains the shared
Document route and native downloads. No schemas, encoding identifiers, native
profile or numbered conformance files change in this archival task.

`ifccad-convert`, `cad-preservation`, `cad-workspace-convert`,
`cad-geometry-convert`, `cad-presentation-convert`, `cad-text` and `cad-dimension`
are historical reference source under [the converter archive](../legacy/ifccad-convert/README.md).
That page records the individual dependency audit, source/test/fixture inventory,
complete fixed checkpoint, codec patch, lock/config identity and reconstruction
commands. No independent active caller remained for any of the six adapters.
Runtime-independent native kernels remain in the core; no adapter was kept active
merely for possible future use.

The archive is a partial source reference, like the OCDraw archive, not a second
product workspace. Manifests have `.snapshot` suffixes. Building requires restoring
the complete recorded checkpoint, so old relative fixtures, native APIs and
dependency selection remain coherent. Current main never supplies its core to
the archived converter. Maintaining compilation against future profiles or restoring
the old Explorer integration is outside ordinary development.

## Codec repository location after 4 November 2026

The former `HakanSeven12/opencadcodec` repository, later named
`HakanSeven12/acadrust`, was merged into
[hakanaktt/acadrust](https://github.com/hakanaktt/acadrust) and the fork is being
removed; see [IFCCAD #30](https://github.com/OpenAEC-Foundation/ifccad/issues/30).
The recorded codec commits are available there with the same hashes, including
the [reference-converter base](https://github.com/hakanaktt/acadrust/commit/bd10efc2c54dfe1e6288c5f10486dce9be0d7f32)
and the [OCDraw base](https://github.com/hakanaktt/acadrust/commit/325fa98d2b881f48e106a830d5cf161624f63131).

Frozen manifests, locks, configurations, source snapshots and measurement
provenance retain their original URLs and pins. For an explicitly requested
historical reconstruction, the new repository is an alternate location for the
same source revision, not a dependency upgrade. No archived build has been
requalified after the move, and keeping either converter buildable is outside
ordinary maintenance.

Historical commit and comparison links can use the new repository with unchanged
hashes. Issue and PR numbers do not transfer that way: open fork issues were
copied under new numbers, while closed issue/PR links may no longer resolve.
The current field-classification request is
[acadrust #305](https://github.com/hakanaktt/acadrust/issues/305), formerly #50;
the preservation discussion is retained in
[acadrust #307](https://github.com/hakanaktt/acadrust/issues/307), formerly #91.
Historical reports and frozen references retain their original identities.

Active dependency adoption remains a separate, audited change. This relocation
does not change the viewer/browser pins, the annotation development patch or its
source key. Review that key against the selected Document dependency when next
updating the pair; the current lock resolves the codec from `BjornFidder/opencadcodec`.

## Coverage and tools

The [active mapping coverage](architecture/explorer-document-route.md) is the
authority for Explorer/CLI exchange. Historical converter numerical certificates,
source-detail audits, Allow/Reject policy, partial opening, opaque spline
restoration, Hatch version matrix and codec repair proofs qualify their recorded
route only. Native opaque storage and strict readback remain active independently.
Topic documents distinguish their native sections from reference CAD exchange.

The old CBOR measurement driver and codec CI action/config/patches moved into the
source archive. Run them only from a complete applicable historical checkpoint,
and measurements only for an explicitly requested scope. Compression probes that
operate on a supplied, hash-identified artifact remain independent manual tools.
Existing benchmark reports/results, accepted provenance and numbered conformance
remain byte-for-byte at their original paths. Historical paths in those reports
refer to their recorded checkpoint; they are not instructions for active main.

Current CI fetches only active dependencies and no longer prepares the historical
codec. Its browser and Rust cache namespaces/inputs follow the active sources.
The active workspace guard rejects archived packages, transitive legacy paths,
targets, Rust APIs and fixture references. The Explorer-specific negative guard
continues rejecting the old route. Native release packaging keeps defensive
exclusions for all CAD/application and legacy sources/dependencies; this archive
does not expand the published native source scope or change an existing release.

Open issues #4 (native splines/preservation), #12/#18 (reference accuracy), #6
(encoding evidence) and #20 (future evaluation API) overlap historical evidence
and future design. They do not authorize parity work, resurrecting the reference
route or automatic measurements. Future work follows the current roadmap and
qualifies the shared mapping explicitly.
