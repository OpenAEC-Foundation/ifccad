# October 2026 user-research requirements review

Reviewed 2026-10-21 against the active IFCCAD checkout and
[DutchSailor's issue #17](https://github.com/OpenAEC-Foundation/ifccad/issues/17)
(updated 2026-10-21). The issue contains 17 Must and 13 Should items. It says
the priorities are proposals not yet validated with this project's users.
No study methodology, participant evidence or underlying report is attached to
the issue. Treat the list as useful workflow hypotheses; its origin does not
establish either user demand or technical completeness.

This is a documentation and implementation review, not a new interoperability
certification or benchmark. Evidence links identify current contracts and
existing tests; those tests were inspected, not rerun for this review. Existing
local SOLID/TRACE, Dimension-graphics and polyline-width roadmap changes are
included as planned work, not implemented capabilities.

[ROADMAP.md](../ROADMAP.md) owns sequencing. Active schemas/conformance, code
and tests own implemented behavior. Older issue text mentioning CBOR revision 2
does not override the current mutable revision-1 mapping. Package release
numbers, profile contracts and physical mapping revisions are separate concerns.

The Explorer has since moved to the [shared Document route](architecture/explorer-document-route.md).
This dated review’s reference-converter accuracy/preservation evidence does not
qualify that newer route; native validation and source inspection remain active.

## How to read the checklist

IFCCAD provides a drawing contract, core implementation and converter. The DWG
reader/writer belongs to opencadcodec; the embedded Open CAD Studio preview is
a separate application. Saving typed values, exchanging them through our codec,
displaying them correctly and passing an independent CAD reader are different
claims. None implies the next automatically.

**Qualified subset** means implemented within documented restrictions. **Partial**
means substantial parts exist but the combined issue item remains open. **Data
only** means relevant settings/references exist without the requested execution
or rendering. **Open** means the requested capability is not implemented or its
required evidence is unavailable. These labels do not justify checking a complete
issue item whose remaining clauses have not passed.

The following narrower capabilities can already be checked off:

- [x] Model and multiple Paper layouts, tab order, optional media and plot mappings.
- [x] Qualified viewport cameras, layer overrides and analytic/closed-path clips.
- [x] Shared and nested local blocks, ownership and authored draw order.
- [x] Native linear/aligned Dimensions, styles, overrides and bounded source relations.
- [x] Solid, explicit line-pattern and gradient Hatch data with bounded source groups.
- [x] Simple and qualified complex text/SHX line-pattern data and CAD exchange.
- [x] All 25 drawing-unit tokens/INSUNITS mappings, with separate block units.
- [x] Hard numerical conversion limits, exact mode and explicit accuracy evidence.
- [x] Located conversion-loss diagnostics and Allow/Reject policies in both directions.
- [x] Explicit entity-local partial import and a downloadable omission/error report.
- [x] Active schema/conformance fixtures, production native readback and semantic
  DXF/DWG regression tests for qualified content.

These checks apply to the qualified profiles linked below. They do not claim
automatic association updates, complete font/fill rendering, arbitrary-file
losslessness or external-application certification.

## Workstreams and feasibility

Effort is relative scope, not a duration estimate: S = documentation or a narrow
adapter; M = a bounded semantic/conversion slice; L = several contracts or
resource/rendering dependencies; XL = a new engine or substantial upstream work.
Representative source material can change an estimate.

| Lane | Place in the roadmap | Dependencies and practical acceptance |
| --- | --- | --- |
| E: evidence and recovery | Start with the current qualified profile; can accompany existing work. | Publish reader/version/feature results, fixtures and provenance. Distinguish retained, modified, omitted, preserved and unassessed content. Converter recovery does not promise recovery from a codec parse failure. |
| A: construction annotations | Keep the current SOLID/TRACE → Dimension graphics → widths → remaining kinds/selectors → joint annotation order. | Prioritize radius, diameter and angular use cases within the remaining kinds. Specify common text/Dimension/Hatch scale contexts, drawing scale lists and mm/m/in tests. Viewport associations need the later selector/update boundary. |
| R: resources and libraries | Next resource foundation; bounded adapters can proceed independently where current semantics suffice. | Start from #15 for source identity and application-owned font/symbol resolution. Add a bounded PAT importer and an explicitly selected NLCS library subset; report unmapped content. No filesystem/network resolver in the core. |
| T: additional drawing semantics | After the queued annotation work, select attributes and multileaders before tables or dynamic authoring. | #2 supplies the design boundary. Specify local identity, style references, ownership and exchange independently for each family. Layer states/groups/filters are a separate bounded slice, not merely more layer flags. |
| P: plot fidelity | Design native color/CTB/STB use under #16; qualify output after resource and rendering dependencies. | Stored media/mappings/margins are the starting point. PDF needs correct fonts, pattern/fill evaluation, Dimension display, style application and physical output tests. Core data and application rendering remain separate. |
| X: external/dynamic content | Later, use demonstrated project exchange cases. | Xrefs build on R but additionally need graph ownership, origin, cycles, reload/bind and archive rules. Dynamic preservation needs an approved whole-dependency capture/restore design and codec capability; the spline pilot is not that design. |
| F: large-file evidence | Define workloads first; run measurements only on explicit request. | #6 covers time/memory evidence; #3 covers later indexed access. Record hardware, entities, resources, retained content and separate parsing/conversion/display costs before selecting optimizations or thresholds. |
| H: host application | Track as application work or a separately approved IFCCAD integration, without changing the core's purpose. | PDF/batch UI, dynamic authoring, command aliases, DWT selection, desktop packaging and MCP need a concrete host, owner and user workflow. Preview integration is not a promise of these features. |

## Must items

IDs follow the issue's original order, so every requirement remains traceable.

| ID | Requirement | Current support and remaining gap | Next step / owner / effort |
| --- | --- | --- | --- |
| M01 | DWGs open without repair in main readers; AC1018/1024/1027/1032 | **Partial.** Export selects CAD versions and performs codec readback/version checks. Existing semantic qualification is chiefly AC1032. Our own writer/reader pair does not establish repair-free opening in other readers or all versions. [Exchange evidence](#exchange-and-conformance-evidence). | E: name independent readers and versions; record per-fixture opening/audit results and route defects to the codec or converter. M for the matrix; wider qualification/repairs L. |
| M02 | An unreadable record never blocks a document; recovery report | **Partial.** Explicit partial import skips eligible entity/occurrence conversion failures with original evidence and strict retained-output validation. Codec parse failures, unsafe shared structure and failed final validation can still block opening. [Recovery evidence](#loss-and-recovery-evidence). | E: separate codec recovery from converter recovery. Replace the absolute “never” with safely recoverable local failures and an explicit fatal boundary. Codec work L. |
| M03 | Paper layouts/viewports, including overrides and clipping, roundtrip | **Qualified subset.** Multiple sheets, cameras, freezing/color/opacity/pattern/weight overrides and circle/full-ellipse/closed straight-or-bulged clips have real DXF/DWG tests. Spline clips, some current-context relationships and broader metadata remain limited; no universal reader/version claim. [Layout evidence](#layouts-and-presentation-evidence). | E: credit the implemented subset; expand qualification only for specific missing source cases. S documentation, M per selected gap. |
| M04 | Scale-correct PDF, lineweight/linetype, ISO sheets and 1:1/50/100 | **Data only.** Media, fixed output ratios, weights, patterns and Paper scaling exist; a house example stores A3/1:100 intent. There is no qualified IFCCAD PDF plot pipeline proving those outputs. [Plot evidence](#layouts-and-presentation-evidence). | P/H: qualify rendering and PDF output with measured page/line geometry and visual references after resource/style support. L–XL. |
| M05 | CTB/STB load/apply, including colors 251–255 | **Data only.** Plot-style mode/name/options and qualified ACI identities exist. References do not retain table contents or prove application of indices 251–255. The codec's parsing/model basis is not an IFCCAD plot contract. [Color evidence](#layouts-and-presentation-evidence). | P, #16: decide table authority/resources and named versus color-dependent behavior; explicit 251–255 tests. Refuse unsupported STB output. L. |
| M06 | DIMSTYLE for radius/diameter/linear/angular/aligned, associative in viewports | **Partial.** Linear/aligned definitions, styles, sparse overrides, formatting and bounded same-owner relations exist. Other definition kinds, complete pictures and trans-space/viewport update behavior remain later work. A kind name in the style enum is not an implemented definition. [Dimension evidence](#annotation-evidence). | A: keep the existing graphics/width prerequisites, then remaining kinds and later selector/update work (#13). L across slices. |
| M07 | Annotation scales for text/Dimensions/Hatch in mm/m/in; drawing scale list | **Open.** Text/style scale, Dimension formatting factors and Paper linetype scaling are different features. No shared annotative-context/scale-list contract is implemented. [Annotation evidence](#annotation-evidence). | A: joint context design and file exchange, with each unit and multiple viewport scales. Assess the paused Hatch boundary explicitly. L. |
| M08 | Xref load/reload/bind, paths and complete archives | **Open.** Local shared blocks exist; external/XREF definitions and referring inserts are omitted with loss. IFCX composition is not a CAD xref lifecycle. [Block evidence](#blocks-and-preservation-evidence). | X after R/#15: resource origins plus referenced-document graph and packaging rules. L–XL. |
| M09 | Display SHX/TrueType; report missing fonts and substitutes | **Data only.** Text styles/font requests and SHX symbol sources are retained. Native geometry evidence does not certify glyphs. An embedded preview may resolve its own assets; that is not a common font/substitution contract. [Resource evidence](#resources-and-pattern-evidence). | R/H, #15: application-owned lookup, explicit resolution/substitution report and glyph/layout qualification. L. |
| M10 | Preserve blocks, nested blocks, attributes and multileaders | **Partial.** Local blocks, nesting, shared definitions, transforms and ordering exist. Attributes and multileaders have no native IFCCAD entity profile; unsupported source content remains diagnosed. [Block evidence](#blocks-and-preservation-evidence). | T/#2: check off blocks separately, then design attributes and multileaders as individual slices. M–L each. |
| M11 | Read/save dynamic blocks intact without editing | **Open.** Definitions with directly owned dynamic objects and their inserts are omitted. Opaque spline preservation does not capture a dynamic block's dependency graph. [Preservation evidence](#blocks-and-preservation-evidence). | X/codec: assess approved preservation of the complete graph, restoration eligibility and edits, including upstream #91. XL/high uncertainty. |
| M12 | ANSI/ISO/custom PAT/solid/gradient Hatch, correct and associative | **Partial.** Three native fills, explicit pattern families, stored association state/groups and qualified CAD tests exist. Standalone PAT-library import, evaluated fill certification, spline boundaries and automatic updates are separate gaps. A named pattern is not proof of complete ANSI/ISO library coverage. [Hatch evidence](#hatch-evidence). | R: bounded PAT adapter; A/#13: explicit update scenario; P/H: rendered-fill qualification. M for a bounded importer; L for updates/rendering. |
| M13 | Units/INSUNITS and large coordinates without precision loss | **Qualified subset.** All 25 unit codes and block units map; binary64 source projection, whole-curve and nested-occurrence accuracy are checked. Exact mode can reject required rounding; default mode accepts certified residuals within a hard limit. This is not arbitrary-magnitude zero-loss arithmetic. [Numeric evidence](#numeric-and-performance-evidence). | E: document the guarantee and application coordinate budget; add representative regressions when faults occur. #11/#12/#18 remain separate, evidence-driven work. S/M. |
| M14 | 15–100 MB / 200,000+ entities, reasonable time/memory; ≤20× proposal | **Open evidence.** Existing scoped reports do not establish these workloads or this memory limit on current code. Compressed file size alone poorly characterizes decoded content, sharing and resources. [Performance evidence](#numeric-and-performance-evidence). | F/#6: select representative drawings and hardware; separate cold/warm parse, conversion, strict readback and display. Only then consider #3. Measurement needs a separate explicit request. M investigation, implementation unknown. |
| M15 | Every roundtrip reports kept/changed/dropped content | **Partial.** Both converter directions provide located losses and ID mappings; Explorer separates input/output, preservation, accuracy and CAD readback. There is no complete per-item retained/changed/dropped ledger across every codec/reader boundary; data not exposed by the codec are unassessed. [Loss evidence](#loss-and-recovery-evidence). | E: build a consumable coverage summary from existing results, with separate preserved/unassessed states and failed-stage evidence. M. |
| M16 | Public per-release conformance matrix, reader and result | **Partial.** Public native fixtures, converter inventories, audits and real exchange tests exist. A versioned independent-reader/release matrix is not yet published. [Conformance evidence](#exchange-and-conformance-evidence). | E: combine M01/S11, record exact candidate/dependency/patch and distinguish not tested from failed. First publish a development snapshot; attach release evidence when releases exist. S–M. |
| M17 | Fixture for every DWG fix; unchanged sections byte-identical; clean licenses | **Partial; one clause needs a different owner.** Focused regressions and provenance fixtures exist, but this review does not establish universal bug/fixture/license coverage. Conversion constructs a fresh CAD document, rebinding handles and rebuilding storage. It cannot promise original DWG section bytes. [Preservation evidence](#blocks-and-preservation-evidence). | E: require reproducible regressions and explicit redistribution provenance. Codec-only source-preserving editing, if wanted, needs its own raw-section/dependency design; do not make byte identity an IFCCAD conversion requirement. S–M policy, XL raw editing. |

## Should items

| ID | Requirement | Current support and remaining gap | Next step / owner / effort |
| --- | --- | --- | --- |
| S01 | Author dynamic blocks; correctly display imported ones | **Open.** No native dynamic constraint/action model or authoring engine. Source retention in M11 would still not supply evaluation or authoring. | X/H after a demonstrated preservation/display workflow; separate editor project. XL. |
| S02 | Printable margins, full bleed and batch publishing | **Data only.** Media/printable areas, rotations and output mappings exist. PDF/driver output, full-bleed device qualification and batch publishing are not delivered by retaining them. [Plot evidence](#layouts-and-presentation-evidence). | P/H after M04/M05: physical page/driver tests, then batch orchestration. L. |
| S03 | Tables, merged cells and table styles | **Open.** No native table entity/style contract. | T/#2 after attributes/multileaders: define cells/spans/styles, rich content and exchange; font layout is a separate dependency. L. |
| S04 | Complex SHX/text linetypes with upright rotation | **Partial.** Qualified relative/absolute markers, resources and actual AC1032 exchange exist. Upright is unavailable on our current pin, but upstream #108 was completed on 2026-10-20 in `8d995b8`. Rendered glyphs remain unassessed. [Pattern evidence](#resources-and-pattern-evidence). | Audit/adopt the upstream change, then add native mode/validation/mapping and exchange regressions. Qualify the older-DXF restriction explicitly instead of silently emitting relative rotation. R handles assets. M; no longer waiting for an upstream implementation. |
| S05 | Import NLCS 5.0.x layers/patterns/hatches/symbols/plot settings | **Open.** Several target primitives exist, but no qualified NLCS importer or library mapping. No exact 5.0.x source library/version is selected. | R/T/P: select a redistributable reference library; map supported fields first and report gaps. Avoid inventing a second drawing model. M for a restricted subset, L for full scope. |
| S06 | Migrate scripts, PGP aliases and shortcuts | **Open; application-owned.** IFCCAD has no interactive command environment or compatibility interpreter. | H: establish target host, source command set and user scripts before an adapter. M–L; no current core milestone. |
| S07 | Choose/configure default DWT templates | **Open; application-owned.** Native styles/blocks/layouts can represent part of a template, but DWT recognition, full content retention and new-file/default-template UI are not qualified. | H with converter/codec support for a selected template profile; diagnose unsupported content. M–L. |
| S08 | macOS/Linux parity, shortcuts/layers/signing/Flatpak | **Open; application-owned.** Native Rust and browser/WASM checks are useful foundations, not desktop feature or packaging certification. | H in the intended desktop application/release process; specify supported platforms and workflows. L. |
| S09 | Same fonts on web and desktop | **Open evidence.** No shared, versioned font-resolution/shaping/substitution policy with cross-host results. | R/H after M09: identical asset identity and layout rules plus cross-platform visual/layout fixtures. L. |
| S10 | Small automation/MCP API with dry run | **Partial foundation.** Typed construction/conversion APIs, CLI diagnosis and WASM entry points exist. No MCP service, proposed-edit transaction model or dry-run command surface is implemented. | H: select one validate/convert or batch-edit use case; keep the service outside the core and specify effects before adding tools. M, larger for editing. |
| S11 | Compatibility page for each DWG version | **Partial.** Version choices and documented family restrictions exist, including AC1032-only Hatch qualification. A version selector is not a compatibility matrix. | E: same artifact as M01/M16; show feature, route, target version and independent reader result separately. S–M. |
| S12 | Layer states, groups and filters | **Partial foundation.** Layers retain visibility, freeze, lock, plottability and viewport overrides. Named saved states, groups and filter expressions are not native contracts. [Presentation evidence](#layouts-and-presentation-evidence). | T: bounded storage/validation/exchange for a selected state/filter use case; host UI separately. M–L. |
| S13 | Weekly release after green conformance; 15 MB opens in ≤15 s | **Partial foundation.** CI runs workspace/frontend/WASM checks and deployment smoke. There is no weekly release policy, independent-reader gate or qualified 15-second reference workload. Existing measurement policy excludes automatic benchmark gates. | E: repeatable correctness/conformance release evidence. F: manual, explicitly requested performance qualification. Review package calendar versioning in #19 separately; do not change profile/CBOR versions or impose timing gates here. M process; performance unknown. |

## Evidence sources

### Exchange and conformance evidence

- [Active contract](../schemas/ifccad/experimental-contract-0.1.0.md),
  [candidate cases](../conformance/next/ifccad/README.md) and
  [production-reader conformance test](../tests/ifccad_conformance_next.rs).
- [Converter coverage and policy](../crates/ifccad-convert/README.md),
  [from-CAD inventory](../crates/ifccad-convert/docs/FROM-CAD-COVERAGE.md),
  [to-CAD inventory](../crates/ifccad-convert/docs/TO-CAD-COVERAGE.md),
  [current dependency audit](geometry/opencadcodec-update-2026-10-16.md).
- [CAD output/readback route](../crates/viewer/src/document_route.rs),
  [version choices](../format-explorer/src/cad-formats.mjs) and
  [CI/deployment checks](../.github/workflows/format-explorer.yml).

### Layouts and presentation evidence

- [Media/plot mappings/margins](layout-output.md),
  [presentation and viewport overrides](presentation.md),
  [workspace restrictions](workspace-state.md).
- [Plot exchange tests](../crates/ifccad-convert/tests/plot_exchange.rs),
  [viewport exchange tests](../crates/ifccad-convert/tests/viewport_exchange.rs),
  [override exchange tests](../crates/ifccad-convert/tests/viewport_presentation.rs).
- [Color and plot-style design issue #16](https://github.com/OpenAEC-Foundation/ifccad/issues/16).

### Annotation evidence

- [Dimensions and later kinds/selectors/annotation](dimension.md),
  [implemented definition variants](../src/dimension/values.rs),
  [Dimension exchange tests](../crates/ifccad-convert/tests/dimension_exchange.rs),
  [text profile](text.md).
- [Future associative updates #13](https://github.com/OpenAEC-Foundation/ifccad/issues/13).

### Hatch evidence

- [Fill/profile/association boundaries](hatch.md),
  [Hatch exchange tests](../crates/ifccad-convert/tests/hatch_exchange.rs),
  [association tests](../crates/ifccad-convert/tests/hatch_associations.rs),
  [gradient exchange](../crates/ifccad-convert/tests/hatch_gradient_exchange.rs).

### Resources and pattern evidence

- [Text and symbolic font requests](text.md),
  [complex patterns/SHX/upright restrictions](line-patterns.md),
  [complex-pattern exchange tests](../crates/ifccad-convert/tests/complex_line_pattern_exchange.rs).
- [Source identity/resolution #15](https://github.com/OpenAEC-Foundation/ifccad/issues/15),
  [codec upright #108](https://github.com/HakanSeven12/opencadcodec/issues/108),
  [upstream upright implementation](https://github.com/HakanSeven12/opencadcodec/commit/8d995b85f098db31448d6c6a12e4fdb0446007fb).
  The maintainer confirms DXF output carries the bit from R2013 onward; older
  DXF versions fall back to relative rotation in that implementation. Downstream
  adoption must assess/diagnose this target restriction, not claim all-version parity.

### Blocks and preservation evidence

- [Native entity variants](../src/ifccad/logical/model.rs),
  [block regression tests](../crates/ifccad-convert/tests/blocks.rs),
  [source omissions](../crates/ifccad-convert/docs/FROM-CAD-COVERAGE.md).
- [Preservation boundary and raw-record exclusions](preservation.md),
  [IFCCAD lifecycle/source writeback boundary](experiments/ifccad-document-lifecycle.md),
  [additional typed families #2](https://github.com/OpenAEC-Foundation/ifccad/issues/2),
  [upstream preservation #91](https://github.com/HakanSeven12/opencadcodec/issues/91).

### Loss and recovery evidence

- [Converter outcomes and policies](../crates/ifccad-convert/README.md#conversion-diagnosis),
  [report-only diagnosis](conversion-diagnostics.md),
  [partial-import tests](../crates/ifccad-convert/tests/partial_open.rs),
  [loss-policy tests](../crates/ifccad-convert/tests/loss_policy.rs).
- [Explorer recovery/reporting](../format-explorer/README.md),
  [separate input/output report](../format-explorer/src/inspection-report.mjs),
  [partial-open UI tests](../format-explorer/tests/partial-open.test.mjs).

### Numeric and performance evidence

- [Shared accuracy contract](geometry/shared-geometry.md),
  [unit implementation](../crates/ifccad-convert/src/units.rs),
  [all-unit conversion regression](../crates/ifccad-convert/tests/conversion.rs),
  [per-domain numeric policy tests](../crates/ifccad-convert/tests/geometry_policy.rs).
- [Scoped five-case CBOR results](benchmarks/ifccad-cbor-results-summary-v1.md),
  [placement workload evidence](benchmarks/placement-preparation-v1.md),
  [measurement policy/reproduction](benchmarks/ocdraw-size-exchange-v1.md).
  These retain their original scope and are not current large-DWG acceptance results.
- [Measurement #6](https://github.com/OpenAEC-Foundation/ifccad/issues/6),
  [partial access #3](https://github.com/OpenAEC-Foundation/ifccad/issues/3),
  [placement #11](https://github.com/OpenAEC-Foundation/ifccad/issues/11),
  [bulge proof #12](https://github.com/OpenAEC-Foundation/ifccad/issues/12),
  [Hatch rounding #18](https://github.com/OpenAEC-Foundation/ifccad/issues/18).

## Decisions and unresolved validation

Keep the existing architectural sequence and make construction-document fidelity
the next semantic focus. Credit implemented subsets before adding work. Start
public qualification/reporting with current content; it does not require a new
format or completion of every application feature.

Do not adopt universal recovery, byte-identical regenerated DWG sections,
unvalidated memory/time targets or weekly performance gates as core contracts.
Do not infer native dynamic-block preservation, general external assets or a
new extension protocol from existing spline storage or IFCX composition.

The market-priority assumptions still need examples from users: a typical
deliverable, its receiving application/version, the failure that blocks delivery,
and licensed source/expected-output material. Use that evidence to select the
first bounded slice within each lane. No user-study rerun, performance run,
external-reader session, GitHub issue edit, commit or release is part of this review.
