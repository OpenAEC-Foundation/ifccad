# Codec checkout configuration

Current upstream base: `bd10efc2c54dfe1e6288c5f10486dce9be0d7f32` (package `acadrust` 0.6.6, local dependency alias `opencadcodec`), audited on 2026-10-23. The exact upstream fix for [Hatch DXF angles #132](https://github.com/HakanSeven12/opencadcodec/issues/132) is selected without local source patches. Dimension text-position polarity #117 and earlier viewport repairs are upstream.

```text
git clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach bd10efc2c54dfe1e6288c5f10486dce9be0d7f32
cargo test --config patches/opencadcodec-viewports.toml --workspace
```

For persistent local selection, copy `patches/opencadcodec-viewports.toml` to ignored `.cargo/config.toml`, preserving local build configuration. The shared `.github/actions/viewport-codec` action selects the same unmodified base; its historical action/configuration names remain stable. Historical viewport and Dimension patch files are provenance only and must not be applied to this revision.

See the [current public-model audit](../../docs/geometry/opencadcodec-update-2026-10-23.md). Changing this pin does not authorize a size/exchange measurement.
