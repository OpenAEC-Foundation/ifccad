# IFCCAD alpha releases and file versions

The [alpha compatibility policy](alpha-compatibility.md) applies to every
release. A public application integration does not promise that files saved
today will open weeks later. Old readers and automatic or manual migrations
are optional. Reassess this when the format is more complete and mature.

## Software numbering

Use `YYYY.W.N`: ISO week-year, ISO week without a leading zero, and sequence
within that week starting at 0. The week-year can differ from the calendar year
around New Year. Tags are `vYYYY.W.N`. All active workspace packages inherit
`workspace.package.version`; the private Explorer and release manifest carry
the same version. This implements the scheme in
[issue #19](https://github.com/OpenAEC-Foundation/ifccad/issues/19).

The date-shaped number is accepted by Cargo/npm version syntax, but conveys
no SemVer compatibility promise. Consumers should pin an exact release or Git
revision instead of inferring API/file compatibility from a version range.
GitHub releases must be marked **prerelease** and titled **IFCCAD YYYY.W.N alpha**.
No stable 1.0 status is implied by the year in the software version.

`2026.44.0` is the first native source alpha (27 October 2026, ISO week 44).
The initial inspection on 26 October found no GitHub tags/releases or crates.io
`ifccad` crate; no crates.io publication is part of this release. Before each
publication, check remote tags/releases and any intended registry, then choose
that week's next unused sequence. Never reuse a published number or move a
published tag.

## File identification and contract changes

Software and file identifiers are independent. The first published alpha uses:

| Layer | Identifier | Meaning |
| --- | --- | --- |
| Software | `2026.44.0` | Exact implementation/source release; alpha |
| IFCX syntax | `header.ifcxVersion = ifcx_alpha` | Upstream syntax, not an IFCCAD release number |
| IFCCAD JSON profile | `profileVersion = 0.1.0` | Current logical profile, import `urn:example:ifccad:0.1.0` |
| Standalone CBOR | Envelope mapping revision `1` | IFCCAD projection, not general binary IFCX |
| CBOR + Zstd | Transport revision `1`, inner mapping `1` | One checksummed Zstd frame around CBOR; no ZIP |
| Dataset | `header.dataVersion` | Caller-owned dataset revision; never the software release number |

The experimental `example` import remains temporary, resolved offline by this
reader. This alpha publication does not make it a production IFCX namespace.
The release manifest and source hashes identify the exact bundled schemas,
mapping, fixtures and implementation. Zstd's standard frame has no private
transport-revision field; its inner CBOR envelope is the on-wire IFCCAD identity.

Before their first publication in 2026.44.0, these drafts could change in place
without increments. Publication freezes that release's artifacts and contract,
not the working tree. For development after a published contract:

- A normative logical profile change, including new fields/defaults or changed
  validity/meaning, advances `0.N.0` to `0.(N+1).0` before landing. Update the
  profile module/import and all active references together. These counters do
  not claim SemVer compatibility. Pure explanation or implementation fixes that
  restore the unchanged contract need no file-version change.
- A normative CBOR change, including a changed embedded logical profile, advances
  its integer mapping revision. Update the single active mapping, codec and
  `conformance/next` together. Historical mappings/readers need not remain in
  the active tree; the previous release tag retains its exact sources.
- A changed Zstd framing/acceptance contract advances its documented transport
  revision **and the inner CBOR mapping revision**, so changed acceptance is
  identifiable on wire. Compression-level tuning and dependency upgrades that
  leave the transport contract unchanged do not advance it. JSON profile need
  not advance for a physical-only change.
- Several unpublished edits can share one next draft revision. Before publishing,
  compare it with the last published contract and list all known breaks.

Readers may reject unsupported identifiers; no fallback reader or migration is
required. A software/API fix alone advances only the software number. A new
feature can preserve existing files, but this creates no general compatibility
guarantee. Numbered released conformance collections remain immutable. Develop
in `conformance/next`; tagged alpha snapshots identify this evolving corpus
without claiming that historical `conformance/1.0.0` is an IFCCAD stable release.

## First release scope and reproduction

The first deliverable is a **native core source alpha**: only `ifccad`, its
schemas, core tests, examples and release documentation. `ifccad-convert`, all
CAD companions, acadrust/opencadcodec, opencaddocument, viewer/browser and Explorer
are excluded. The core has registry-only production dependencies. Workspace
companions remain `publish = false`; synchronized repository software versions
do not imply inclusion in this release. No Studio/Explorer deployment is included.

`python scripts/prepare_release.py package --staging FRESH_DIRECTORY` produces a
standalone core `.crate` with a normalized Cargo manifest and core-only lockfile.
It first resolves the existing workspace unchanged, then packages/verifies the
isolated native sources without the repository's codec configuration. Removing
unused workspace patch metadata must not change any resolved registry dependency.
Use a short, fresh staging directory outside Git for the cleanest package provenance.
The verified result is `target/package/ifccad-YYYY.W.N-native.crate`.
The preparation helper checks that package
for CAD/application sources and dependencies before deriving the source archive.
The archive contains no workspace `.cargo/config.toml`, local codec checkout or
CAD runtime. `SOURCE-MANIFEST.json` records its scope, package checksum, every
source byte hash, base commit, candidate status and format identifiers. Rust
`1.98.0` is selected by the included toolchain file. Archive member order, times,
ownership, modes and gzip header are fixed. Equal source bytes produce equal
archive bytes. This is source/build-input reproduction, not a promise of identical
compiled binaries across platforms or compilers. Registry dependencies are fixed
by lockfile checksums; this is not an offline vendor bundle.

After extracting the native source archive, use the standalone core directory:

```text
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo clippy --all-targets --locked --features ifccad-cbor-zstd -- -D warnings
cargo test --locked --features ifccad-cbor-zstd --test ifccad_cbor_zstd
```

Native Zstd needs the platform's ordinary C build toolchain. Core tests use the
production readers and strict validation. No CAD codec is required for these
commands. A `.crate` can also be extracted directly as a gzip-compressed tar.

The **repository integration gate** still covers the whole workspace under
AGENTS.md. Its codec patch is a test/conversion dependency, not a released core
dependency. For repository verification, reconstruct the codec from the
[documented base and patch](../patches/opencadcodec-polyline-widths/README.md),
select it explicitly and retain its exact content/config identity separately
from the native package's dependency record. Do not replace it with an arbitrary
warm local codec. Current standalone codec failures remain documented and are
not claimed passing by the IFCCAD workspace gate.

```text
python scripts/prepare_release.py check
python scripts/check_active_workspace.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo clippy -p ifccad --all-targets --locked --features ifccad-cbor-zstd -- -D warnings
cargo test -p ifccad --locked --features ifccad-cbor-zstd --test ifccad_cbor_zstd
python -m unittest discover -s scripts/tests -p test_prepare_release.py
npm --prefix format-explorer test
```

Frontend tests above check repository manifest consistency and regressions; they
do not qualify or publish an Explorer release. No WASM build, host deployment or
size/exchange benchmark is a gate for this native-only artifact.

## Preparation and publication checklist

1. Inventory intended main, dirty/untracked files and active tasks. Review relevant
   open issues without expanding scope. Select accepted changes explicitly.
2. Resolve the picture-task dependency check against its final source, lock and
   patch contents. Include necessary IFCCAD changes before freezing the release.
   Converter-only opencaddocument fixes do not require an IFCCAD format release.
3. Check remote tags/releases and any intended package registry. Choose the next
   free software number; run `python scripts/prepare_release.py set-version YYYY.W.N`.
   Refresh the lock with `cargo metadata --offline --format-version 1`, then run
   `python scripts/prepare_release.py check`. Update notes for that version.
4. Compare file contracts against the last published release and apply the rules
   above. Identify supported versions and known breaks in the notes. For the first
   release, no compatibility with earlier unversioned drafts is guaranteed.
5. Prepare the final candidate against intended current main. Run the full gate
   above and relevant regressions. Record source inventory, lock/config/codec
   contents, tool versions, environment, commands, exits, timings and log paths.
   Reuse only when every relevant input is equivalent. A changed candidate needs
   new verification; prose-only edits do not invalidate Rust checks.
6. Prepare and verify the core package with `python scripts/prepare_release.py
   package --staging FRESH_DIRECTORY` (local preparation, no publication).
   Inspect its file list,
   normalized manifest and native lockfile: no CAD/application package belongs
   in this artifact. Do not infer companion publishability from this.
7. Build a local review bundle with `python scripts/prepare_release.py bundle
   --candidate --output .superpowers/ifccad-YYYY.W.N-candidate.tar.gz`. Unexpected
   untracked files are refused; exact approved new paths are in the release manifest.
   Repeat to a second path and compare SHA-256; extract and verify the inventory,
   native dependency selection and production-reader checks. No upload occurs.
8. Obtain explicit authorization for the concrete commit/integration, tag, assets
   and publication destinations. No commit, push, merge, tag or publication is
   implied by preparation. Keep the candidate worktree while review is active.
9. After authorized integration, compare **all** inputs or rerun the full gate.
   Use a clean checkout for the final bundle (omit `--candidate`). Record the
   final source commit, archive checksum and verification evidence. Publish an
   immutable `vYYYY.W.N` tag, prerelease notes, source bundle and checksum only
   to explicitly authorized destinations. Never silently replace an artifact;
   correct it with a new software release.
10. After integration/retirement, audit and clean only this completed task's local
    worktree/material/caches under [the maintenance rules](local-maintenance.md).

The release helper is local preparation, not a publication workflow. It does not
check remote number availability, perform gates or grant publication approval.
