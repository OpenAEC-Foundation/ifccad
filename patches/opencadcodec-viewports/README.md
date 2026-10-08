# Viewport codec configuration

Pinned upstream base: `ab2eecdbffc31120b5ad6d899f6fc67cf21ede39` (opencadcodec 0.6.0), audited on 2026-10-08.

PR [#88](https://github.com/HakanSeven12/opencadcodec/pull/88) and [#89](https://github.com/HakanSeven12/opencadcodec/pull/89) are merged into this base. Clipping activation/group 340 and DXF degree/radian conversion now come from upstream. Files 0001 and 0002 remain historical provenance for the previous pin; **do not apply them to the new base**.

`0003-viewport-off-state.patch` is the only selected repair. It retains the independent 0x20000 off bit and its effect on DXF group 68. It is submitted upstream as [PR #103](https://github.com/HakanSeven12/opencadcodec/pull/103) and has not been merged. The repair also makes the public visibility helpers respect the independent off flag. The patch is rebased against this exact upstream revision; this local repair is not evidence for the unmodified dependency. The two cores remain independent of the CAD codec.

Create an independent checkout; preserve unrelated existing local dependency edits and do not modify the Cargo source cache:

```text
git clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach ab2eecdbffc31120b5ad6d899f6fc67cf21ede39
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
cargo test --config patches/opencadcodec-viewports.toml --workspace
```

For persistent opt-in, copy the contents of `patches/opencadcodec-viewports.toml` to the ignored `.cargo/config.toml`, preserving existing local configuration as needed. CI's `.github/actions/viewport-codec` selects the same base and only patch 0003. Without that patch, native clipping and angle mappings are now upstream capabilities, while IFCCAD's off-state capability gate still diagnoses unsupported authored viewport conversion.

The new [dependency audit](../../docs/geometry/opencadcodec-update-2026-10-07.md) records public-model changes and preservation compatibility. The spline DXF fix [PR #99](https://github.com/HakanSeven12/opencadcodec/pull/99) is included in this base, together with the DWG secondary Paper owner and overall-viewport role repairs. The 2026-10-07 audit remains historical; current qualification is recorded in workspace-state documentation and the 2026-10-08 audit.

Existing literal viewport fixtures and physical DXF/DWG tests retain their scope. The passing DWG clip profile includes a conventional overall paper canvas. Neither this configuration nor the pin update authorizes any size/exchange measurement. Historical reports continue to describe their recorded revisions/configurations.