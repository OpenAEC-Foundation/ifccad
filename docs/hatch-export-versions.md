# Hatch DXF/DWG output qualification

Native model/validation/storage sections remain active. Direct `ifccad-convert`
exchange and codec qualification below describe the [archived reference](converter-parking.md);
the [shared Document route](architecture/explorer-document-route.md) has its own coverage.

This document qualifies the standalone reference converter, not the shared
Explorer mapping. See [the current route](architecture/explorer-document-route.md)
and [retired viewer integration](architecture/retired-viewer-converter.md).

This is correctness qualification of the selected unmodified codec `bd10efc` (package `acadrust` 0.6.6). It is not a benchmark, renderer
claim, CAD editing/update protocol, or guarantee for arbitrary drawings.
Native IFCCAD stores independent contours, literal fill, source groups,
associativity, provenance, TextIsland, classic editing and construction records.
The converter and physical file each have separate loss boundaries.

## Target matrix

Every cell is actually written and production-read in
`legacy/ifccad-convert/reference/crates/ifccad-convert/tests/hatch_version_matrix.rs`. Versions are checked against
the emitted header and reader; export never substitutes a different version.

| Format | Version | Sources, provenance, TextIsland | Active gradient |
| --- | --- | --- | --- |
| DXF | AC1015 | Retained | Delivery blocked: gradient requires R2004+ |
| DWG | AC1015 | Retained | Delivery blocked: gradient fields omitted |
| DXF | AC1018 | Retained | Retained |
| DWG | AC1018 | Retained | Retained |
| DXF | AC1021 | Retained | Retained |
| DWG | AC1021 | Retained | Retained |
| DXF | AC1024 | Retained | Retained |
| DWG | AC1024 | Retained | Retained |
| DXF | AC1027 | Retained | Retained |
| DWG | AC1027 | Retained | Retained |
| DXF | AC1032 | Retained | Retained |
| DWG | AC1032 | Retained | Retained |

