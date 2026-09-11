# Remaining fixes on the current cadcodec pin

The normal dependency is upstream acadrust 0.5.4 at
`2f2cd25832db298524fb5eb36ced5a438a877e95`. It includes deterministic DXF object
ordering, so no DXF implementation patch is needed.

Two defects remain upstream:

- [#41](https://github.com/HakanSeven12/cadcodec/issues/41): DWG current
  lineweight must encode/decode the table index, not the raw hundredths value.
- [#42](https://github.com/HakanSeven12/cadcodec/issues/42): read/write the
  independent MEASUREMENT field in AcDb:Template.

The three patches here rebase those fixes and their regression tests onto the
new pin. The tests also verify upstream's DXF ordering. The local override is
explicit; normal Cargo commands use the unmodified upstream commit.

From the IFCCAD repository root, using a new checkout directory:

```text
git clone https://github.com/HakanSeven12/cadcodec.git target/cadcodec-upstream-fixes
git -C target/cadcodec-upstream-fixes switch --detach 2f2cd25832db298524fb5eb36ced5a438a877e95
git -C target/cadcodec-upstream-fixes apply --check ../../patches/cadcodec-upstream/0001-dwg-header-lineweight.patch ../../patches/cadcodec-upstream/0002-dwg-measurement.patch ../../patches/cadcodec-upstream/0003-regression-tests.patch
git -C target/cadcodec-upstream-fixes apply ../../patches/cadcodec-upstream/0001-dwg-header-lineweight.patch ../../patches/cadcodec-upstream/0002-dwg-measurement.patch ../../patches/cadcodec-upstream/0003-regression-tests.patch
cargo test --manifest-path patches/cadcodec-upstream/harness/Cargo.toml --target-dir target/cadcodec-regression-build
cargo test --config patches/cadcodec-upstream.toml --workspace
cargo clippy --config patches/cadcodec-upstream.toml --workspace --all-targets -- -D warnings
cargo run --config patches/cadcodec-upstream.toml -p ifccad-convert --example size_baseline -- --run upstream-fixed-review --cargo-config patches/cadcodec-upstream.toml
```

Inspect an existing checkout before using it; do not overwrite existing work.
Choose a fresh benchmark run name. Run commands using different dependency
configs sequentially: they share the root Cargo.lock, and locked provenance
deliberately rejects a mismatch. Candidate results record the actual patched
source hash and remain separate from accepted upstream baselines.
