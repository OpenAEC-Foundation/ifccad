# Block attributes — development contract

Unpublished development profile 0.6.0 / CBOR6. Focused native and paired conversion checks pass. Review files precede OCS, ATTSYNC and final qualification. Published 2026.45.1 remains profile 0.5.0 / CBOR5.

## Native development contract

### Identiteit, ownership en volgorde

- Nieuwe entitykind `AttributeDefinition(IfccadAttributeDefinition)`; gewone
  entity-identiteit `/cad/dN/eI`, common layer/appearance/visible/pattern-scale.
  Bewaart `tag`, `prompt`, `flags`, `lockPosition`, `fieldLength`, `text` en
  optioneel `mText`, plus eigen optionele `annotation` zoals hieronder.
  `text.content` is de default/fallback; geen tweede tegenstrijdig value-veld.
- `BlockInstance` krijgt `attributes: Vec<IfccadBlockAttribute>`, standaard leeg.
  `IfccadBlockAttribute` bewaart `id`, `visible`, `layer_id`, `appearance`,
  `line_pattern_scale` en een `IfccadAttributeValue` met dezelfde inhoudsvelden
  als de definitie, zonder prompt. De tekstcontent is de concrete waarde.
  Deze aparte public naam onderscheidt INSERT-children van algemene IFCX-attributen.
- Beide gebruiken het bestaande globale entity-ID-domein en `nextEntityId`,
  inclusief nul en IDs boven 2^53. Geen nieuwe counter en geen source handles
  als native identiteit. Embedded MText heeft geen afzonderlijke entity-ID.
- ATTDEF blijft een geordende entity in block.entities, Model of Paper. Een
  blockdefinitie bevat de herbruikbare definitie, nooit instantiechildren.
  Een waarde heeft precies één INSERT-owner en kan geen vrije layoutentity,
  gedeeld child of child van een ander attribuut zijn.
- INSERT blijft één positie in zijn scope. De childvector is onafhankelijk
  geordend en wordt niet gesorteerd op tag of definitievolgorde. Geen extra
  scopepositie of kopie van dezelfde waarde. Verwijderen van INSERT verwijdert
  zijn children; herordenen/wijzigen bewaart IDs.
- IFCX: `ifccad::attributeDefinition` als nieuwe payloadrol; childnodes krijgen
  `ifccad::entity` en `ifccad::blockAttribute`. INSERT heeft eigen numerieke
  `children`-keys `0..n-1`. De nieuwe ownerrol is expliciet gesloten tot
  attribuutwaarden; layout/block numeric children blijven hun gewone rol houden.
  Foreign named children confereren geen CAD-ownership.

### Tags en koppeling

Tags blijven exacte strings: geen uppercase, trim, deduplicatie of renummering.
Ze zijn niet leeg en bevatten geen ASCII-spatie of besturingstekens. Unicode
blijft toegestaan. Lege waarden en lege prompts zijn geldig.

Tag is een bronlabel voor correspondence binnen een INSERT en zijn definitie,
geen unieke native sleutel. Dubbele tags, veranderde tags, ontbrekende waarden
en waarden zonder actuele ATTDEF blijven op zichzelf geldig. Native IO maakt
geen ontbrekende waarden uit defaults en doet geen ATTSYNC. Een applicatie kan
kandidaten tonen, maar mag bij ambiguïteit niet de eerste nemen.

Een instantieattribuut kan een
optionele expliciete definitieverwijzing bewaren wanneer de bron die betrouwbaar
levert. De verwijzing moet naar een ATTDEF in de direct gerefereerde blockdefinitie
resolven. Afwezigheid is geldig; tag of vectorpositie verzint geen sterke link.
Een non-null in-memory `attdef_handle` wordt gecontroleerd en waar geldig gemapt.
Kan fysiek CAD-transport die binding niet behouden, dan volgt een gerichte
diagnose; dat beperkt het native model niet tot de transportmogelijkheden.
De field-/relatiedialect en codecdekking worden in de eerste planstap tegen de
geselecteerde brongevallen gecontroleerd, vóór productcode daarvoor start.

