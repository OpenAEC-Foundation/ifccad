# IFCCAD examples

The four curated examples are `ifccad/overview.ifcx`, `ifccad/text.ifcx`,
`ifccad/hatches.ifcx` and `ifccad/dimensions.ifcx`. They cover Model/Paper layouts,
blocks, appearance, line patterns, workspace, text, Hatch and ordinary Dimensions.
They are authored source assets, separate from focused test fixtures and conformance.

The Explorer ships only these IFCCAD examples. OCDraw counterparts and generators
are parked in `legacy/ocdraw/examples`; restore the complete checkpoint for legacy
reproduction. Numbered conformance and historical measurement evidence stay in place.

Use `cargo run --example write_explorer_examples -- <new-directory>` to create a
new curated set, or `write_hatch_examples` for the Hatch pair. Generators refuse
to overwrite existing files through the production writer. Their output is loaded
with the production reader and strict validation. Inspect generated changes before
replacing maintained examples; generation is not a completion gate or measurement.
