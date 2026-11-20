# External drawing sources

Profile 0.12.0 introduces drawing-local external sources and Model XREF
attachments. A native entity has its ordinary entity ID and refers to an
independent `IfccadExternalSourceId`. Allocation uses
`nextExternalSourceId` (omitted means 1); deletion never resets it. Multiple
sources and placements are allowed, including unused sources. Equal locations
or GUIDs do not merge identities.

Each `/cad/dN/externalSource/I` node is a drawing child with one
`ifccad::externalSource` attribute. Its closed value contains `name`,
`location` and typed `source` content. Initially `source.format` is `dwg`;
`basePoint` is required, `insertionUnit` and `fingerprintGuid` are optional.
A supplied unit uses the existing coordinate-unit tokens. A supplied GUID is
a canonical lowercase UUID string. This source identity is independent of the
main document's identity. XREF sources have no content hash.

The two location variants are:

```json
{"kind":"uriReference","uri":"../Referenties/grondplan.dwg"}
{"kind":"searchFileName","fileName":"grondplan.dwg"}
```

`uriReference` accepts a nonempty RFC 3986 URI-reference, including relative
references, absolute `file:` URIs and other schemes. Its exact spelling is
retained; native IO neither normalizes it nor opens it. Space and non-ASCII
filename bytes must be URI-encoded. `grondplan.dwg` directly names a file
beside the main document; its meaning differs from `searchFileName`.
Search names contain literal text without separators, a drive prefix, NUL,
`.` or `..`. URI query/fragment syntax can be retained even when a host cannot
resolve it. This `uri` belongs to the custom source record and is independent
of IFCX `imports[].uri` and schema URI fields.

`ifccad::xref` contains a drawing-local `source` path and a positive finite
`scale`. The existing `ifccad::geom::placement` supplies O/X/Y. The first slice
allows Model attachments with rotation about Z and positive uniform scale.
The scale is effective: any source-to-main unit factor has already been
applied once. A source point is transformed relative to the source's stored
base point before scale and placement. Source geometry stays external.

A missing source is a valid native drawing. Core validation does no external
IO, and a marker at the insertion point cannot establish complete bounds.
Any scope containing an XREF therefore has unknown full extents. Host loading,
search order, relinking, reload and GUID mismatch messages are host concerns.
An unsupported URI scheme stays stored and unresolved. A found session path
does not replace the authored location implicitly.

CBOR mapping 12 uses document key 29, allocation key 17 and entity-kind code
22. The detailed record codes and omission rules are in the
[machine-readable mapping](../schemas/ifccad/cbor-mapping-12.json).
Native JSON and CBOR share the same intrinsic and ownership validation.

DXF and IFCCAD source formats, nested graphs, overlays, binding, packaging and
remote downloads are deferred. SHX and CTB/STB keep their existing records
until the shared source model has demonstrated stable XREF behavior.

## Host and CAD exchange

The development pair uses Document `353bdef5` and codec `13f16ebc`, with explicit
local XREF patches. It is recorded in `format-explorer/ocs-source.json`; these
base revisions alone do not supply the extension. Studio uses its per-tab
native roundtrip context to retain IDs, original URI spelling and expected GUID.
Browser source selection binds chosen DWG bytes to that tab, source ID and stored
locator. A same-named file in another document cannot supply those bytes.
Closing the tab releases its registrations when its snapshots are released.

Desktop resolution tries the stored path, the host directory's basename, its
`xref/` subdirectory, then directories in `OCS_XREF_PATH`, in that order. A
relative locator is based on the host drawing's directory. An unsupported URI
does not enter basename fallback or start a download. Explicit browser choices
retain the authored locator; permanent relinking remains a separate edit.

The expected FingerprintGUID comes from the DWG dependency record, independently
of the main header GUID. A mismatch warns and continues loading for both native
and DWG hosts. Session observations do not overwrite the native expected GUID.
A changed source base point or known insertion unit refuses replacement before
the existing displayed body is removed; placement and scale require reassessment.
The source base is subtracted once when preparing display geometry. Newly
attached sources receive their unit factor once in the effective INSERT scale.

DWG FileDepList preserves the optional expected GUID. DXF follows the existing
OCS writer behavior: it does not carry this list and adds no new GUID warning.
Other physical dependency metadata has no native source field and is diagnosed
on CAD-to-native conversion. URI schemes without a whole-file CAD path mapping
block DWG/DXF export instead of silently changing the location.


## First-slice qualification

The explicit local development pair passed the full native gate, the optional
Zstd checks, the six Document configurations, the five native codec CI
configurations and 151 Studio XREF tests. The actual desktop executable also
passed source editing, grip movement, undo/redo and fresh-process offline save.
Hidden Explorer checks exercised both document isolation and GUID mismatch,
source reload and physical CAD download/readback. The actual module worker
used protocol 7 and carried the physical DWG dependency GUID.

The compact project uses source metres, INSBASE (10,20,0), host millimetres,
O=(10000,5000,0), X=(0,1,0), Y=(-1,0,0), and effective scale 500. The line from
(10,20,0) to (12,20,0) lands at (10000,5000,0) to (10000,6000,0), within
0.001 mm. A source endpoint edit to 13 gives a 1500 mm line. This bounded
synthetic qualification is not a practice-corpus or performance measurement.

Independent LibreDWG geometry/header readback and a literal ODA FileDepList
parser qualify the retained project. An AutoCAD-authored known-GUID fixture
remains an explicitly open external qualification item. Native staging,
downstream review and publication remain separate actions under the release
policy; local paths and development patches are not official upstream pins.