Veldnaam: `definition_id: Option<u64>` op
`IfccadAttributeValue`, JSON `definition` als volledige ATTDEF-entitypath.
Bij expliciete binding is een afwijkende tag toegestaan: een opgeslagen rename
zonder ATTSYNC is geen corrupte relatie. De doelrol en directe blockdefinitie
moeten wel kloppen. Verwijderen van een definitie vereist een bewuste keuze om
de binding te verwijderen/te veranderen; IO wist de relatie niet stilzwijgend.

Secundaire attributen/definities en hun verwijzingen worden per concrete
bronconstructie onderzocht, inclusief eventuele MText-/annotationafhankelijkheid,
ownership, volgorde, referentie-integriteit en physical readback. Wat nodig is
voor de geselecteerde betrouwbare attributen wordt getypeerd meegenomen, zo
nodig met een begrensde codecuitbreiding. Een onbekende relatie krijgt geen
automatische betekenis en geen willekeurige raw-body als vervanging.

### Tekst, MText en resources

`text: IfccadText` is altijd aanwezig: buitenste stijl, de bestaande TextLayout,
rotation, mirrors, shear, signed thickness en literal runs. `mText` is
optioneel; aanwezigheid kiest de multilinevariant en de actieve waarde/opmaak.
De buitenste Text-record blijft onafhankelijk bewaard. Geen gelijkheidseis
tussen fallback en actieve MText, ook niet bij een lege actieve waarde.

Dit bewaart de native Text-betekenis, niet alle ongebruikte ruwe CAD-ankers of
berekende glyphmetingen. De gewone Text-mapping kiest de actieve insertion/
alignment volgens de layout; niet-vertegenwoordigde bronwaarden krijgen dezelfde
expliciete normalisatie-/verliesdiagnostiek, geen exacte bronrecordclaim.

`mText: IfccadAttributeMText` bevat bestaande `IfccadMText` plus eigen
`visible`, `layer_id`, `appearance` en `line_pattern_scale`, zonder eigen ID of
owner. Die common-gegevens zijn onafhankelijke embedded properties, geen
tweede drawable of geërfde kopie van de buitenste entity. Bestaande Text/MText-
stijl-IDs en exacte FontRequest/SHX-broncontracten worden hergebruikt; geen
resolutie of nieuwe fontregistry. Baseline, wrapping, columns, backgrounds,
paragraph/runformaten, stacks en flow houden hun bestaande native betekenis.

De geneste Text/MText-waarden hebben `annotation = None`: attribuutannotatie
staat eenmaal op het attribuut, om conflicterende dubbelregistratie te vermijden.
Gedeelde text-validatie wordt via een geleende waardeview aangeroepen, met de
attribuutdiagnosticpath. Geen tweede semantische tekstvalidator.

`flags` bewaart afzonderlijk `invisible`, `constant`, `verify`, `preset` (false
defaults). `lockPosition` is apart (false). `fieldLength: i32` (default 0)
bewaart de historische advisory waarde, geen maximale waarde-/tekstlengte.
De codecgrens controleert de smallere signed-16-bit CAD-range bij export.
Constante en lock flags zijn opgeslagen intent, geen verbod om het native
datamodel bewust te bewerken. Verify/preset bouwen geen invoerwizard.

De codecbooleans achter niet-standaard flagbits 16/128 worden niet blind als de
normatieve position-lock of volledige annotatie geïnterpreteerd. De aparte lock,
AcadAnnotative-data en echte contexten zijn de onderzochte bronnen. Niet
vertegenwoordigbare of conflicterende bitinformatie wordt gelokaliseerd gemeld;
structureel tegenstrijdige actieve annotationdata wordt afgewezen.

### Coördinaten, presentatie en bounds

ATTDEF-plaatsing is in de eigen scope: drawing units in Model/block, Paper-
coördinaten in Paper. Constant defaults lopen via het bestaande blocktransform,
inclusief basepoint, signed/nonuniform scale en outer nesting.

ATTRIB heeft **reeds geplaatste tekst in de scope van zijn INSERT**. Het is
logisch van INSERT, maar de INSERT-transform wordt niet nogmaals op zijn tekst
toegepast. Een nested INSERT-child staat in de omringende blockdefinitie;
alleen de nog buitenliggende occurrences transformeren dat samen verder.
CAD OCS wordt met dezelfde text-frameconversie als gewone Text verwerkt; MText
houdt de eigen WCS-richting/conventie. Geen gemiddelde schaal of decomposition
van een samengestelde affine transform. Unit conversion gebeurt alleen waar
het bestaande scope-/INSERT-contract dat expliciet voorschrijft.

