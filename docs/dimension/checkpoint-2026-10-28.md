# Remaining Dimensions: checkpoint, 28 October 2026

Work is on the IFCCAD `remaining-dimensions` branch. This is an unpublished
development candidate; no merge into main, software release or live Explorer
deployment is included. The user asked to stop for today and resume tomorrow.

## Implemented and connected

- All nine native kinds, shared validation/measurement/formatting, source roles,
  IFCX profile 0.2.0, CBOR mapping 2 and matching inspection are implemented.
  Published alpha v2026.44.0 and its tagged profile 0.1.0/CBOR1 remain fixed.
- [opencaddocument PR #7](https://github.com/HakanSeven12/opencaddocument/pull/7)
  is open at `ad6fec3f99dae5f7f0e93aca8ef5e4c98ca680ea`. Its default, serde and
  serde+ifcx CI matrix passes on Linux, macOS and Windows. It uses native core
  `bce9da06a8e1684e86e35f0302e3870141e6f9f9` and unmodified codec
  `501b694eee3433b157e56a32da4268dc8b3b5bd3`. The canonical short codec selector
  is shared with Studio/kernel; Cargo.lock records the exact resolved commit.
- Explorer uses that PR revision. Studio base
  `4d1cd17dafd7d47cd76f8674835636776eb90fd6` uses the same Document/core/codec
  through explicit overrides, including both plugin API dependencies.
  [The preparation script](../../scripts/prepare_explorer_ocs.py),
  [frozen host lockfile](../../format-explorer/ocs-inputs/Cargo.lock) and
  [source descriptor](../../format-explorer/ocs-source.json) reproduce this.
  Main and parser-worker assets have checked build provenance; Explorer WASM
  is built separately. The local Studio build uses a development main bundle
  and the upstream release worker; this does not certify a production deployment.
- The local Explorer runs at `http://127.0.0.1:4187/`. The user accepted the
  sample_AC1032 DWG-to-IFCX display there, after accepting the CAD roundtrip
  review package. Regeneration with final PR sources retains the same reviewed
  Dimension semantics and reported losses. This is bounded acceptance evidence.

## Latest catalogue changes

The entity catalogue has one wide, three-by-three Dimension section. Linear
projects an oblique reference span to 18 mm; aligned measures its slope as
21.63 mm. All nine remain ordinary, independently inspectable Model entities.

Captions and reference lines/circles/arcs are now separate ordinary entities.
Only actual annotation belongs to each stored Dimension picture. Studio's
`Scene::apply_grip` invalidates the old baked picture and regenerates the
annotation. Previously that also removed the example caption and reference
geometry because the example had included them in the picture. The corrected
example retains them; a real Studio grip edit was checked visually.

All nine examples store native role/selector references to their source
geometry. The inspected two-line angle is recognized as associated in Studio.
This does not qualify every automatic source update or reactor path. Circular
parameters and some arc endpoint links still produce explicit loss diagnostics
on CAD conversion; see the PR coverage contract. Shared annotation contexts and
automatic associative-update qualification remain follow-up work.

The inspector's raw angle value now uses radians rather than the drawing's
length unit. Formatted angle labels retain their existing number formatting.

## Verification at this checkpoint

- Final Rust workspace audit, fmt, all-target Clippy with warnings denied and
  full workspace tests passed on the final reference/caption/association example
  candidate (including native Zstd coverage). The full test command took 120 s.
- The subsequent changes are limited to the frontend angle-unit display and its
  regression test, plus prose. They do not change Rust build/test inputs.
  Full frontend checks then passed: 162 passed, 5 expected skips, no failures.
- Ten focused example tests pass, including independent captions, oblique linear
  projection and native source roles surviving picture removal and CBOR readback.
- Actual browser WASM tests pass for the updated catalogue and house, native
  IFCX/CBOR/Zstd, direct native preview and production DXF/DWG readback.
  The house asset is byte-for-byte unchanged by the catalogue work.
- Release-helper tests: 7 passed. Workflow lint and diff checks passed.

Detailed input hashes, dependency/configuration/tool identities, commands, exit
codes and logs remain in the ignored task evidence directory for this task.
The accepted review package and load-time measurement baseline are retained.
No performance or size measurements were rerun.

## Resume tomorrow

1. Review the final grouped catalogue with the user, particularly native source
   relationships and the distinction between stored links and automatic updates.
2. Follow PR #7 review. Any changed dependency revision needs an explicit pin and
   relevant verification; do not silently adopt the newer upstream codec bump.
3. Integrate, release or deploy only when separately requested. Main and the live
   Explorer are unchanged. Keep the active worktrees, evidence and warm caches;
   the local viewer and pending integration still use them.

Other worktrees and unrelated main-checkout changes are preserved. The local
Document checkout retains only an uncommitted standalone-workspace setup line;
that setup is deliberately excluded from PR #7. No separate Studio contribution
is required to reproduce the checked dependency overrides above.
