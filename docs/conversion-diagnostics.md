# Diagnosing CAD conversion

`ifccad-convert` provides a report-only diagnosis pass over its production
CAD-to-IFCCAD conversion route. It borrows the original `CadDocument`, applies no
exclusions and exposes no partial native document. Ordinary conversion retains
its fail-fast hard-error behavior and unchanged tolerances.

## Native command

```text
cargo run -p ifccad-convert --bin diagnose_cad -- SOURCE.dxf FRESH_REPORT_DIRECTORY
cargo run -p ifccad-convert --bin diagnose_cad -- SOURCE.dwg FRESH_REPORT_DIRECTORY
```

The parent directory must exist; the report directory must be new. The command
has no exclusion or benchmark options. It reads the source once through the
production DXF/DWG reader, uses the default coordinate-domain geometry tolerance
(1e-9), default Hatch creation tolerance, semantic `Allow`, and explicitly enabled
`SupportedTyped` spline preservation. It performs no encoding or CAD exchange,
size/timing comparison, repeated measurement or automatic source repair.

Outputs:

- `provenance.json`: input SHA-256, byte length, reader, command and options.
- `progress.jsonl`: flushed events as checks complete, including run start and
  finish. An interrupted run has no successful terminal event; completed checks
  remain available. This is a journal, not an exhaustive report after interruption.
- `report.json`: complete individual evidence, source handles/kinds/layers,
  owner/block/layout context, numerical intervals and tolerances, Hatch loop/edge
  locations, original Display/Debug messages and cause chains.
- `summary.md`: grouped hard failures, entity preparation coverage, diagnosed
  losses/recoveries and blocked checks. Diagnostic counts are not lost-entity counts.

Exit 0 means logical conversion is ready under the selected Allow policy. Exit 1
means diagnosis completed with hard errors or blocked conversion. Exit 2 means
usage, input reader or report IO failure. Reader failures receive an explicit
failed report when the destination is writable. An existing destination is
never overwritten, including an interrupted run. IO failure can prevent final
report creation; retain the journal and process exit status together.

Keep the input file identity and report together with an external build-input
manifest: source base and dirty/untracked content, lock/configuration, selected
codec revision and patch/content identity, toolchain and relevant environment.
The input hash alone does not identify the converter executable. Reports and
local verification material belong outside `target/`, e.g. a task-specific
ignored `.superpowers/diagnoses/` directory; they are not benchmark evidence.

## API and check boundaries

Use `diagnose_cad_document(source, metadata, options)` for an in-memory report or
`diagnose_cad_document_with_progress` with a fallible event observer. Observer IO
failure immediately stops traversal and is returned to the caller.

The pass uses ordinary source inspection and shared preparation/mapping routines.
Known entity scalar validation is collected individually with the same production
validator, including invalid entities that would otherwise be omitted. After
each independent drawable preparation failure, geometry registrations and
aggregate assessment are rolled back. Failed entities receive no native mapping.
No source entity or owner-membership entry is removed. Other eligible entities
and owners are still examined. Each entity preparation reports its first hard
failure; later checks requiring that preparation remain unevaluated.

Prepared block roots are checked independently with the production occurrence
proof. Original source membership identifies failed/unvisited descendants,
including through nested definitions; their referring roots are marked blocked.
Other roots remain assessable. Passing local preparation alone is insufficient
for occurrence accuracy. Unsupported content remains omitted under Allow, with
loss diagnostics; it is never certified as exact geometry.

Shared prerequisites can still stop the pipeline: initial structure, styles,
layout/tolerance setup, viewport/reference binding or preservation capture.
Unvisited entities remain blocked, and remaining checks are explicitly blocked.
A completed eligible traversal can include omitted external/dynamic definitions
and non-drawable structural/overall-canvas state. Source-context checks apply to
that state; it does not receive drawable geometry proof.

With any entity/occurrence hard failure, whole-drawing relation, preservation,
workspace, bounds and strict logical validation are blocked. Diagnostics gathered
from partial preparation may include dependent losses; they do not describe a
successful converted document. Otherwise the normal finalization and strict
logical validator run. With Reject, diagnosed semantic loss also fails readiness.

Neither readiness nor a passing local check claims all original source semantics,
fonts, evaluated Hatch fill or complete Dimension graphics are represented.
Production encoding/readback and actual DXF/DWG exchange remain separate checks.
No format/schema, tolerance, preservation scope or roadmap milestone is changed.

The [test.dxf Dimension investigation](dimension/reference-elevations-investigation-2026-10-20.md)
explains the 267 noncoplanar-reference failures, the implemented native XYZ
reference/XY dimension-line contract, and the complete original-file recheck.
Other source failures and explicitly diagnosed conversion losses remain visible.