Niet-constante definities zijn templates: in een gewone occurrence worden ze
niet als defaulttekst getekend. Constante definities leveren hun opgeslagen
default zonder automatisch ATTRIB te creëren. In een definitie-/authoringview
kan een niet-constante ATTDEF zijn tag tonen. Losse layout-ATTDEF blijft
authoringinhoud. Expliciet opgeslagen ATTRIBs worden niet op tag weggedrukt;
ook uitzonderlijke constante childwaarden blijven opgeslagen inhoud.

Document krijgt `attributeVisibility: Normal | HideAll | ShowAll`, default
Normal, voor ATTMODE. Dit behoudt de attribuut-invisible flag onafhankelijk
van common entity-visible: ShowAll overrulet de attribuutflag, niet willekeurig
de gewone layer-/entityvisibility. ATTREQ/ATTDIA blijven toepassingsinvoerbeleid
en krijgen een diagnose waar niet-standaard broninstellingen verloren gaan.

Bounds hergebruiken de text-estimates/proven subsets en rapporteren per
occurrence. Maak template/default en concrete children expliciet in de bestaande
traversal, zodat teksten niet ontbreken of dubbel bijdragen. Zichtbaarheid
verwijdert geen geometrische validatieplicht. Geen glyphcertificaat, refresh of
fontresolutie uit een box. Annotatieve display/bounds behouden de bestaande
unassessed-status waar geen volledige presentationselectie is gekwalificeerd.

### Annotatieve attributen, inclusief MText

Eigen `annotation: Option<IfccadAnnotation<IfccadAttributeAnnotationContext>>`
op definitie en child, met bestaande drawing-local schaalregistry, independent
reference/default, orientation en Enabled/Disabled modes. Geen ByDefinition-
modus op een attribuut zonder sterke definitielink. Block-/style-intent en
contexten blijven zelfstandige opgeslagen waarden; geen genereren op load/save.

De nieuwe gesloten presentationvariant is:

- `Text { text: IfccadTextAnnotationContext }` voor gewone Text-contextdata.
- `MText { text: IfccadTextAnnotationContext, mTextEnabled: bool,
  embedded: Option<IfccadAttributeEmbeddedContext> }` voor de codec-
  MTextAttribute-context. Niet terugbrengen tot een gewone MText-context.
- `IfccadAttributeEmbeddedContext { scale_id, is_default,
  presentation: IfccadMTextAnnotationContext }`: de embedded context heeft een
  eigen schaallink/default; gelijkheid met de buitenste schaal is niet vereist.
  Disabled plus opgeslagen embedded data mag blijven bestaan. Ontbrekende
  presentation blijft membership-only, zonder fictieve layout.

Hergebruik bestaande Text/MText-contextvalidatie, inclusief eindige waarden,
columns en scalaire ranges. Alle buitenste en embedded scale refs moeten in
dezelfde drawing resolven. Type/contextvorm moet bij de attribuutvariant passen.
Transportclass versions, handles, reactors en extension dictionaries zijn geen
vrije native contextbody: onverwachte/semantische restdata krijgt concrete
diagnostiek of rejection, geen raw replay. Ordinary annotative/nested blocks
blijven geselecteerd; eigen attribuutplaatsing is geen tweede toepassing van
de blockpresentatie. Fixtures testen beide lagen onafhankelijk en gecombineerd.

## Gedeelde validatie, IFCX en CBOR

Breid de bestaande logical access/owner-index uit met INSERT-children, en roep
één attribuutvalidator aan voor typed writer en readerbackings. Controleer
globale IDs/watermarks, precies één owner, childrol, tekst/resources,
appearance, scale/contextrefs, eindige geometrie en blockcycles. Attributen
worden geen Hatch-/Dimension-source of actieve viewportclip in deze slice.
Unknown corevelden, null en onbekende content/contextkinds falen expliciet.

IFCX-payloads bevatten de metadata en de gesloten text/mText-inhoud; factor de
bestaande lokale text-value codecs voor hergebruik. Een eigen attributepayload
kan niet tegelijk `ifccad::text`, `ifccad::mText` of een ander drawable zijn.
De buitenste plaatsing volgt de bestaande sibling placementconventie; embedded
MText-plaatsing zit in zijn eigen getypeerde lokale object. LaterWins vervangt
hele attributeDefinition/blockAttribute-values; ownership/draw order composeert
via de gewone childkeys en wordt na compositie streng gevalideerd.

