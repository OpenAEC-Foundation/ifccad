# Public DWG corpus provenance

`public-dwg-v1.json` records exact upstream revisions, source paths, HTTPS URLs,
SHA-256 hashes, sizes and DWG signatures for 42 independently sourced test files:

- 22 DWGs present in DomCR/ACadSharp at the recorded revision.
- 8 DWGs in LibreCAD/libdxfrw's fixture collection at the recorded revision.
- The 12 largest DWGs in LibreDWG/libredwg at the recorded revision.

The selection was made from each repository's recursive Git tree on 2026-10-21.
Size is only a selection heuristic, not proof of semantic complexity. The source
repositories' license/provenance links are retained under `sources`. libdxfrw's
fixture README explicitly identifies its project-authored regression inputs.
ACadSharp has an MIT repository license. LibreDWG has a GPL-3.0-or-later repository
license; individual drawing provenance has not been separately verified. These
statements record source evidence and do not relicense the drawings or assert
permission for every possible redistribution/use.

Only the manifest is tracked here. Downloaded binaries, upstream license copies
and generated candidate files/reports belong in ignored local directories.
Never silently replace a file when its hash changes: review the new source and
update the manifest deliberately. No proprietary AutoCAD implementation or UI
assets are included. See [the workflow](../../docs/cad-roundtrip.md).