The raw DXF writer emits gradient tags even with an AC1015 header and its reader
accepts them. Mutual codec agreement does not qualify version-conforming output:
the [ezdxf author documentation](https://ezdxf.readthedocs.io/en/stable/dxfentities/hatch.html#ezdxf.entities.Hatch.set_gradient)
requires R2004 or later for gradient fills. The
[Autodesk Hatch tag reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm)
describes the dependent 450–470 group. AC1015 raw behavior is characterized by
tests, while active gradient delivery is explicitly blocked for both formats.
The version stays selected so the user can decide what to do.

Coverage includes all 32 provenance/TextIsland combinations, enabled/disabled
and unbound association, ordered repeated sources, three fills and area rules,
edge/polyline backings, Model/Paper/block owners, supported text/MText and whole
BlockInstance/curve sources. Text provider cases use a bounded CAD-compatible
text profile; richer unsupported text presentation is not implicitly enabled.
Independent hand-authored DXF loops pin literal expectations. Gradient coverage
includes nine types, both modes and concrete RGB/ACI endpoints.

## Remaining physical losses

These conditions are independent of source-group transport:

| Condition | DXF | DWG |
| --- | --- | --- |
| Enabled association without sources | Switch cleared, all six versions | Switch cleared, all six versions |
| Inactive gradient state | Omitted, all six versions | Omitted in AC1015; retained in AC1018+ |
| Reported definition-line/general angles from codec #132 | Exact source bits retained in all six versions | Qualified source values retained |
| Pattern description | Omitted, all six versions | Omitted, all six versions |
| Classic definition lines/angle/scale/double in solid mode | Omitted, all six versions | Omitted, all six versions |
| Pixel size with no existing DERIVED path | Omitted, all six versions | Omitted, all six versions |
| Ordered seeds, including duplicates/outside points | Retained, all six versions | Retained, all six versions |
| Pattern creation type and stored ACAD XY origin | Retained, all six versions | Retained, all six versions |

No DERIVED provenance is fabricated to retain pixel size. No fill is silently
downgraded. Changed/missing correspondence, owners or loops, partial nonempty
source groups and invalid strict readback remain hard failures. Complete group
omission and known field loss have located diagnostics; Allow offers diagnosed
output, Reject offers no download. Runtime readback remains required even when
a target profile passed its regression cases.

`TARGET_CODEC_HATCH_*_LOSS` diagnostics distinguish association state, source
group, provenance, TextIsland, gradient state/parameters, classic settings and
construction. The existing pattern-description code remains available.
`HATCH_TARGET_FEATURE_UNSUPPORTED` names a necessarily blocked active profile;
`HATCH_TARGET_PROFILE_UNQUALIFIED` identifies unavailable qualification rather
than asserting impossibility. A default-tolerance preparation failure is instead
`HATCH_PREFLIGHT_UNAVAILABLE`: known mandatory target restrictions still apply,
and actual export checks the user's selected output tolerance. This notice alone
does not assert loss or disable an otherwise valid custom-tolerance export.
`HATCH_RECIPE_PREFLIGHT_CHECK` similarly makes a default-tolerance recipe trial
advisory, including under Reject; actual export rechecks the selected tolerance
and still refuses any resulting semantic loss under Reject.
The former viewer selected the current format/version entry from these Rust
profiles. That integration is retired. The current Explorer reports the shared
mapping’s own losses and does not apply these reference-codec preflights.

## Original AC1024 practice acceptance

The following is the historical reference-viewer correctness regression. Run it
only in an isolated checkout of the recorded `f80d7bd` reference snapshot, using
its documented codec preparation and both original 3bm files:

```text
IFCCAD_HATCH_FOUNDATION_INPUT_ROOT=<directory containing both originals>
cargo test -p viewer --test hatch_foundation_regression -- --ignored --nocapture
```

The basename is `2705_model Funderingsherstel - Constructie - Sheet - CP-21 -
Constructietekening`. Originals must match:

| Input | Bytes | SHA256 |
| --- | ---: | --- |
| DWG | 239012 | 24c225ffffae53f779646f824f4230799cbf39c475c0eb14f365997a453301b5 |
| DXF | 1314182 | 054317ae053be20d9c0f9c8650b5070f76826a7eede9690cd9c572700ea6149c |

Each original has 85 Hatches. One spline-boundary Hatch remains explicitly
unsupported by the deferred spline profile (DWG handle 150E); 84 eligible
Hatches are compared in every output cell. Known clockwise controls are DXF
9AA and DWG 9A7. The complete original is imported under its existing Allow
policy with that located omission, never manually filtered or repaired.
Other existing non-Hatch diagnostics remain separate. Dimensions/owned graphics
remain part of the complete conversion; their slice is not reopened.

The original-parameter candidate proof precedes the storage migration: all twelve
active patterns per original qualify at the unchanged 1e-9 limit on real contours.
These patterns are local leaves, with no relevant original block occurrence;
separate nested nonuniform controls verify amplification and fallback. The native
record and qualified DWG output retain exact raw definition values and absent
origins. Ten DWG-source and eleven DXF-source model recipe losses are removed.
Upstream [opencadcodec #132](https://github.com/HakanSeven12/opencadcodec/issues/132)
is adopted at `bd10efc`: the reported DXF angles and original practice recipes
now require exact scalar retention. Physical comparison remains active and
locates any genuinely changed field; this is not a claim that every arbitrary
binary64 radian has an exact degree preimage. This is not a renderer/editor-regeneration assessment.

The regression requires twelve actual target cells per original representation,
strict native/file readback and unchanged source hashes. Missing input or a
different hash fails the explicit acceptance run. It is ignored in ordinary CI
because external user source files are not distributed; synthetic regressions
remain ordinary CI tests. The historical reference-viewer AC1024 selection remained AC1024.

For the bd10efc adoption, the user stopped the repeated complete-original matrix
after 19 successful cells, before the native upright extension. These remain
scoped evidence for the pin update, not a full 24-cell pass of the final combined
candidate. The new mode uses small targeted DXF AC1024/AC1027 and DWG AC1024
controls; the complete external fixture remains available for an explicit run.
