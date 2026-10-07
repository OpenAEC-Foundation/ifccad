# Viewport codec configuration

Pinned upstream base: `063c10671fe7833d562f772159771318c7a0ebb9` (opencadcodec 0.6.0), audited on 2026-10-07.

PR [#88](https://github.com/HakanSeven12/opencadcodec/pull/88) and [#89](https://github.com/HakanSeven12/opencadcodec/pull/89) are merged into this base. Clipping activation/group 340 and DXF degree/radian conversion now come from upstream. Files 0001 and 0002 remain historical provenance for the previous pin; **do not apply them to the new base**.

`0003-viewport-off-state.patch` is the only selected repair. It retains the independent 0x20000 off bit and its effect on DXF group 68. It has not been merged upstream. The patch is rebased against this exact upstream revision; this local repair is not evidence for the unmodified dependency. The two cores remain independent of the CAD codec.

Create an independent checkout; preserve unrelated existing local dependency edits and do not modify the Cargo source cache:

```text
git clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach 063c10671fe7833d562f772159771318c7a0ebb9
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
cargo test --config patches/opencadcodec-viewports.toml --workspace
```

For persistent opt-in, copy the contents of `patches/opencadcodec-viewports.toml` to the ignored `.cargo/config.toml`, preserving existing local configuration as needed. CI's `.github/actions/viewport-codec` selects the same base and only patch 0003. Without that patch, native clipping and angle mappings are now upstream capabilities, while IFCCAD's off-state capability gate still diagnoses unsupported authored viewport conversion.

The new [dependency audit](../../docs/geometry/opencadcodec-update-2026-10-07.md) records public-model changes and preservation compatibility. The separate spline DXF fix is submitted as [PR #99](https://github.com/HakanSeven12/opencadcodec/pull/99); it is not included in this base. Correct fit-only spline DXF export therefore remains a known target-codec limitation until that repair is adopted in both converter and viewer builds.

Existing literal viewport fixtures and physical DXF/DWG tests retain their scope. The passing DWG clip profile includes a conventional overall paper canvas. Neither this configuration nor the pin update authorizes any size/exchange measurement. Historical reports continue to describe their recorded revisions/configurations.