Dit is een normatieve uitbreiding: **ontwikkelprofiel 0.6.0 / CBOR6**,
Zstd framing blijft 1. entitykind-code 16 voor AttributeDefinition;
BlockInstance-code 8 krijgt field 3 voor `attributes` (default []); Document
field 23 voor `attribute_visibility` (default Normal). Nieuwe gesloten records
en contextvariantcodes komen samen in de ene actieve mapping/registry. Geen
opaque-bodyframework en geen enumcode voor willekeurige INSERT-kinderen.
Alle codes worden tegen de frozen implementatie gecontroleerd vóór landing.
`conformance/next` beweegt mee; genummerde collecties en release 0.5.0 blijven
ongewijzigd. Oude readers/migratie zijn geen opleveringsvereiste.

## Mapping en verliesregels

| Bron-/nativegeval | Afgesproken gedrag voor de voorgestelde implementatie |
| --- | --- |
| Gewone Text/MText, flags, prompt/default, eigen waarde/plaatsing, styles, thickness | Beide richtingen modelleren en semantisch testen; dit mag geen permanent loss-only resultaat worden |
| Constante ATTDEF zonder ATTRIB | Definitie blijft constant; geen child verzinnen |
| Dubbele tags, ongebruikte ATTDEF, ontbrekende/of extra waarde | Volgorde en content bewaren; geen impliciete synchronisatie of unieke binding |
| Buitenste Text en embedded MText verschillen | Native beide exact bewaren; actieve MText leidend, ook leeg. CAD-normalisatie van fallback/line-count expliciet diagnosticeren; geen exact fallback-readbackclaim |
| Bestaande Text/MText-native mogelijkheden zonder CAD-equivalent | Bestaande gerichte verliesregels hergebruiken; attribuut als geheel weigeren/overslaan met pad/handle/reason wanneer actieve inhoud niet past, geen platte fallback |
| Geselecteerde Filename-/Line-Length-fields en hun textcontainers | Typed codeparts, relaties, cache en bronstatus bewaren/rebinden, zonder evaluatie; native en fysieke readback kwalificeren |
| Overige fields of unsupported fieldinhoud | Geen evaluatie of cache als literal presenteren. Niet-vertegenwoordigbare actieve attribuutinhoud gericht melden en overslaan/refuseren; geen preservationclaim |
| Eigen Text- en MTextAttribute-annotatiecontexten | Geselecteerd: typed snapshots, onafhankelijke embedded schaal, modes/default en dormant data samen ontwikkelen; geen contextrefresh |
| Secundaire attribuutrelaties | Inventariseer de concrete rollen; noodzakelijke relaties voor de geselecteerde betrouwbare attributen horen bij implementatie en fysieke readback, geen algemene uitsluiting |
| Overige XData, extensiondicts, unsupported contextrest | Eigen attr-annotatie/legacy multiline transport en geselecteerde relaties onderscheiden van echte rest; concrete verliezen of structurele rejection, geen algemeen verliesvrij statement |
| Niet-null attdef_handle | Valideer en map als optionele expliciete definitieref; rapporteer verlies van de binding waar fysiek CAD-transport die niet bewaart |
| MINSERT/dynamic/xrefsemantiek | Bestaande expliciete unsupportedgrens behouden, geen exploderen om attribuutdekking te suggereren |
| Losse ATTRIB zonder INSERT-owner, dubbele native IDs/owner, gebroken vereiste refs | Structurele fout; geen stil herstellen |
| Downsave naar CAD-versie zonder de geselecteerde betekenis | Expliciet unsupported/diagnose; geen stil versimpelde multiline/contextoutput |

Begin fysieke qualification met **AC1032 DWG en ASCII/binary DXF**. Legacy
ACAD_MLATT-readerinhoud wordt herkend via de productiecodec; diens bestaande
legacy tests zijn geen automatisch overgenomen IFCCAD-versiematrix. Voeg
hoogstens één klein legacy regressiegeval toe om geen hulp-TEXT als native
attribuut te dupliceren; bredere CAD-versiedekking is geen corpusopdracht.
Located diagnostics noemen attributepad/INSERT plus source handle als beschikbaar.
De huidige route heeft geen algemene Reject-policy; voeg die claim niet toe.

