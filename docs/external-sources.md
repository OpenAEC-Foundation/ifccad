# External drawing sources

Profile 0.12.0 introduced drawing-local external sources and Model XREF
attachments. The unpublished 0.13.0 draft extends placement and ownership as
described below; published 0.12.0 files retain their original contract. A native entity has its ordinary entity ID and refers to an
independent `IfccadExternalSourceId`. Allocation uses
`nextExternalSourceId` (omitted means 1); deletion never resets it. Multiple
sources and placements are allowed, including unused sources. Equal locations
or GUIDs do not merge identities.

Each `/cad/dN/externalSource/I` node is a drawing child with one
`ifccad::externalSource` attribute. Its closed value contains `name`,
`location` and typed `source` content. `source.format` is `dwg`, `dxf` or `ifccad`. `basePoint` is required and
`insertionUnit` is optional. DWG and DXF may carry `fingerprintGuid`; the
IFCCAD variant has no fingerprint field.
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

`ifccad::xref` contains a drawing-local `source` path and `transform`, using
`IfccadBlockTransform`: an orthonormal O/X/Y owner frame, rotation in radians
(default zero) and signed finite nonzero XYZ scale (default one). It has no
separate `ifccad::geom::placement`. Model, Paper layouts and local blocks may
own XREFs. The effective scale already includes insertion-unit conversion.
The source base point is subtracted once before scale, rotation and frame:
`p_owner = O + F * Rz(rotation) * diag(scale) * (p_source - basePoint)`.
Nested placements retain their graph; no affine decomposition is required.
A local block boundary does not stop Overlay propagation; an external drawing
boundary does. Only source Model geometry is loaded, including when the owner
is a Paper layout. Source geometry stays external.

A missing source is a valid native drawing. Core validation does no external
IO, and a marker at the insertion point cannot establish complete bounds.
Any scope containing an XREF therefore has unknown full extents. Host loading,
search order, relinking, reload and GUID mismatch messages are host concerns.
An unsupported URI scheme stays stored and unresolved. A found session path
does not replace the authored location implicitly.

CBOR mapping 13 uses document key 29, allocation key 17 and entity-kind code
22. The detailed record codes and omission rules are in the
[machine-readable mapping](../schemas/ifccad/cbor-mapping-13.json).
Native JSON and CBOR share the same intrinsic and ownership validation.

The optional XREF `referenceType` is `attach` (omitted default) or `overlay`.
An overlay is visible directly but does not propagate when its owner is referenced.
Source descriptors can be shared by relationships of either type. Core validation
does not traverse external files. OCS supplies finite host-owned loading and
session-only cycle interruption; no cycle cut is stored in the drawing.

Bind for the native workflow, complete project archives and remote downloads
remain outside this qualified slice. SHX and CTB/STB retain their existing
records. Their migration to the shared locator/source model is a separate execution
slice, planned for the same unpublished 0.13.0 / CBOR 13 draft and future release.
Placement alone does not authorize publication of that joint release.

## Host and CAD exchange

The release integration selects official Studio merge
`0ca42af051bfdf808f8824e15132c8504dbab331` ([#3651](https://github.com/HakanSeven12/OpenCADStudio/pull/3651)),
official Document merge `1db213a7babde93b64a4887fd6ba441f0177b00d`
([#32](https://github.com/HakanSeven12/opencaddocument/pull/32)), canonical codec
`b0b768cee674c28bac63c75cbd5638b894f33f23` and native contract `785590abb669c38afb700b339d36cc8cc0c18161`.
`format-explorer/ocs-source.json` records the exact shipped identities. No
source patch or development dependency override is required.
Studio uses its per-tab
native roundtrip context to retain IDs, original URI spelling and expected GUID.
Browser source selection binds chosen drawing bytes to that tab, source ID and stored
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
A changed source format, base point or known insertion unit refuses replacement before
the existing displayed body is removed; placement and scale require reassessment.
The source base is subtracted once when preparing display geometry. Newly
attached sources receive their unit factor once in the effective INSERT scale.

DWG FileDepList preserves the optional expected GUID. DXF follows the existing
OCS writer behavior: it does not carry this list and adds no new GUID warning.
Other physical dependency metadata has no native source field and is diagnosed
on CAD-to-native conversion. URI schemes without a whole-file CAD path mapping
block DWG/DXF export instead of silently changing the location.


## Additional source formats

DXF sources use the production DXF reader and its INSBASE, INSUNITS and optional
header FingerprintGUID. Native sources use the strict IFCCAD JSON or CBOR reader,
origin `(0,0,0)` and the native length unit; no source UUID is invented. Zstd
source decoding is unavailable in the current host build. This source reader
does not change the main drawing's open/save encoding policy.

The shared nesting implementation follows placed Attach relationships through
DWG, DXF and strict native sources. A direct Overlay is visible in its own host;
its relationship is excluded when that host is used as an external source.
Unused definitions remain inspectable without reading their files. Relative
locations resolve against their immediate owning drawing.

The active resource chain detects cycles, including references back to the open
drawing. Only the closing relationship is interrupted for the current session;
its finite prefix and healthy siblings remain visible. Shared descendants and
repeated placements are not cycles. No relationship is deleted on save.

Failed corrupt or incompatible replacement retains the previous committed body
and association. A missing descendant may retain compatible previous graphics
and is marked stale; a newly missing branch stays empty. Source bytes are session
inputs and are not embedded on native save. Browser bindings belong to the tab
and the owning source resource. Reuse of an already supplied drawing is explicit;
matching filenames, GUIDs or bytes do not establish resource identity.

New DXF/IFCCAD attachments require an opened IFCCAD host with native source
context. An ordinary CAD tab refuses them before creating the reference, because
it cannot retain the typed source association.

Native context-aware import/save retains all three formats. Stateless CAD
conversion and context-aware DWG/DXF save refuse typed DXF and IFCCAD sources,
independently of filename suffix. Convert those external drawings to DWG first;
there is no automatic source conversion, flattening or packaging.

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

## Source-format follow-up qualification

The explicit source-format pair passes the final native workspace gate, six
Document configurations and both Document WASM checks. Studio passes 171 XREF
regressions and 24 save/reader tests. Both new formats pass real desktop source
editing, selected-byte replacement, reference grip/undo/redo, automatic source-save
refresh and fresh-process offline save. The hidden browser opens the three-format
gallery, loads each source through its picker, isolates two tab bindings and
refuses unsupported CAD delivery. The matching worker retains protocol 7 and
physical DWG source identity. Frontend and packaged real CAD readback checks pass.
These are bounded local development results; upstream review and publication are
separate. The main drawing's open/save/backup encoding policy is unchanged.

## Nesting review example

The existing public three-format gallery retains its DWG/DXF/native roots.
Its native source now owns one shared detail used by an Attach and an Overlay.
Open `xref-source.ifccad` separately to see both; when referenced by the gallery,
only its attached detail propagates. Explorer supplies `xref-detail.ifccad` as an
explicit owner-scoped choice. The larger local review project additionally
covers the chain, diamond, cycle, missing source and root self-reference cases.
The public gallery is checked again with the actual release processor and embedded host; older development build records remain historical evidence.
