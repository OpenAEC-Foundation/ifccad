# Binary IFCCAD roundtrip qualification

`roundtrip_cad` exercises actual file boundaries:

```text
DWG/DXF reader → typed IFCCAD → base-CBOR file → production CBOR reader
              → CAD writer → DWG/DXF file → CAD reader → typed IFCCAD
```

The binary stores the existing language-neutral IFCCAD drawing model. It does
not embed the source CAD file, enable opaque preservation, or skip invalid
entities. An inspectable `.ifcx` projection is written alongside it. This uses
the repository's experimental base-CBOR revision 1; it does not define a general
binary IFCX graph standard, change schema semantics or publish a stable format.

## Run one drawing

Select the [documented codec configuration](../patches/opencadcodec-viewports/README.md), then:

```sh
cargo build -p ifccad-convert --bin roundtrip_cad
mkdir -p .superpowers/roundtrips
target/debug/roundtrip_cad source.dwg .superpowers/roundtrips/example-01
```

The parent must exist and the output directory must be new. The input is never
overwritten. The source DWG/DXF version is retained; this command does not perform
cross-version conversion. It emits these files as far as each phase succeeds:

| File | Meaning |
| --- | --- |
| `report.json` | Status, input/output hashes and versions, reader evidence, conversion losses, comparison and any failure |
| `drawing.ifccad.cbor` | Validated native binary drawing after import |
| `drawing.ifcx` | Inspectable IFCX representation of that binary drawing |
| `roundtrip.dwg` or `roundtrip.dxf` | Candidate CAD output written from the persisted binary drawing |
| `readback.ifccad.cbor`, `readback.ifcx` | Validated native projection of the re-opened candidate CAD file |

Artifacts are **candidates**, including when the report fails qualification.
They may omit source information explicitly listed in the report. Never infer
success from a file's existence. A killed process leaves an `incomplete` report.
The final report is replaced atomically. Existing output directories are refused.

## Qualification boundary

Exit **0** (`qualified-native-roundtrip`) requires all of:

- Both CAD readers complete without recovery, skipped records, read diagnostics,
  dropped notifications or unclassified notifications.
- Import, export and reimport have no diagnosed loss, including rounding. All
  three use exact geometry tolerance. Allow is used to collect semantic loss
  evidence; it never qualifies a run with omissions or modifications.
- The persisted binary passes the production reader and strict validation.
- The final native projection encodes to exactly the original canonical CBOR
  bytes, including identities, reference ordering and numeric bits.
- The actual CAD readback version matches the source version.

The pinned codec labels some successful header/page-map progress messages as
warnings. Only the narrowly matched templates in `informational_notice` are
informational; all original notifications remain in the report. Changes in codec
messages require review rather than a broader warning exemption.

Exit **1** means not qualified or a conversion/encoding/readback failure. Exit
**2** means invalid invocation, input reading or filesystem failure. First hard
failures stop that drawing's pipeline. Use [diagnose_cad](conversion-diagnostics.md)
to collect multiple independent preparation failures.

This is deliberately a **bounded native-projection check**, not universal DWG/DXF
losslessness certification. Information not exposed by the pinned codec cannot
be assessed here. CAD rendering, fonts, evaluated Hatch fills, complete Dimension
display, external-reader compatibility and arbitrary edits remain unassessed.
The controlled integration test additionally demonstrates editing native line
geometry, recomputing bounds, re-encoding binary and exporting both CAD formats.

IDs and ordering are not normalized. A harmless allocation/order change can
therefore prevent qualification. Such a mismatch is not automatically proof of
lost geometry. The report includes up to 100 JSON-pointer differences; the
complete before/after projections remain available. CBOR differences can also
identify numeric-bit differences not visible in JSON comparison. No generated
field is discarded and no tolerance is widened to manufacture a passing result.

## Run the public corpus

The [source manifest](../tests/corpus/public-dwg-v1.json) pins 42 public DWGs to
Git revisions and SHA-256 hashes. It covers several file generations, including
dynamic-block, AEC, material and leader examples. It is not every complex drawing
on the Internet, nor a representative performance benchmark. Source files are
downloaded separately and are not bundled or relicensed as IFCCAD assets.

```sh
python3 scripts/cad_corpus.py fetch --cache .superpowers/cad-corpus
python3 scripts/cad_corpus.py run --cache .superpowers/cad-corpus \
  --binary target/debug/roundtrip_cad --output .superpowers/roundtrips/corpus-01
```

`run` is offline and verifies every cached input before executing anything. Each
drawing runs in a separate process with a default 120-second timeout (override
with `--timeout`). A failed/crashed/timed-out drawing is recorded and the batch
continues. `summary.json` is updated after every file and remains incomplete if
the batch is interrupted. It retains the manifest, executable hash and every
result. A timeout is unqualified, not a performance measurement. Missing or
invalid child reports and source-hash mismatches cannot count as passes.

Run only on explicit request; corpus downloads and qualification are not CI
benchmarks. Offline runner regression tests run in CI. Retain the source commit,
working-tree manifest, codec revision/patch, Cargo.lock and toolchain identity
with local results; an executable hash is not a replacement for build provenance.

```sh
python3 -m unittest discover -s scripts/tests -p test_cad_corpus.py
cargo test -p ifccad-convert --bin roundtrip_cad --test roundtrip_cli
```
