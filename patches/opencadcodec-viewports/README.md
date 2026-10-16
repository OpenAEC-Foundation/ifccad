# Viewport codec configuration

Current upstream base: `325fa98d2b881f48e106a830d5cf161624f63131` (package version 0.6.0), audited on 2026-10-16.

Viewport repairs #88, #89 and #103 are merged into this base. Paper grid #106 and DXF ShadePlot #107 are also included. Files 0001, 0002 and 0003 remain historical provenance; do not apply them to this base.

The only selected local repair is the [Dimension text-position patch](../opencadcodec-dimension/README.md), tracked in upstream [issue #117](https://github.com/HakanSeven12/opencadcodec/issues/117). Create an independent checkout and preserve unrelated dependency edits:

```text
git clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach 325fa98d2b881f48e106a830d5cf161624f63131
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-dimension/0001-text-position-flag.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-dimension/0001-text-position-flag.patch
cargo test --config patches/opencadcodec-viewports.toml --workspace
```

For persistent opt-in, copy `patches/opencadcodec-viewports.toml` into the ignored `.cargo/config.toml`, preserving local build configuration. The shared `.github/actions/viewport-codec` action selects the same base and patch. The historical action/configuration names remain stable. Local patched results do not describe the unmodified dependency.

See the [current public-model audit](../../docs/geometry/opencadcodec-update-2026-10-16.md) and [workspace contract](../../docs/workspace-state.md). No measurement run is authorized by changing this pin.
