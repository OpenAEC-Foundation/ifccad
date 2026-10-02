# opencadcodec dependency qualification, 2026-10-02

Both converters select unmodified opencadcodec 0.5.5 at
`d96e3fa2fe5acbeac966f1db4c01142618bf9c79`, replacing acadrust 0.5.5 at
`5b682ed66ea2c89be8142c8dd83d83774fc3de08`. Cargo manifests pin the revision;
the locally generated Cargo.lock resolves it. The existing `cadcodec` dependency
alias and reexports remain available. This update changes neither native format,
schemas, identity allocation nor the CAD editing-session boundary.
Upstream now boxes the wide `EntityType` variants Helix, MultiLeader, Surface,
Table and Extended. None is in either supported geometry subset; existing typed
matches still classify them. Callers constructing these reexported upstream
variants must supply their boxed payloads.

The [upstream comparison](https://github.com/HakanSeven12/opencadcodec/compare/5b682ed66ea2c89be8142c8dd83d83774fc3de08...d96e3fa2fe5acbeac966f1db4c01142618bf9c79)
contains 71 commits and 167 changed files. Changed document, entity, table,
object and inventory declarations were checked against both direction-specific
coverage contracts. Exhaustive inventory matches are still insufficient for
field coverage: [upstream #50](https://github.com/HakanSeven12/opencadcodec/issues/50)
remains open. This is a bounded manual audit of the pinned exposed model.

| Changed public surface | OCDraw treatment | IFCX-CAD treatment |
| --- | --- | --- |
| Header `dwf_frame`, `dgn_frame`, universal creation/update Julian dates | Nondefault values remain header loss through typed residual equality | Nondefault values remain located header loss through serde residual comparison |
| `Viewport.off_screen` | Explicit partial-loss diagnostic on authored viewports and overall paper canvases; Reject prevents output | Typed scaffold equality includes the flag; a changed viewport is omitted with diagnostics and Reject prevents output |
| Hatch/MPOLYGON invalid boundary loops replacing a count | Whole entity family remains unsupported and diagnosed | Whole entity family remains unsupported and diagnosed |
| Underlay `unloaded`, PointCloudEx hidden scans/regions replacing opaque counters | Whole entity families remain unsupported and diagnosed | Whole entity families remain unsupported and diagnosed |
| Point-cloud color ramp ID, name and typed colors/visibility | Non-scaffold class object remains diagnosed | Non-scaffold class object remains diagnosed |
| Dynamic-block action parameter IDs, offset multiplier/flags, lookup columns and polar-stretch bindings/codes/options | Canonical dynamic objects remain diagnosed; INSERT behavior is not flattened into ordinary instances | Canonical dynamic objects and affected definitions remain diagnosed/omitted |
| Solid-history evaluation-graph link, sweep DWG vector, loft embedded path/options | Solid entities and history objects remain unsupported and diagnosed; graph helper adds no independent supported drawing component | Same unsupported entity/object boundary |
| FIELD object XDATA | Canonical FIELD object remains diagnosed; its duplicate `fields` side view grants no preservation | Canonical FIELD object remains diagnosed; side views grant no preservation |
| Generic non-entity object XDATA | Newly decoded DXF records and DWG EED traverse `NonEntityExtendedData`; unsupported payloads receive inventory loss, without duplicate counting | Same inventory category receives explicit XDATA loss |
| ACIS parser/body helpers and internal raw/original-object snapshots | Unsupported ACIS entity/object boundary remains unchanged; private passthrough state is outside the converter guarantee | Same boundary |

Constructor default changes, formatting/documentation, geometric helpers and
codec serialization changes do not by themselves add native coverage. No new
top-level inventory category or supported geometric field other than viewport
off-screen state needs new classification. Changed data in unsupported families
continues to be diagnosed at its whole entity/object boundary. Header tests cover
each new field separately; XDATA tests exercise actual DXF and DWG readback and
both loss policies. Production native readers check all generated native output.

The Model layout's `plot_flags.model_type` is derived role bookkeeping: DXF's
writer sets it from the name. IFCX-CAD now normalizes just that bit after structural
ownership checks; other authored plot flags remain loss. The current DXF reader
also parses padded plot integer codes correctly. OCDraw therefore uses typed
layout fields directly and no longer replays retained raw codes over subsequent
edits. A read/edit/export regression verifies the changed rotation survives.

DWG nonzero block base points (#52), anonymous block names (#55), and DXF block
descriptions (#49) are resolved. The old anonymous-name repair is removed;
conflicting present names remain fatal with or without a name collision. An
unchanged synthetic AutoCAD fixture checks actual anonymous-name decoding and
strict OCDraw conversion. Nested/shared nonzero-base block exchanges now pass
for both native models. See the [block boundary qualification](block-cad-boundary.md)
for remaining extra-paper marker ownership, scale clamping and numerical limits.

The primitive [size/exchange experiment](../benchmarks/ocdraw-size-exchange-v1.md)
was rerun with this pin: all 12 cases and two fresh generations passed strict
readback, semantic exchange and repeatability. Native byte counts/hashes remain
unchanged; actual DXF and DWG sizes/hashes changed as recorded in the report. Historical
package, practice-file and placement measurements retain their original pins;
they are not relabelled as evidence for the new dependency. This update makes no
new claim about unsupported semantic families or typical CAD file sizes.
