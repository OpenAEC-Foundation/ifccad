# CAD Format Explorer examples

The explorer chooser uses the representative drawings below. Six are
authored by `write_explorer_examples.rs`, which production-reads every result.
Regenerate with `cargo run --example write_explorer_examples -- examples`.
Generated native files here are source examples; WASM, CAD build output and
screenshots remain local artifacts outside this directory.

All files under `ifccad/` use the writer's node presentation order: `path`,
optional `children`, then `attributes`. Additional IFCX fields such as `inherits`
follow these fields and retain their values. IDs are compact per domain, with
entity IDs starting at 1 in Model/Paper/block writer traversal order. ID 0 remains
for the existing standard layer and continuous line pattern. References and
allocation counters follow the new IDs; multiple source contributions to one
node remain separate.

Normalize authored IFCCAD examples with
`cargo run --example normalize_ifccad_examples -- examples/ifccad`.
The explorer generator uses the same example-only normalization. The production
writer preserves caller-supplied IDs and allocation history; sparse/high IDs remain
valid and are still exercised by conformance and writer tests.

| Example | Demonstrated content |
| --- | --- |
| IFCCAD overview | Current native line/circle/planar-polyline families, multiple layouts, analytic clipping, layers and named patterns, unused definitions, shared/nested blocks, reflected scale and nonzero block base |
| IFCCAD layouts and viewports | Paper coordinates with a fixed millimetre plot mapping and independent physical A3 media, viewport frame/camera/display/depth state, clipping and frozen layers with populated Model content |
| IFCCAD blocks and fragments | Nested/shared definitions, multiple contributions to the same node, foreign attributes/nodes and children/inherits references that do not confer CAD ownership |
| OCDraw overview | Line, point, circle, arc, full/partial ellipse, bulged planar polyline, spatial polyline, shared/nested blocks, reflected transforms, named UCS, layers/patterns and allocation state |
| OCDraw layouts and viewports | Model geometry and paper viewport with a closed bulged clip, clipping activation, scoped membership and bounds |
| OCDraw state and storage | Named/current UCS, active model window, view/grid/snap and dormant depth state, interleaved draw order across streams and pooled coordinates |
| OCDraw preserved spline | Opaque entity identity/ownership, typed SPLINE source record, source provenance and guarded CAD restoration |
| OCDraw Text and MText | Requested CAD font, distinct Text layouts, literal Unicode/control-like spellings, authored paragraph formatting, a fraction stack, trailing empty paragraph, canvas background and estimated bounds |

The IFCCAD overview now also includes points, arcs, full and partial ellipses,
and spatial polylines introduced by geometry-bounds. Its valid stored bounds
are prepared by the production core while original fragment structure stays
intact. The additional primitive tuples are authored in the new drawing's
millimetre coordinate system; the source reference document is unchanged.

The spline example is generated separately with
`cargo run -p viewer --example explorer_preservation -- examples/ocdraw/preserved-spline.ocdraw.json`.
This producer refuses overwrite and validates the actual generated native
bytes. Its source is the pinned `open-cubic.dxf` converter fixture. It exposes
source-only geometry explicitly and does not substitute a sampled native curve.

The text example is generated separately with
`cargo run --example write_text -- examples/ocdraw/text.ocdraw.json` on a new output
path. It passes production native readback. Its glyph outlines/layout are not
independently verified; the Explorer distinguishes estimates from producer claims.
The qualified CAD subset and diagnosed authoring-dependency changes are described
in [text support](../docs/text.md). IFCCAD text remains deferred.

The overview covers semantic families, not every enum value or numerical edge
case; candidate conformance and Rust tests cover those boundaries. IFCCAD's
current geometry differs from OCDraw. The preserved spline example covers the
approved typed OCDraw source-preservation route; native spline geometry, IFCCAD text
and unimplemented plot semantics are not implied by this collection.
Additional physical clip shapes and plot/paper workspace state should
be represented by focused examples as this collection expands.

The recipe reuses the independently validated viewport construction from
`ifccad/hello-viewports.ifcx` and the candidate OCDraw bulged-viewport fixture,
then authors new Model geometry, layers/patterns, blocks, state and source
contributions around them. The intended IFCX fragment example has exactly two
source contributions for `/cad/d1/e1`, with geometry supplied by the later
fragment; its foreign graph information is retained natively and may be
diagnosed when projected to CAD. All claims are about the experimental IFCCAD
profile, not arbitrary IFCX conformance or lossless CAD exchange.

No size/exchange measurement is part of generation or example validation.
