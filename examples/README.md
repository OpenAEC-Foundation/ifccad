# CAD Format Explorer examples

There are four representative drawings per native format. IFCCAD and OCDraw
remain independently authored and opened; neither route requires the other.

| Drawing | Content |
| --- | --- |
| `ifccad/overview.ifcx` | Model and A3 Paper, geometry families, shared/nested blocks, layers/patterns, clipping, named UCS/model window, viewport workspace and IFCX source contributions/foreign references |
| `ocdraw/overview.ocdraw.json` | Model and A3 Paper, geometry families, shared/nested blocks, layers/patterns, bulged viewport clip, named UCS/model window and viewport workspace |
| `ifccad/text.ifcx`, `ocdraw/text.ocdraw.json` | Text/MText, text styles, formatting, paragraphs and supported background/layout choices |
| `ifccad/hatches.ifcx`, `ocdraw/hatches.ocdraw.json` | Solid and line-pattern fills side by side, contour source links, authored pattern phase and spacing |

| `ifccad/dimensions.ifcx`, `ocdraw/dimensions.ocdraw.json` | Linear/aligned dimensions, styles and overrides, Model/Paper/block owners and source references |

The chooser contains only these eight files. Add supported semantic variants to
the appropriate drawing instead of adding one chooser entry per enum value.
Both overviews include a line-pattern palette: GAS text, `ltypeshp.shx` shape 132,
custom external shape 42, mixed markers, unused definitions and duplicate source
filenames. Their Model, Paper and shared block contexts retain separate glyph
coverage evidence; external assets are not required for native validity.
Focused regression cases and recipe inputs live in `tests/fixtures/ifccad/` and
`tests/fixtures/ocdraw/`; conformance collections remain separate and unchanged.
Rust programs below `examples/` demonstrate authoring APIs or generate drawings.
Unrelated local experiments are not part of the curated set.

White foreground layers keep `indexedColor` (`system: ACI`, `index: 7`) plus RGB
white fallback. CAD viewers draw them white on dark Model backgrounds and black
on light Paper. Authored true-white backgrounds/masks remain RGB white.

IFCCAD nodes use writer presentation order: `path`, optional `children`, then
`attributes`, followed by extra IFCX fields. IDs are compact per domain, with
entity IDs starting at 1 in Model/Paper/block traversal order. Standard layer and
continuous-pattern ID 0 remain. Multiple contributions to a node remain separate.
The production writer still preserves caller IDs/allocation history; focused
fixtures retain sparse/high-ID and source-composition coverage.

Generate overviews with explicitly selected output directories:

```text
cargo run --example write_explorer_examples -- <examples-directory> <fixture-output-directory>
cargo run --example write_hatch_examples -- <examples-directory>
cargo run --example write_ifccad_text -- <new-ifccad-text-path>
cargo run --example write_text -- <new-ocdraw-text-path>
```

Use fresh output directories for regeneration/review. The overview producer
writes only the two curated overviews into the examples directory; specialized
workspace/layout/composition outputs go to the separately selected fixture root.
Hatch production writes the two curated Hatch drawings. Producers use production
native writers/readers; browser smoke checks open all eight drawings, export DXF
and DWG, and strict-read the actual outputs. Native geometry/bounds and source
links remain distinct from unassessed fill topology and estimated text extents.
Pattern/gradient coverage follows the current native/converter contracts; a
successful preview is not a claim of whole-file CAD fidelity.

Spline preservation and numerical/physical/encoding edge cases remain focused
fixtures and tests. No size/exchange measurement is part of generation or example
validation. See `tests/fixtures/README.md` for regression input ownership.
