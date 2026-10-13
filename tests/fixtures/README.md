# Shared drawing regression inputs

`ifccad/` and `ocdraw/` contain focused drawing inputs formerly shown as individual
Explorer examples. The relocation preserves their bytes and their regression
purpose: clipping, workspace/high-width identities, source composition, primitive
geometry, text, preservation and individual Hatch families. They are test/recipe
inputs rather than entries in the public example chooser.

Keep their native routes and contracts separate. They are not numbered
conformance collections and do not replace those collections. Curated drawings
are under `examples/ifccad/` and `examples/ocdraw/`; test specialized variants here
before incorporating a representative case into a curated overview/text/Hatch
drawing. Generator fixture outputs must use an explicitly selected root so that
regeneration does not restore the retired example list.
