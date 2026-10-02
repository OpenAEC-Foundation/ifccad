# Common-subset transport experiment

This repeatable experiment compares native OCDraw, IFCX-CAD, DXF and DWG
representations of exactly the same retained content, plus external gzip.
It is separate from the accepted historical and standalone OCDraw experiments
under `benchmarks/size/`.

Measurements run only on explicit user request, for the full experiment or the
requested selection. They are never automatic completion gates or CI tasks, even
when main, converters, writers, codecs or dependencies change. Focused correctness
and strict-readback tests remain part of normal verification. Retained reports
state the tested revision and coverage; they do not automatically certify later
revisions. The current runner executes the complete fixed corpus plus optional
practice input. A narrower requested scope needs an explicit recipe/runner selection
before execution; omitting practice still runs all nine synthetic recipes.

Before an accepted run, prepare the pinned dependencies and local Cargo.lock:

```text
cargo build --offline -p ocdraw-viewer --example size_exchange --release
```

This one-time build also updates a stale local lockfile before provenance is
captured. The measurement rejects source or lock changes during its two passes.
Use Python 3.11+ and run from the repository root:

```text
python scripts/size_exchange.py --run NEW_RUN_NAME --practice PATH_TO_ORIGINAL_DWG_OR_DXF
```

Use `--release` for optimized execution on larger drawings. The practice reader
is selected by the case-insensitive `.dwg` or `.dxf` extension; both fresh passes
start from the original source. DXF practice uses the `practice-dxf` case ID;
the existing DWG practice case ID remains `practice-foundation`.

Omit `--practice` for the synthetic corpus only. Cargo runs offline: fetch the
normal pinned dependencies first if needed. `CARGO_TARGET_DIR` may select an
existing build cache. Runs go under `target/size-exchange/NEW_RUN_NAME/` and
never overwrite an existing directory. Both passes regenerate from the original
input. Compression probes are external transport variants, not format encodings.

- `corpus-v1.json` defines nine deterministic recipes, all in millimetres.
- `results-v1.json` retains the accepted detailed run: raw/compact/pretty/gzip
  sizes and hashes, semantic snapshots' hashes, strict readback evidence,
  initial loss diagnostics, normalization changes and source provenance.
- `test-dxf-results-v1.json` retains the independent large-DXF run with matching
  [report](../../docs/benchmarks/common-subset-test-dxf-v1.md).
- [Report](../../docs/benchmarks/common-subset-size-exchange-v1.md) explains
  the comparison contract, practice subset and limitations.

Recipes: empty drawing; 100 and 10,000 lines; 1,000 lines with binary fractions;
100 and 10,000 four-vertex straight polylines (alternating open/closed);
1,000 circles; three nested/shared block occurrences; and 128 entities with
named dashed/dotted patterns, scales, and an unused pattern definition.
All use two named layers and mixed explicit/ByLayer/ByBlock appearance.
The block case has nonzero definition base points, nonuniform/mirrored scales,
two nesting levels, repeated targets and an unused block definition.

The final equality check includes unused definitions, names, references,
draw order, units, placement and appearance. It excludes allocated identities
and bookkeeping. Native output uses core writers; every artifact is loaded
through its production reader. IFCX uses its source-aware Reject conversion,
so foreign graph content cannot be hidden by taking the CAD projection.

Future runs must use fresh names and start from the original practice source.
Compare retained inventories and semantic hashes before attributing size
changes to encoding improvements. Add representative recipes when new entity
families become supported; this corpus cannot establish their size behavior.
Retain the last accepted report and matching JSON until an applicable successful
run replaces them. Failed cases remain in their run directory and never enter
the accepted comparison table.
