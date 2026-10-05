# Explicit viewport codec development patches

Base: opencadcodec `fe69506cb99dea6f4c4a73b690a27fdf04403ea0`.
The first two patches were taken from the local `ifcx-cad-paperspace` worktree on
2026-10-05, with user authorization to adopt both in the OCDraw viewport worktree.
The converter manifests share this upstream base. These are explicit local
repairs, not evidence for the unmodified dependency or upstream acceptance.

- `0001-viewport-clipping.patch` matches the production changes in
  [PR #88](https://github.com/HakanSeven12/opencadcodec/pull/88), commit
  `ffd9959298ffb936a3b1701587d4e9e122ee69b0`. It retains status bit `0x10000`
  independently of the boundary handle and reads/writes DXF VIEWPORT group 340.
  An omitted serde activation field defaults to false.
- `0002-viewport-angle-units.patch` converts VIEWPORT groups 50/51 between DXF
  degrees and the public model's radians, and documents the public field units.
  These production changes are published independently in
  [PR #89](https://github.com/HakanSeven12/opencadcodec/pull/89), commit
  `c5ac46a33ed8b38e0be22a4853807fc8ed3d70aa`, directly based on upstream main.
- `0003-viewport-off-state.patch` additionally preserves the independent off bit
  0x20000 and reflects its effective state in DXF group 68. The IFCX-CAD slice
  requires this to distinguish view enabled from entity visibility and zoom lock.
  It is a local repair, not part of either published PR; adding it exposes a new
  serde-defaulted public boolean field to the shared dependency. OCDraw's typed
  viewport residual comparison continues diagnosing state it does not map.

Create an independent checkout; do not change the Cargo cache:

```text
git clone https://github.com/HakanSeven12/opencadcodec.git .superpowers/dependencies/opencadcodec-viewports-development
git -C .superpowers/dependencies/opencadcodec-viewports-development checkout --detach fe69506cb99dea6f4c4a73b690a27fdf04403ea0
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-viewports/0001-viewport-clipping.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-viewports/0001-viewport-clipping.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-viewports/0002-viewport-angle-units.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-viewports/0002-viewport-angle-units.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply --check ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
git -C .superpowers/dependencies/opencadcodec-viewports-development apply ../../../patches/opencadcodec-viewports/0003-viewport-off-state.patch
cargo test --config patches/opencadcodec-viewports.toml --workspace
```

For persistent opt-in in this worktree, put the contents of
`patches/opencadcodec-viewports.toml` in `.cargo/config.toml` (create `.cargo`
if needed; preserve any existing configuration). That local file is ignored.
The existing main integration selected the first two repairs; this IFCX-CAD
worktree selects all three. The deployment workflow prepares the same base and
three patches through
`.github/actions/viewport-codec` before Rust checks and browser compilation.
This workflow selection is included in the authorized integration into main.
Without the local file or the
explicit configuration, Cargo selects the unmodified upstream base; the new
codec and clip-preservation tests intentionally expose its known defects.

The stored fixture is hand-authored, with literal degree angles and a forward
CIRCLE boundary reference. `ocdraw_viewport_codec` tests raw DXF values rather
than relying only on a writer/reader pair. `ocdraw_viewport_clips` and
`ocdraw_viewport_clip_exchange` check active/dormant references, policies,
ordered conversion and strict production OCDraw readback through DXF/DWG.
The DWG profile requires a conventional overall paper canvas; this repair does
not address authored-viewport reclassification when it is absent.

Different Cargo configurations share Cargo.lock and must run sequentially.
Keep the lock consistent with the locally selected configuration. Replacing
these explicit repairs with an upstream revision requires a separate dependency
review. No measurements are required or authorized by this setup.