## Noodzakelijke relaties en begrensde fieldopslag

De definitielink, INSERT-ownership, resource-/schaalrefs en veldafhankelijkheden
zijn verschillende relaties. Houd ze gescheiden; een secundaire CAD-pointer
wordt geen gewone definitielink. Onderzoek de werkelijke typed codecvelden en
transportrecords in de eerste planstap en leg de bron→native→bron-toewijzing vast.
Dit is uitvoering van de geselecteerde scope, geen algemene preservationbijdrage.

Voor secundaire attr-/ATTDEF-relaties is `secondary_attributes: Vec<u64>` de
native inhoud: geordende verwijzingen naar bestaande attribuutdefinities of
INSERT-childattributen in dezelfde drawing. JSON `secondaryAttributes` bevat
de volledige entitypaths; de doelrol wordt gecontroleerd. Deze links confereren
geen ownership en hun doel wordt niet nogmaals als geometry uitgegeven. Geen
ongefundeerde unieke-tag-, sibling- of acycliciteitseis: behoud herhaling/volgorde
en bezoek met een visited-set waar traversal nodig is. De codec krijgt alleen
de getypeerde velden en read/write-aanpassing die concrete geselecteerde
relatiegevallen vereisen. Niet-gemodelleerde extra dictionary-/reactorsemantiek
blijft expliciet gediagnosticeerd, zonder blanket-uitsluiting van de relaties.

Bounded fieldopslag kiest twee gevallen: een tekeningvariabele (`AcVar Filename`)
en een objecteigenschap (`AcObjProp` Length van een gewone Line). Gemengde
literal/fieldtekst en een FIELD-container met children horen erbij, op ATTDEF
en ATTRIB, voor Text en MText. Geen klok, bestandssysteem, formuleberekening of
actualisatie tijdens native/CAD IO. De originele cached weergave blijft zichtbaar
en wordt niet als bewezen actueel aangeduid. Sheet sets, tabelformules, DIESEL en
andere evaluators krijgen concrete unsupporteddiagnostiek; geen stille flattening.

`fields: Option<IfccadAttributeFieldGraph>` op definitie en waarde bewaart:

- `root: u64`, `next_id: u64`, `records: Vec<IfccadAttributeFieldRecord>` met
  attr-local IDs (geen nieuwe drawingcounter). IDs/watermark overleven native
  save/reopen, alle records zijn vanaf root bereikbaar, elke nonroot heeft één
  eigenaar en de childgraph is acyclisch. Geen identiteitsclaim door CAD-handles.
- Record: `id`, `evaluator` (gesloten TextTemplate/DrawingVariable/ObjectProperty),
  `code: Vec<IfccadAttributeFieldCodePart>`, `format: String`, `cache`,
  `evaluation_options`, `filing_options`, `state`, `evaluation_status`,
  `error_code` (alle i32) en `error_message: String`. De i32-velden behouden
  opgeslagen bronstatus; zij sturen geen native engine aan.
- Codepart is `Literal(String)`, `Child(u64)`, `Object(u64)` of
  `CurrentBlockReference`. Literals bewaren de exacte overige code in de
  expliciete CAD-fielddialectversie 1; deze dialect is onderdeel van het
  contract, geen willekeurige JSON of algemene opaque body. De objectvariant
  verwijst naar een bestaande native entity; de symbolische blockref is alleen
  geldig als definitie-intent. Source-handlecodes/object-indexcodes en childslots
  worden herkend en veilig gebonden, nooit met globaal stringreplace aangepast.
- Cache is een getypeerde Empty/Text/Integer(i64)/Real(f64)/Boolean/Object(u64) waarde plus
  `display: String`; reals moeten eindig zijn. Cache, format en refresh/status
  blijven onafhankelijk van de momenteel opgeslagen tekst. Geen gelijkheidseis
  of stil herberekenen. Text/MText blijft de opgeslagen displaytoestand; de
  roottemplate beschrijft de fieldbinding. Afwijkende cache of tekst bewaart
  expliciet de nog niet ververste toestand.

