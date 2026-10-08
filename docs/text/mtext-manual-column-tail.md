# CAD MText manual-column tail qualification

The final dynamic manual column takes the remaining content. Its stored height
does not define a fixed layout cap. OCDraw imports it as the existing native
`MTextColumnHeight::Auto`, retaining column count, width, gutter, flow and positive
fixed heights before that final entry. CAD export emits Auto as a zero height
sentinel. Native fixed final caps remain valid in OCDraw but are unsupported
in this CAD profile: Allow skips the whole text with a located diagnostic,
and Reject refuses the loss.

This is a conversion qualification, not a change to the native model or schemas.
IFCCAD text integration remains deferred. Glyph contours, rendered extents and
overflow are not established by these IO checks.

## Evidence and root cause

The retained Sample_AC1032 DWG is byte-identical to the public
[ACadSharp sample at revision 0647526036f472e967b4e65343240a3514f86fe9](https://github.com/DomCR/ACadSharp/blob/0647526036f472e967b4e65343240a3514f86fe9/samples/sample_AC1032.dwg).
The same revision supplies matching ASCII and binary DXF samples.

| Input | SHA256 |
| --- | --- |
| `sample_AC1032.dwg` | `0e8faaca949c9429c92240082d9ea4d3524aca5baf7c7815ec439245c321b528` |
| `sample_AC1032_ascii.dxf` | `c2934f616ecd7097e773b85e5d6e0548d21ccc635abcc8e7276f7d4a585a7ae2` |
| `sample_AC1032_binary.dxf` | `e2c59abecc7bc0fd0ace7b9b6f083d2a16d36077b7a7d7ceccd2aa7edfb348e1` |

Four single-column manual states have the following stored final heights:

| MText handle | DWG | Both DXF variants |
| --- | ---: | ---: |
| `3EC` | approximately -2.944292860391087 | 0 |
| `3F5` | approximately -23.726011823556163 | 0 |
| `3F6` | -3 | 0 |
| `779` | approximately -204.34970441109567 | 0 |

An independent GNU LibreDWG 0.14 read of the same DWG succeeds and reports these
negative values too. They are not specific to opencadcodec's decode. The official
[portable release](https://github.com/LibreDWG/libredwg/releases/tag/0.14) was used;
the win64 ZIP SHA256 is
`1ad7e15344d20b3426c3435b078d82fb84b35062815946b2cca9c5fc9810fea8`.

The [ezdxf layout documentation](https://ezdxf.readthedocs.io/en/stable/layouts/layouts.html#ezdxf.layouts.BaseLayout.add_mtext_dynamic_manual_height_columns)
explains that the final manual column takes remaining content and ignores its
stored height. Its independent
[renderer at revision d389b47693cf0abce3d10d1816c42c1767d7abe7](https://github.com/mozman/ezdxf/blob/d389b47693cf0abce3d10d1816c42c1767d7abe7/src/ezdxf/render/abstract_mtext_renderer.py#L150)
sets the last layout height to automatic regardless of its stored sign/value.
That source rule, together with the paired files, qualifies the native Auto
interpretation. No application rendering comparison was performed.

The former helper applied positive-distance validation to every manual height.
It consequently rejected the legitimate final cache value in the DWG and treated
the zero DXF sentinel as unqualified. Taking an absolute value would also be
wrong: it would invent a fixed cap, and the vertical-flow `3F5` value does not
even equal its cached overall text height.

## Qualified boundaries and checks

Every stored height must be finite, including the ignored final entry. Earlier
heights must be strictly positive. Count/list agreement, positive column width,
nonnegative gutter and the existing reference-width profile remain checked.
No count or authored height is reconstructed from cached extents. The final
finite entry becomes Auto even when its stored value is positive. The native
format continues to allow fixed final caps; their CAD restriction is explicit.

An independently reconstructed minimal AC1032 ASCII fixture exercises the
embedded zero sentinel through the production DXF reader. Focused regressions
exercise negative/zero/positive final values, invalid earlier heights, nonfinite
values, count mismatch, native strict readback, Allow/Reject diagnostics, and
actual DXF/AC1032 DWG exchange of one- and two-column Auto tails. These tests
compare semantic column state and text rather than runtime handles.

LibreDWG independently reads a newly emitted two-column DWG as manual columns
with heights `[60, 0]`, width 40 and gutter 5, and reports success. This checks
the written sentinel separately from the pinned writer/reader pair. The fix
passes all 783 workspace tests in a temporary source candidate based on main
`d6082df8790041a07814eaa490e66820d44250a7` (one pre-existing test remains ignored),
along with workspace formatting and Clippy with warnings denied. The original
`8dede55` base also passed its full 750-test suite. These checks do not imply
that the candidate has been committed or integrated.

The three complete samples now import to OCDraw and pass production strict
readback with 23 Text and 19 MText entities. Their existing unsupported-semantics
diagnostics remain present; this is not a lossless claim for all drawing content.
Full DXF re-export/readback succeeds. Full DWG re-export still exposes the
separate additional-paper-block ownership problem described in
[opencadcodec issue 78](https://github.com/HakanSeven12/opencadcodec/issues/78).
The additional markers have the correct names but point to the primary paper
record as owner, rather than their own linked block records.
The focused Auto-tail DWG tests do not contain those extra paper blocks and pass.

All converter evidence uses opencadcodec base revision
`063c10671fe7833d562f772159771318c7a0ebb9` with the existing explicit
viewport-off-state patch. No dependency upgrade or source-cache edit was made.
These are correctness and qualification checks; no size/exchange measurement
was requested or run.
