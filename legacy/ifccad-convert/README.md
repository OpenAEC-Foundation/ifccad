# Archived IFCCAD reference converter

`ifccad-convert` and six codec adapters are historical reference source, outside
ordinary Cargo builds, tests, CI and release packaging. Active CAD exchange uses
the shared `opencaddocument` route in viewer/browser; see
[its coverage](../../docs/architecture/explorer-document-route.md). Numerical,
preservation, Allow/Reject and physical-version proofs here qualify only this
reference. They are not fallback APIs or guarantees for the shared route.

[FREEZE.json](FREEZE.json) records complete source checkpoint
`13b5ef740b61f1e7b891b374fa8efe2e3a07cf91`, tree, archived source hashes, lock,
codec base/patch and tools. `reference/` keeps original repository-relative
source paths, tests, owned fixtures, coverage contracts, converter-only helpers,
the manual CBOR driver and historical codec preparation. Cargo manifests and the
old CI action have `.snapshot` suffixes deliberately: this partial source tree
is not a build entry point. Relative paths and source documentation are interpreted
in the restored checkpoint. Shared native helpers, core, schemas, examples and
fixtures come from that complete checkpoint, never the evolving active core.

## Reconstruct and test

From a current checkout with Python 3.12+ and Git, select a **new** destination:

```text
python legacy/ifccad-convert/restore.py --destination .superpowers/reference-converter
```

This restores the complete frozen Git tree, verifies every archived byte against
it, and installs the recorded lock and portable Cargo configuration. It does not
change the current checkout, clone dependencies or run measurements. In the new
directory, prepare the exact codec (network needed once):

```text
git -c core.longpaths=true clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -c core.longpaths=true -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach bd10efc2c54dfe1e6288c5f10486dce9be0d7f32
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-polyline-widths/0001-preserve-polyline-widths.patch
cargo +1.98.0 fetch --locked
cargo +1.98.0 test --locked -p ifccad-convert --test polyline_width_exchange --test cbor_document
```

Use a short destination on Windows; `core.longpaths` also permits long Git paths.
The portable configuration selects its own `target/` and debug=0 development/test
profiles, avoiding an ancestor checkout's build cache. The patch path above is
relative to the codec checkout. Do not apply the older
viewport/Dimension patches on this base: those repairs are upstream already.
For all reference regressions, replace the final command with:

```text
cargo +1.98.0 test --locked -p ifccad-convert -p cad-preservation -p cad-workspace-convert -p cad-geometry-convert -p cad-presentation-convert -p cad-text -p cad-dimension
```

These are optional historical tests, not active completion gates. Standalone
codec suite/lint limitations are documented in the frozen width-patch README.
No promise is made that the archived source compiles against future native
profiles, toolchains or dependencies. The restored complete tree is the supported
reproduction boundary. Existing benchmark reports retain their own earlier pins,
inputs and exclusions; restoring this checkpoint does not reproduce every report.
Measurements still need a separate explicit scope request; `test.dxf` stays excluded.
The restored directory is a source export without Git metadata. Git-dependent
measurement launchers require an actual detached checkout of the checkpoint in
their own report; do not run them in this export and accidentally inspect an
ancestor repository. The archived launcher is retained for that historical use.

## Individual companion decisions

The audit checked Cargo dependencies, Rust imports, include/path references,
examples, scripts and tests outside the archived route. None has an active
consumer after the Explorer migration.

| Crate | Reason to archive |
| --- | --- |
| `cad-preservation` | Codec-specific snapshot/capture/restore; active native preservation storage stays in core, shared-route capture is unavailable. |
| `cad-workspace-convert` | Codec-to-native workspace field adapters; neutral workspace kernel remains in core. |
| `cad-geometry-convert` | Codec geometry preparation and certified conversion proofs; neutral geometry/interval kernels remain in core. |
| `cad-presentation-convert` | Codec colors/presentation scalars; neutral color/line-pattern/plot kernels remain in core. |
| `cad-text` | Typed codec text preparation; native text model/validation/bounds remain in core. |
| `cad-dimension` | Codec dimension styles/overrides/preparation; native dimensions/graphics/validation remain in core. |

The helpers `cad_dimension_repair.rs`, `cad_hatch_associations.rs`,
`far_hatch_pattern.rs` and `hatch.rs` had only archived consumers and moved with
them. Shared native helpers and all enabled native tests remain active. No second
product route or compatibility layer is maintained. See
[the parking decision](../../docs/converter-parking.md) for current development rules.