In deze eerste dialect zijn root/childevaluators beperkt tot de geselecteerde
families en hun in de fixture vastgelegde bronspelling. Unknown evaluator,
ongeldige codegrens, unsupported CellValue-/child-value-inhoud of een niet-native
objectdependency maakt het gehele betreffende field-backed attribuut expliciet
unsupported; een noodzakelijke referentie die structureel niet resolveert is
een fout. Native gevalideerde codeparts bevatten geen verborgen source handles.
Document reconstrueert eerst alle entityhandles, daarna de fieldobjecten en hun
ownerdictionaries; herstel voert geen `update_fields` of `attach_attribute_fields`
uit. Die helpers kunnen evalueren of op tags matchen en zijn dus niet de IO-route.
Source raw object details buiten de gekozen fieldmapping krijgen een expliciete
verliesclassificatie. De fielddialect- en secundaire-linkcontracten worden in de
reguliere schema-/attribuutdocumentatie vastgelegd vóór code daarvoor wordt
geschreven; het plan benoemt hiervoor een afzonderlijk controleerbaar contracttaakje.


## Implemented transport boundary

The development pair uses official Document base `6ebe581ef85e38f4d287319259b68006014f932c` and codec base `beec5a5692eddde2d71282839c49d9d25e37f6f1` plus explicit local patches. Codec changes preserve signed ATTDEF/ATTRIB extrusion thickness. The archived converter is not used.

Legacy ACAD_MLATT secondary pointers encode the split lines of one multiline attribute. A focused AC1027 DXF regression verifies folding to one logical attribute without duplicated geometry. Generic native secondary links currently diagnose unsupported CAD transport; they are distinct from strong ATTDEF binding.

The codec's strong in-memory ATTDEF pointer has no physical DWG/DXF slot. Native JSON/CBOR retains it; native-to-CAD conversion diagnoses physical loss. Physical readback leaves it absent rather than guessing from tags. Physical storage also normalizes outer Text fallback to active embedded MText, including empty active MText; this is diagnosed before writing and does not change active MText.

Field caches retain scalar flags, unit_type, format, formatted_value, selected auxiliary checksum/ATTDEF metadata and typed Object references. Object caches are rebound together with code dependencies. AC1032 has no separate pre-2007 Field.format scalar; this loss is diagnosed while the formatting in field code and typed cache is retained. IO does not evaluate fields. Unsupported evaluators reject conversion.

The grouped gallery covers two values, renamed tags, constant defaults without ATTRIB, invisible/preset/verify/lock flags, empty active MText, mirrored/nonuniform insertion, nonzero base, nesting, Paper titleblock, independent embedded annotation scale and selected Filename/Line-Length fields on definitions and values. sample_AC1032 retains five definitions and four values through DWG and ASCII DXF. Other unsupported sample entities remain listed in diagnostics; no whole-file fidelity claim is made.

Review files have been visually accepted. Native conformance and Explorer attribute inspection are implemented. On 2026-11-05 the native full gate and the paired Document/codec test gates passed on the explicit local development pair. Bounded OCS tests cover Text/MText edits, committed grips, undo/redo, strict same-name save/reopen and existing ATTSYNC on two instances with distinct values. Matching final web-bundle and frontend checks pass; actual browser IFCX opening, child-to-owner selection and AC1032 DWG export/readback were exercised. This does not qualify general editor behavior or deployment. Reachable upstream dependency revisions and publication remain separate integration requirements.

The Entity catalogue retains its three existing Catalogue marker INSERTs (normal, rotated and mirrored), with separate CODE Text and NOTE MText children and two ATTDEFs in the same definition. Child rows are inspectable and select their owning INSERT in OCS; they do not acquire an independent top-level selection handle. The dedicated attribute gallery remains a separate example for flags, fields, contexts and Paper review.


### Review-gallery construction convention

A standalone Paper review must carry initialized physical media, positive page
limits and an explicit saved Paper canvas/view. An uninitialized page may cause
the CAD host to create a default Model viewport when opened, even if no viewport
was stored. The attribute gallery uses A4 landscape, one Paper block occurrence,
80-unit top-aligned MText boxes and 18-unit row spacing. Example regression tests
exercise these properties after physical DWG and both DXF backings. The visual
reference is schematic, not evidence of a host-render pass.

Physical media units alone do not define Paper coordinate units. A millimetre
review sheet also carries an explicit millimetre plot unit and 1:1 plot mapping;
otherwise a CAD default inch plot unit can shrink the displayed sheet by 25.4.
The physical gallery regression checks the effective page-to-scope dimensions.
