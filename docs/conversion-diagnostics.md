# Diagnosing CAD conversion

Historical reference tooling: the commands below run only after restoring the
[complete converter checkpoint](../legacy/ifccad-convert/README.md#reconstruct-and-test).
They are not active-workspace commands or shared-route guarantees.

For explicitly authorized partial import, see `CadImportErrorPolicy::SkipInvalidEntities`
in the [converter contract](../legacy/ifccad-convert/reference/crates/ifccad-convert/README.md#conversion-diagnosis).
It produces a strictly validated retained drawing with a separate omission report.
This report-only diagnosis API keeps collecting hard failures without output, even
when supplied partial-import options; it never disguises a failed source check as
a successful full conversion. Partial import does not change proof tolerances.

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

## Layout reference recovery

The converter resolves `ACAD_LAYOUT` through the root named dictionary and
checks dictionary identity, exact membership, ownership and layout/block
relationships. A stale derived `header.acad_layout_dict_handle` receives a
located Recovery diagnostic only when one unambiguous, structurally consistent
named relationship establishes the role. A conflicting existing dictionary,
duplicate membership or contradictory non-null owner remains an error. A missing
owner is recoverable only from one exact dictionary membership. The original
`CadDocument` is unchanged; residual comparison uses the resolved role.

Model VPORT table references and Paper VIEWPORT entity references are checked
against their distinct roles. Paper references must resolve in their own owner;
unsupported Model selection remains diagnosed rather than becoming invented
native state. The exact current missing-reference and loss boundaries are in
[layout-reference tests](../legacy/ifccad-convert/reference/crates/ifccad-convert/tests/layout_references.rs),
the [source resolver](../legacy/ifccad-convert/reference/crates/ifccad-convert/src/source/layouts.rs) and
[workspace coverage](workspace-state.md). This recovery is not geometric repair
and does not relax either semantic loss policy or hard numeric limits.

### Historical five-file qualification (2026-10-05)

The following original inputs converted under Allow and passed production
strict native readback at the correction checkpoint. Counts describe that
supported-subset conversion, not complete source or rendering fidelity.

| Original | SHA-256 | Model entities | Local definitions | Paper layouts |
| --- | --- | ---: | ---: | ---: |
| 3bm funderingsherstel DXF | `054317ae053be20d9c0f9c8650b5070f76826a7eede9690cd9c572700ea6149c` | 1070 | 127 | 2 |
| 3bm funderingsherstel DWG | `24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5` | 1070 | 127 | 2 |
| sample_AC1032.dwg | `0e8faaca949c9429c92240082d9ea4d3524aca5baf7c7815ec439245c321b528` | 54 | 7 | 3 |
| sample_AC1032_ascii.dxf | `c2934f616ecd7097e773b85e5d6e0548d21ccc635abcc8e7276f7d4a585a7ae2` | 54 | 7 | 3 |
| sample_AC1032_binary.dxf | `e2c59abecc7bc0fd0ace7b9b6f083d2a16d36077b7a7d7ceccd2aa7edfb348e1` | 54 | 7 | 3 |

The exact 3bm basename was `2705_model Funderingsherstel - Constructie - Sheet -
CP-21 - Constructietekening`. The codec was
`fe69506cb99dea6f4c4a73b690a27fdf04403ea0` with the existing three-patch
[viewport development configuration](../legacy/ifccad-convert/reference/patches/opencadcodec-viewports/README.md).
Those repaired results did not qualify the unmodified dependency. No dependency
change or size/performance measurement was part of the correction.

On main basis `c26ce87`, the adapted correction recorded formatting, strict
workspace Clippy and workspace tests passing: 496 passed, one existing ignored.
Seven regression groups included relocated-dictionary DXF, Model-VPORT DWG,
logical/encoded agreement, source immutability and invalid-reference policies;
four positive cases had failed with the original structural errors before the
repair. Review was by the author. All five files also passed the host-side
production viewer adapter; no new browser WASM build or deployment was performed.
These are historical results, not newly executed verification or current
full-file coverage claims.


The [coincident-polyline follow-up](geometry/coincident-polyline.md) checks the
same complete test.dxf after retaining source-coincident planar segments:
all 154 previous LWPOLYLINE failures are resolved and the 267 Dimension fixes
still pass. The same 89 Hatch failures remain; the whole drawing is still
blocked. No exclusions, benchmark or tolerance change was used.
