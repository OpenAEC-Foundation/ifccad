# IFCCAD examples

The Explorer offers two curated drawings:

| File | Purpose |
| --- | --- |
| `ifccad/entities.ifcx` | A numbered catalogue: all 14 native entity families, three Hatch fills, linear/aligned Dimensions, layers, simple/text line patterns, block transforms and an A3 Paper viewport. |
| `ifccad/house.ifcx` | A fictitious 10000 x 12000 mm ground floor with wall hatching, actual openings, repeated door/window/furniture blocks, room labels, dimensions, north arrow and an A3 sheet at 1:100. |

Both open with an authored Model view. Their Paper tabs contain locked orthographic
viewports and explicit millimetre plot mappings. Dimensions use native definitions
only; the CAD host regenerates their display. There are no duplicate line/text
stand-ins. The house uses whole millimetres; the catalogue uses up to two decimals
with trailing zeros suppressed. Units are stated on the drawing, not repeated in
every dimension label. This does not add baked native Dimension graphics or
certify every CAD host's rendering.
The Point sample includes a separate cross so its position stays visible.

## Reproduction

Run `cargo run --example write_explorer_examples -- <new-directory>` from the
repository root. It builds both drawings, recomputes bounds, encodes them through
the production writer, strictly reads them back and normalizes example IDs/node
order. The program refuses to overwrite existing output files. Compare the new
files before replacing the two maintained assets. The focused check is
`cargo test -p ifccad --test explorer_examples`; WASM smoke also exercises both
curated examples through DXF/DWG production readback.

## Extending the catalogue

Edit `support/explorer_catalogue.rs`: add one `Sample` to `SAMPLES`, with a title,
a short note and a drawing function. Each function gets the lower-left local
origin of an 82 x 42 mm drawing area. The shared builder places the frame,
number and labels; adding entries automatically grows the four-column grid,
Paper viewport and initial Model view. Use the common authoring helpers in
`support/explorer_drawing.rs`; keep family-specific content in the sample.
Append entries to preserve the reading order. New resources need ordinary
drawing-local references and allocation counters, just like existing samples.

Regenerate both maintained assets. The tests check recipe reproducibility,
strict validity, every current native family and catalogue growth across an
extra row. The exhaustive entity-kind match intentionally requires a coverage
decision when a native variant is introduced. Add new semantic variants to the
coverage assertions when a family expands; no test can infer every future
variant automatically. Do not present an unimplemented family as supported.

## Retained regression assets

`overview.ifcx`, `text.ifcx`, `hatches.ifcx` and `dimensions.ifcx` remain unchanged
for focused tests and historical benchmark provenance. They are not in the
chooser or published site bundle. The catalogue reuses the qualified ordinary
Dimension style from `dimensions.ifcx`. The older overview still demonstrates
IFCX fragments, workspace and external SHX requests. No benchmark was rerun.
OCDraw assets remain parked in `legacy/ocdraw/examples`.
