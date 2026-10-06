# CAD Format Explorer interface design

Design recorded on 2026-10-06. The overall direction, semantic tree, dual-format
support and representative examples have been accepted in the design discussion.
The interface incorporates the shared geometry accuracy controls and typed
spline preservation. This application design changes neither native format
contract.

The explorer should explain the contents of IFCCAD and OCDraw drawings through
an expandable, readable structure and contextual information. JSON, IFCX
relationships and OCDraw storage remain inspectable from the same selection.
Conversion evidence and CAD viewing occupy separate workspaces within one
browser application.

## Scope and foundations

Use the [OpenAEC application template](https://github.com/OpenAEC-Foundation/OpenAEC-style-book/tree/main/project-templates/Tauri%2BReact)
for the visual shell: application header, workspace tabs, contextual toolbar,
document strip, side panels and status bar. Follow its theme tokens and OpenAEC
typography. Adapt those patterns to the existing web application; introducing
a desktop runtime or wholesale frontend migration is outside this design.

Retain local browser processing through the WebAssembly worker and the existing
Open CAD Studio bridge. Files are not uploaded. IFCCAD is the first presentation
and the initial target for new DXF/DWG input; OCDraw has the same navigation
and interaction quality with its own capabilities. A native file determines
its own format. Choosing a CAD target does not introduce a native IFCCAD to
OCDraw conversion or make either model depend on the other.

The format sources of truth remain the active schemas, conformance material
and production readers. The shared UI vocabulary does not establish identical
semantic coverage. Future semantics from active branches become example and
inspection capabilities only when their contracts and implementation are available.

## Application shell and workspaces

| Workspace | Default content | Contextual actions |
| --- | --- | --- |
| IFCCAD-inhoud or OCDraw-inhoud | Readable structure and selected-item information | Bestandsboom/JSON, structure selection, native download |
| Conversie / controle | Conversion settings and located reports | Target route for CAD input, tolerance, CAD format/version, process/cancel/download |
| Tekening | Open CAD Studio in the available workspace | Compact filename/fullscreen toolbar; temporary loading/error notice and viewer restart |

The contents workspace is selected on initial startup. Opening another file,
converting it or selecting an example preserves the active workspace tab. The
contents tab name derives from the actual native result: `IFCCAD-inhoud`
or `OCDraw-inhoud`. A selected filename and format appear in the document
strip outside Drawing. No multiple-file editor is introduced; the application still processes
one selected source at a time.

Desktop fills the available browser height without a scrolling page. Header,
tabs stay visible. Contents and Conversion / checks retain the file toolbar,
document strip and status bar; Drawing replaces them with one compact filename/
fullscreen toolbar and gives the remaining height to Open CAD Studio. Long trees, JSON and reports
scroll within their own panels. The information panel is resizable. Narrow
screens reflow the tree and information panel; constrained screen height must
not make controls inaccessible. Panel width is a presentation preference,
not file metadata.

Tab changes preserve selection, expansion, inspector subtab, settings and CAD
session. Entering a tab is not permission to restart a viewer, reset a selection
or repeat a completed conversion. New source selection clears results from the
previous source. Cancellation or a newer request prevents a late result from
replacing the current drawing.

## File opening and result state

The file action offers a local file picker and the representative example
collection. Native opening follows its production reader. CAD opening exposes
the IFCCAD/OCDraw target and conversion settings before conversion starts;
IFCCAD is the initial target, and subsequent explicit choices remain visible.

During opening, a prominent cancellable loading panel shows the filename,
operation (including the CAD target) and actual reported phase: reading,
preparing, converting, validating or exporting. It remains visible across tab
changes, including Drawing, without inventing percentage progress. Cancellation
also invalidates pending file reads/example fetches; their late results cannot
replace the current source. Completion or failure removes the loading panel.
A conversion failure leaves its report available and, where
supported, the original CAD document viewable; it must not present a previous
result as the new file's contents.

Native validation is shown separately from source-reading messages, conversion
loss and exported-file readback. An invalid file may expose original JSON and
reader diagnostics when available, but its tree must be labelled unvalidated
and may not imply validated semantic records. Malformed JSON still exposes an
honest parse error rather than a fabricated structure.

## Language theme and About

Settings in the application header expose Nederlands/English and Licht/Donker.
The initial language follows the browser language when Dutch or English is
available and otherwise uses English. The initial appearance follows the
system light/dark preference; an explicit selection is persisted locally.
Changing either setting updates the current shell, inspection labels, controls
and readable status without reopening a file, rerunning conversion or resetting
selection. Stored names, JSON keys, diagnostic codes and source messages retain
their original spelling. A storage failure must not prevent the settings from
working for the current session.

About is a short header-accessible dialog describing the experimental IFCCAD
and standalone OCDraw routes, OpenAEC, local browser processing and CAD viewing
through Open CAD Studio. Include source/license links and explain that structural
validation and visual similarity do not establish lossless CAD conversion.
Use the current page's language. Dialogs have accessible titles, close actions,
Escape handling, contained keyboard focus and return focus to their trigger.

## Readable structure and stable selection

Tree rows have separate expansion and selection actions. Expansion reveals
children; selecting a row updates the information panel. Selecting a reference
navigates to its target, opens the necessary ancestors and reveals the row.
Array positions, display labels and storage rows are not identity.

Known fields receive readable labels while their original key and namespace
remain available in the inspector. Values retain their unit and source meaning.
Missing optional fields are distinct from explicit stored values. Registered
defaults may be explained as defaults, not asserted to have been authored.
Compound property values have a single expandable summary row containing their
label, centered chevron and field/item count. Expanded JSON sits beneath that
row and spans the full information-panel width instead of occupying only the
value column. Simple values and native reference links keep their compact rows.

Use one stable selection reference per actual record. IFCCAD uses complete node
paths. OCDraw uses the record domain and exact ID; an entity ID is distinct from
a layer, layout or scope ID. Preserve integer identities above JavaScript's
safe integer range through the presentation boundary. Field selection consists
of its owning record identity plus an exact field location.

Grouping rows such as Layouts are navigation conveniences and have no native
node identity of their own. Their inspector explains that distinction. The
tree follows authoritative ownership and draw order, not JSON member order,
stream family, numeric entity ID or CAD handle order.

## Integrated IFCCAD navigation

The structural selector uses **Documentstructuur / Opslagstructuur**, or
**Document structure / Storage structure** in English, for both native formats.
The document perspective follows each format's interpreted drawing document.
The storage perspective exposes IFCX node paths and source contributions for
IFCCAD, and streams/columns/tables for OCDraw. IFCX node values after composition
are explicitly distinguished from the original storage fragments in Source;
the storage index does not imply one stored fragment per node. The terms
Tekening and Knopen below describe those perspectives rather than the final
control captions.

The application opens IFCCAD Drawing overview automatically, with the Model
layout expanded. Document contents group entities by semantic type within
each layout/block by default. The user can choose Draw order instead. The
underlying ordered membership is never changed; every item shows its original
draw position. Synthetic type groups are presentation records and do not appear
as IFCX nodes or alter the original file/physical stream structure. Both formats
use the same grouping control. Group counts sit directly beside the group label,
for example `Lines · 12`, rather than at the far right of the tree row.

IFCCAD has one inspection model with two readable structural views selected
inside the contents workspace: **Document structure** (default) and **Storage structure**. Both refer
to the same production-composed IFCX nodes and preserve selection when switched.
Neither view creates a second graph or an independent drawing document.

In Tekening, organize the drawing node's contents by CAD meaning: drawing
settings, layouts and their ordered elements, layers, line patterns and shared
block definitions. Include paper/view/clip state and supported drawing metadata
where present. The grouping makes ownership understandable while each real
row keeps its IFCX path visible in the inspector.

Envelope information such as the header, imports and schemas is accessible
under file information in that same explorer. Non-CAD nodes appear under
**Overige IFCX-knopen** and non-CAD attributes appear on their actual owning
node. These labels express profile membership, not importance or unsupported
file preservation. A foreign node referencing a CAD entity does not acquire
CAD ownership.

In Knopen, list every composed node once by its complete path, with its own
attributes and relationship entries. Path grouping is navigation only: a path
prefix must not invent a `children` relationship. All actual relationship
targets are navigable references. Selecting the same CAD node in either view
shows the same properties, relationships and source contributions.

The information panel has **Eigenschappen**, **Relaties** and **Bron**:

| Inspector subtab | Content |
| --- | --- |
| Eigenschappen | Meaning, exact path, recognized CAD properties, other attributes, units and original field names |
| Relaties | Owner/draw position where applicable, outgoing and incoming links, relationship kind and source field/key |
| Bron | Effective node JSON and contributing source fragments, ordered and located within the source |

Children, inheritance entries and schema-defined attribute references remain
distinct relationships. Use typed CAD references and the recognized IFCX/schema
reference definitions; do not treat every arbitrary string as an edge. An
unknown foreign value remains inspectable as data without a fabricated typed
relationship. `inherits` is displayed as stored; the CAD profile does not use
it for blocks or appearance, and the current reader's fragment composition is
not a general inherited-value evaluator.

General IFCX references are not recursively expanded into duplicate subtrees.
Links can revisit shared targets or cycles without infinite expansion. A block
instance links to its definition and retains its own occurrence/transform
context; it must not make the shared definition appear owned by that instance.
Effective appearance depends on occurrence context and is only reported when
it has actually been resolved, with its origin explained.

The earlier mockup's separate **IFCX-graaf en bron** root is superseded by these
two views and per-node inspector subtabs. A node-and-edge drawing could be added
later if it improves relationship exploration; it is not required here.

## What composing fragments means

An IFCX `data` entry is a source contribution to the node identified by its
`path`. Several entries can have the same path. The reader composes them before
validating the CAD profile. The resulting node is the current state for that
path; it is not a new object joining several different entities.

For example, one fragment for `/cad/d1/e42` supplies `ifccad::entity` and another
fragment for that same path supplies `ifccad::geom::lineSegment`. The reader's
node at `/cad/d1/e42` contains both attributes. Source shows both contributions;
Properties and Relationships show the resulting node.

Under the current default LaterWins policy, `children`, `inherits` and
`attributes` merge by immediate key in file order. A repeated key replaces its
earlier value as a whole, including a complete geometry or appearance attribute
object. Other repeated node fields are replaced as whole values. A null
inheritance entry removes that key; null children or attributes remain markers.
The explorer uses this reader behavior and does not recursively merge nested
geometry fields, infer deletion rules or resolve external imports itself.

Use **Knoop** and **Bronfragmenten** in the ordinary UI. Explain composition
where a node has multiple contributions; avoid making “samengestelde knopen”
a separate collection that appears unrelated to the drawing tree.

Source lists each contributing fragment's position, exact original fields,
and the effective node. Replaced values remain visible in source contributions
and may be labelled as replaced only when provenance establishes that fact.
Fragment count includes identical contributions. A source index is sufficient
when accurate line/byte offsets are unavailable; never invent a source location.

## OCDraw document and storage views

OCDraw defaults to **Tekening**, a tree built from its validated logical
document: settings, ordered layouts/scopes, entities, block definitions,
layers, line patterns and supported coordinate/view/workspace state. The
format's independent document model remains authoritative for meaning.

Offer **Opslag** as a second structural view within the same contents workspace.
It exposes header and tables, scopes, streams, columns and shared pool ranges.
A stream row is linked to its logical entity; a logical entity links back to
its actual physical mapping. Scope membership defines ownership and drawing
order independently of how stream rows are arranged.

Properties of a logical element include an expandable **Opslaglocatie** entry:
stream, zero-based row, relevant column names and pool ranges. Show ranges as
offset plus count or a clearly labelled half-open range `[start, end)`. A
planar polyline's coordinates are selected from the pool by its actual offset
and count; row numbers are not vertex offsets. Inline/table-backed records show
their own physical location instead of inventing a stream.

The OCDraw inspector uses Eigenschappen, Relaties and Bron as well. Relaties
contains typed references and scope membership. Bron shows logical inspection
JSON and physical locations/values with explicit labels, rather than suggesting
that reconstructed element JSON is a contiguous original-file object. Original
file JSON remains independently available.

Use the active codec mapping to determine storage provenance. The frontend must
not reverse-engineer every stream family or duplicate production validation.
Coordinate defaults, placements, widths, bulges and other semantics come from
validated logical access. The mapping inspector explains the physical layout
without turning it into a new logical contract.

## JSON interaction

**Bestandsboom / JSON** changes the contents presentation while retaining selection.
The general JSON view is clearly labelled **Bestands-JSON** and refers to the
actual selected native source/result, rather than the application's
`presentation` report. Original native input bytes remain available separately
from the composed IFCCAD envelope and the generated native result of CAD input.

Per-item actions open explicitly labelled Knoop-JSON, Bronfragment-JSON,
Inspectie-JSON or stream/table JSON. The chosen JSON scope remains stable across
tab switches; a global switch to Bestands-JSON resets only that scope. Pretty
formatting is not advertised as preserving original whitespace or numeric
spelling. Exact original bytes continue to govern original native downloads.

## Conversion settings and tolerance

Controls are grouped by operation. CAD input has a native target; native input
has a CAD output format/version. Native download is an action in Contents.
CAD output format/version and tolerance are shared between export and the
generated Open CAD Studio preview. Preserve the existing supported CAD-version
validation rather than duplicating its list in unrelated components.

| Tolerance choice | Applied meaning |
| --- | --- |
| Standaard | Exactly 1 µm in each known coordinate unit; zero for a unitless coordinate domain |
| Exact | Zero geometric residual |
| Zelf instellen | Finite nonnegative value in millimetres, metres or each applicable drawing-coordinate domain |

The control uses the shared geometry tolerance API integrated from
`geometry-bounds`. Browser requests pass the selected limit into both native
conversion routes, generated CAD export and applicable IFCCAD CAD readback.
Keep the control visible beside
conversion settings and show the effective bound and coordinate unit/domain.
Custom units remain explicit. Unitless domains reject physical tolerances,
including when a paper medium has known physical dimensions. Paper and Model
may have different coordinate units; show each relevant resolved limit. With
drawing-coordinate tolerance, the same entered number is interpreted in each
domain's own unit and must be explained as such.

Tolerance is a hard numerical accuracy boundary. It does not alter native
validation, clipping validity, semantic coverage or an Allow/Reject loss
policy. Do not automatically increase it after failure. A failure displays
the requested limit, relevant unit/domain and available deviation interval or
proof-failure reason. Distinguish proven within-tolerance rounding from semantic
loss; preserve the converter's actual classification.

Changing settings marks affected results stale. The user applies settings to
regenerate the native result of CAD input or the generated CAD output as
appropriate. Old output may remain visible while processing, labelled as the
previous result, but cannot be downloaded as the new selection. Native opening
and the original CAD document are not affected by output-only regeneration.

The applied request identity includes source, native route, direction,
format/version, tolerance mode/value/unit, and any selected loss policy.
Preview/export caches use that complete identity. Late progress/results and
cancelled requests are ignored by generation identity. Input conversion and
output conversion retain separate evidence in the report.

## Conversion report and CAD session

The Conversion / checks workspace is the single location for conversion and
readback messages, including output generated automatically for CAD viewing.
After successful native opening or CAD conversion, prepare CAD roundtrip output
in the background with the applied CAD format/version and tolerance. Native
inspection remains usable while its actual conversion progress is shown in the
document strip and Conversion / checks, with cancellation in the latter. Its
readback and numerical evidence do not depend on entering Drawing. Drawing
shares a pending preparation or reuses its validated output; Open CAD Studio
itself is initialized only on entering Drawing. Leaving Drawing does not cancel
background preparation. A new source, changed settings or explicit cancellation
invalidates an unfinished job so stale progress/results cannot replace current
evidence. Applying settings regenerates output. Invalid input does not launch a
background native-to-CAD export.
The drawing tab contains no duplicate CAD preview messages block. A new preview
result updates the central report while retaining original reading and native
validation; viewer startup/session errors remain visible beside Open CAD Studio.
When ready, Drawing shows only its filename/fullscreen toolbar and the CAD
workspace. Explanations of original/generated documents and idle ready/validation
messages do not consume extra rows. The fullscreen button stays inside the
fullscreen surface, so exiting it remains available there.

Present reading, native validation, conversion and exported-file readback as
separate report sections with summary status and expandable details. Located
messages navigate to a native item only if its mapping is unambiguous. A CAD
source handle or skipped entity without a native counterpart retains its source
location instead of linking to an unrelated item. Preserve diagnostic codes,
reasons and any geometric intervals; readable translations complement them.

Show “not performed” when no relevant operation ran. Native validity does not
establish lossless conversion, and an exported file opening successfully does
not establish numerical or semantic identity. A viewer failure is separate
from a converter failure and does not disable valid native inspection/download.

Open CAD Studio opens original DXF/DWG and generated CAD as separate documents.
Native IFCCAD/OCDraw input only has the generated CAD document. CAD viewing
still uses actual exported DXF/DWG bytes; Open CAD Studio does not read either
native drawing model directly. Tab changes retain the session and its edits.
Replacing generated output preserves edited documents under the existing bridge
behavior; no automatic save/discard is introduced. Viewer restart is explicit.

The application does not promise cross-selection highlighting between its
tree and Open CAD Studio until the pinned bridge supplies a verified mapping.
The tree and inspector are immediately linked; graphical similarity remains
separate from conversion evidence.

## Presentation and processing boundaries

Keep the existing core/converter/storage boundaries. The intended flow is:
local source bytes and explicit request settings → worker/WebAssembly →
production reader or format-specific converter → validated native document,
diagnostics and format-specific inspection data → frontend views. CAD preview
consumes only verified exported CAD bytes and their request identity.

The application layer owns these responsibilities:

| Component | Responsibility |
| --- | --- |
| Workspace/session controller | Source, settings, request lifecycle, selection and tab state |
| IFCCAD inspection adapter | Reader-composed nodes, typed CAD meaning, relations and source contributions |
| OCDraw inspection adapter | Logical records, typed references, order and codec-derived provenance |
| Shared tree/inspector components | Readable labels, expansion, stable selection, reference navigation and JSON scopes |
| Conversion report/settings | Explicit options and operation-specific evidence |
| CAD preview controller | Complete cache identity, exported bytes and preserved viewer session |

Existing presentation objects are insufficient for all proposed features.
IFCCAD currently exposes composed graph information; original fragments and
provenance must be exposed from retained source context. OCDraw currently exposes
several raw tables/streams; the adapter must provide complete logical content
and reliable physical locations. Large integer identities and numerical evidence
must survive the existing exact presentation parser. Extend presentation DTOs
and request options, not native format schemas, to supply this data.

Retain one source snapshot and index nodes/records/references once for a result.
Render expanded/visible content rather than copying a graph under every edge.
Large lists and JSON require bounded rendering/pagination or virtualization;
UI choices must be justified by representative examples without claiming a
new performance result. Do not add a generic preservation or extension protocol
to support inspector metadata.

File-derived values render as text, never executable HTML. Source/graph links
navigate within the selected result; selecting imports does not fetch them.
Preserve existing local input limits, worker cancellation and same-origin CAD
message checks. Accessible buttons, keyboard expansion/navigation, labelled
inputs, focus management and text status accompany visual selection. UI strings
and field explanations support Dutch and English through one consistent catalog.

## Representative examples

Create new source examples under `examples/`, with a compact overview for each
format and focused examples for topics that would make one file difficult to
follow. The collection is defined by a documented coverage inventory against
each format's supported contract, not by incidental existing sample filenames.

| Example topic | Required coverage where supported |
| --- | --- |
| Drawing overview | Units, identifiers, settings, several entity families, layers and patterns |
| Geometry and placements | Every supported family, local frames, 3D orientation, curves and bounds |
| Shared and nested blocks | Definitions, base points, transforms, nested occurrences and inherited appearance |
| Layouts and viewports | Model/Paper order, units/media, camera/display/depth/clip state and layer overrides |
| Drawing state | Authored coordinate/workspace/view/plot state exposed by that format |
| IFCX relationships and fragments | Shared references, non-CAD nodes/attributes, actual fragment composition and source contributions |
| OCDraw storage | Multiple streams, columns, pool ranges and correspondence with ordered logical content |

IFCCAD examples are prominent in the chooser, and OCDraw examples remain readily
available. Each example names what it demonstrates and identifies its applicable
format capability. Do not invent feature parity or put unsupported semantics in
a purportedly valid overview. Overlapping topics may share a file if readable;
the coverage inventory remains explicit.

Geometry, text, spline/preservation and other active work are incorporated only
as their applicable contracts are integrated. Representative example creation
is part of the eventual implementation, not a prerequisite for this design
review. The mockup's line-pattern files are temporary interaction data.

Examples pass production reader and strict validation. Generated native and
CAD files also pass applicable production readback and semantic correctness
checks. Keep examples and their source recipes outside `target/`. This work
does not authorize controlled size measurements or new semantic families.

## Verification and acceptance

The implementation must demonstrate:

1. Both native formats open independently, with the correct tab name and default
   readable view. CAD input exposes its route and tolerance before conversion.
2. Ownership/order, unused definitions, supported state and exact identities
   match the validated native document. Switching structural views preserves
   identity and selection, and group rows cannot be mistaken for native nodes.
3. IFCCAD repeated fragments follow production LaterWins behavior, including
   whole-attribute replacement and null rules. Properties, relationships and
   source contributions describe the same node; foreign data stays accessible.
4. Shared/cyclic general references navigate without infinite expansion.
   `inherits` remains distinct from CAD ownership and fragment composition.
5. OCDraw entity/pool mappings are correct for nonzero offsets, multiple rows,
   interleaved draw order and sparse/high identities. Logical and physical views
   navigate to the same record without treating storage order as draw order.
6. Default, exact and custom tolerance reach both appropriate conversion
   directions and formats. Known/unitless and mixed Paper/Model domains,
   invalid values, hard failures and within-tolerance evidence are exercised.
7. Settings and source changes invalidate the right outputs/caches. Stale or
   cancelled requests cannot replace current content or enable a mismatched
   download. Preview and download use the same applied CAD-output settings.
8. Original/generated CAD documents, edited sessions, viewer restart and tab
   transitions retain the existing bridge guarantees. Conversion and viewer
   failures remain separately understandable.
9. Trees, JSON scopes, reports and controls remain usable with keyboard, light/
   dark themes and narrow screens. Desktop has no page scroll; panel scrolling
   does not overlap chrome or hide essential actions.
10. The representative example inventory accounts for every supported topic in
    each format and strict readback proves the validity of supplied examples.

Use focused frontend, worker, adapter and real-reader/converter tests, followed
by applicable browser/WASM smoke checks. Rust, schema or fixture changes require
the repository's format, clippy and workspace test gates; public Rust docs also
use the focused doc-test check. Documentation-only design recording does not
claim those implementation gates have passed. Benchmarks remain manual and
require an explicit measurement request.

## Dependencies and implementation review

The UI redesign does not change roadmap milestone order, format publication,
physical encoding, preservation or converter coverage. Tolerance integration
depends on the shared API and evidence from geometry-bounds. Later semantic
examples depend on their own completed slices; they do not delay inspection of
currently supported documents. Preserve active worktrees and unrelated local
work during implementation.

The implementation sequences presentation/request adapters,
the shell and shared inspection components, IFCCAD integration, OCDraw storage
mapping, conversion/session integration and representative examples. The
integrated graph navigation and UI are available in the local development
viewer for review. Commits, merge and publication remain separate actions
governed by repository authorization.

## Contract and implementation references

- [IFCCAD experimental contract](../../schemas/ifccad/experimental-contract-0.1.0.md)
- [IFCCAD source context](../../src/ifccad/source/graph.rs) and
  [composition](../../src/ifccad/source/composition.rs)
- [OCDraw logical contract](../../schemas/ocdraw/logical-contract-0.1.0.md),
  [JSON mapping](../../schemas/ocdraw/json-mapping-0.1.0.json) and
  [document lifecycle](../ocdraw-document-lifecycle.md)
- [Browser processing architecture](browser-processing.md)
- [Current explorer behavior](../../format-explorer/README.md)
- [Numerical tolerance background](../geometry/unit-tolerance.md)
- [Development sequencing](../../ROADMAP.md)
