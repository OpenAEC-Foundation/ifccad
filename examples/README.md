# IFCCAD examples

The Explorer offers two curated drawings:

| File | Purpose |
| --- | --- |
| `ifccad/entities.ifcx` | A numbered catalogue: all 16 native entity families, three Hatch fills, all nine Dimension kinds in one shared section, layers, simple/text line patterns, block transforms and an A3 Paper viewport. |
| `ifccad/house.ifcx` | A fictitious 10000 x 12000 mm ground floor with wall hatching, actual openings, repeated door/window/furniture blocks, room labels, dimensions, north arrow and an A3 sheet at 1:100. |

Both open with an authored Model view. Their Paper tabs contain locked orthographic
viewports and explicit millimetre plot mappings. One wide Dimensions section
groups all nine kinds in a three-by-three illustration. Each remains an ordinary
Model entity with an independent definition and an authored Generated picture,
including native SOLID arrowheads and measurement Text. Example captions and
reference lines/circles/arcs are independent ordinary entities, so rebuilding a
Dimension picture after a grip edit does not erase them. Linear projects the
same oblique reference span that the aligned example measures along its slope.
Each example stores native point-role links to its reference geometry. These
stored relationships do not certify automatic editor updates. Line/angle point
links have the active CAD mapping; circular parameter selectors and some arc
endpoint links are diagnosed if the DWG/DXF route cannot retain them. The
house dimensions retain independent
definitions without stored graphics and rely on CAD-host regeneration. Owned
parts do not add ordinary scope entities or duplicate stand-ins. The house uses whole millimetres; the catalogue uses up to two decimals
with trailing zeros suppressed. Units are stated on the drawing, not repeated in
every dimension label. Stored parts do not certify every CAD host's rendering.
SOLID shows a filled face and a filled arrowhead, as used in CAD symbols and
dimension pictures. Legacy TRACE shows a straight wide strip and two adjoining
strips with a mitred corner; their width is already expressed by the stored
corners. These SOLID/TRACE catalogue cases have zero extrusion thickness. Signed thickness,
concave, crossing and collapsed cases stay in regression and conformance tests
rather than the main entity catalogue. Their
parameter and CAD-type retention does not certify host filling; Solid Hatch
remains its own entity family.
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
The grouped Dimension section is authored separately from those small cards.
It reuses the reviewed native gallery fixture, explicitly scales the illustrated
geometry and authors new labels through the shared formatter. It is an example
recipe, not a general regeneration engine. Model and Paper views grow with it.

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

The separate local review gallery is generated by `cargo run --example extrusion_gallery -- --output <new-directory> --explorer`; retain its strictly read-back IFCX with the local review files. Its CAD/native control files were accepted on 30 October 2026. OCS #2361 tracks missing circle/arc extrusion display; stored properties and file conversion remain separate from rendering.

The Explorer variant adds an authored isometric Model view so the gallery is visible on opening. Its ordinary/saved geometry is exactly equal to the accepted control recipe; the control files are not regenerated or replaced by this camera-only variant.

The catalogue Point, Line segment, Circle, both Arc sweeps and both Planar polyline samples have thickness 15 mm and keep their base at Z=0. The existing reference cross, grid, layout and reuse examples are retained; no new block or sample section is added. The separate extrusion gallery is not shipped in the example picker.
