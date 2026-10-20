# Onderzoek naar de 267 maatvoeringsfouten in test.dxf

Status: XYZ-ontwerp geïmplementeerd en geverifieerd in de ongemergede
diagnosewerkmap, inclusief volledige hercontrole van het oorspronkelijke bestand.
Onderzocht op main `40b63d28f7552d8a2e649c2bd77610ce176c230f` met de
eerder geïmplementeerde diagnosegang. Productiecode, schema's en toleranties
zijn tijdens het oorzakenonderzoek niet gewijzigd. De implementatie hieronder
past het formaat aan; de toleranties blijven ongewijzigd.

## Oorzaak en bewijs

Op de onderzochte main legde de importer het native maatvlak door het eerste
referentiepunt, met de oriëntatie van de CAD-normal. Hij projecteerde het tweede
referentiepunt en het maatlijnpunt naar lokale XY-coördinaten. De toenmalige
native definitie bevatte alleen 2D-punten. Bij terugplaatsen kregen alle punten
daardoor de hoogte van het eerste punt. De 3D-nauwkeurigheidscontrole weigerde dit
terecht wanneer het oorspronkelijke punt buiten dat vlak ligt.

Dit is een representatiebeperking, geen reden om de tolerantie te verhogen.
DXF bewaart de definitiepunten in WCS: groep 10/20/30 voor het maatlijnpunt,
13/23/33 en 14/24/34 voor de referentiepunten. De tekstpositie 11/21/31 staat
in OCS. Een afwijkende WCS-Z is daarmee oorspronkelijke puntinformatie;
de beschrijving rechtvaardigt niet om die weg te schrijven als nul.
Zie Autodesk [gemeenschappelijke Dimension-codes](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-EDD54EAC-A339-4EBA-AEA6-EC8066505E2B.htm)
en [aligned-definitiepunten](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-7A123D5D-AC98-4A9A-A8CF-1A7EF5030418.htm).

De codeplaatsen zijn `crates/cad-dimension/src/placement.rs`
(`prepare_dimension_from_cad`, `project`, `world`) en
`src/dimension/values.rs` (`DimensionDefinition`). De converter controleert
de voorbereide puntparen in `crates/ifccad-convert/src/mapping/dimension.rs`.
`evaluate_dimension` in `src/dimension/measurement.rs` rekent bewust met XY.

Het oorspronkelijke bestand bevat 658 DIMENSION-records. Voor precies de 267
foutrecords uit de diagnose zijn de oorspronkelijke tags opnieuw gelezen en
de puntprojecties met exacte rationale waarden van de binary64-coördinaten
gereproduceerd. Elke gereproduceerde eerste afwijking ligt binnen het
interval uit het productierapport.

| Waarneming | Resultaat |
| --- | ---: |
| Linear / aligned | 249 / 18 |
| Normal | Alle 267 exact +Z |
| Eerste weigering bij maatlijnpunt / tweede referentiepunt | 229 / 38 |
| Eerste gerapporteerde afwijking | 0,9834754254 tot 1087,9999999999 mm |
| Referentiehoogten exact gelijk | 186 |
| Referentiehoogten verschillend, binnen 1e-9 | 43 |
| Referentiehoogten verschillend, boven 1e-9 | 38 |
| Expliciete tekstposities | 10; alle in het maatlijnvlak |
| Verwijzing naar CAD picture block | Alle 267 |

Bij `0xEF354` staan beide referentiepunten op Z ≈ 544 mm en het maatlijnpunt
op Z = 0. De importer verplaatst het maatlijnpunt daardoor circa 544 mm.
Bij `0x114ADE` liggen de referenties op circa 544 en 1632 mm en de maatlijn
op circa 0 mm. Eén gedeelde referentiehoogte kan die tweede maat niet bewaren.
De 38 grote referentieverschillen zijn dus niet allemaal afrondingsruis.

Alle 267 hebben meer dan 1e-9 hoogteverschil tussen hun drie definitiepunten.
Zelfs het beste parallel verschoven vlak zou per maat nog minstens
24,8996 tot 816 mm maximale hoogtefout opleveren. Alleen de oorsprong
verplaatsen lost het ontbrekende hoogtegegeven niet op. Het vlak kantelen
zou de oorspronkelijke +Z-normal en de betekenis van de meetrichting wijzigen.

## Goedgekeurde native oplossing

Bewaar het maatvlak en de ruimtelijke referentiepunten afzonderlijk. Breid
de bestaande linear/aligned-definitie uit naar twee volledige lokale
XYZ-referentiepunten. `firstPoint` en `secondPoint` hebben ieder precies drie
coördinaten; `dimensionLinePoint` heeft precies twee lokale XY-coördinaten.
Er komen geen losse Z-velden of parallelle bronkopieën. Alle componenten moeten
eindig zijn; null en onbekende velden blijven ongeldig. Het eerdere voorstel
voor aanvullende Z-velden is hiermee op verzoek van de gebruiker vervangen.

De gemeenschappelijke plaatsing bepaalt het maatvlak. Voor CAD-import wordt
de oorsprong bij het oorspronkelijke maatlijnpunt gekozen, met behoud van
de bestaande OCS-oriëntatie. Het maatlijnpunt ligt dan op lokale Z = 0.
Beide referentiepunten worden volledig in dit stelsel uitgedrukt, inclusief
hun eigen Z. De expliciete tekst wordt vanuit OCS naar WCS en vervolgens
naar het maatvlak vertaald, met de bestaande volledige 3D-controle.

Voor de ruimtelijke betekenis kan dezelfde conventie als bij de bestaande
blocktransformatie worden gebruikt:

```text
N = U × V                         (exact kruisproduct van opgeslagen assen)
reference1 = O + U*x1 + V*y1 + N*z1
reference2 = O + U*x2 + V*y2 + N*z2
dimensionLinePoint = O + U*xl + V*yl
```

Zo is lokale Z een coördinaat langs de afgeleide derde as, niet stilzwijgend
wereld-Z. Assen worden bij laden/valideren niet opnieuw genormaliseerd.
De kern definieert en evalueert dit zonder CAD-afhankelijkheid. De adapter
lost de coördinaten in deze basis op; een losse dotproductprojectie onder
de aanname van exact orthonormale binary64-assen is onvoldoende.

De meetwaarde blijft expliciet een meting in het maatvlak: linear gebruikt
de bestaande projectie op de lokale richting; aligned gebruikt de bestaande
XY-afstand. Z is bewaard referentiegegeven en wordt niet opgeteld in een
nieuwe ruimtelijke afstand. De CAD-cache `actualMeasurement` wordt niet tot
gezaghebbende meetwaarde verheven. De huidige formule en formatter blijven
behouden; de documentatie moet het onderscheid met een 3D-afstand verduidelijken.

De API biedt `reference_points()` voor volledige referentiepunten en
`projected_points()` voor de geprojecteerde meetpunten. De algemene naam
`points()` vervalt zodat callers niet ongemerkt alleen XY overnemen. Export
reconstrueert beide volledige WCS-referenties en het maatlijnpunt; OCS-tekst,
normal, rotatie, label, stijl, overrides en relaties behouden hun eigen mapping.

Een puur rekenkundige proef met deze representatie bewaart de 801
definitieankers onder de bestaande 1e-9 mm-limiet. De maximale exacte
reconstructieafwijking, als decimale benadering weergegeven, is 1,59e-17 mm;
de tien expliciete tekstankers reconstrueren exact. Dit bewijst de
haalbaarheid voor deze +Z-records, niet een geïmplementeerde conversie of
kwalificatie van schuine normalen, blokvoorkomens, opslag en CAD-export.

## Waarborgen en grenzen

- De default blijft 1e-9 in het betreffende coördinatendomein. Exacte en
  expliciet ingestelde toleranties en Paper-/occurrence-regels blijven gelden.
  Alle drie WCS-componenten blijven onderdeel van het geometrische bewijs,
  onder zowel Allow als Reject. Afronding krijgt de bestaande diagnostiek;
  overschrijding of ontbrekend bewijs blijft een harde fout.
- Er komt geen nulmaken, middelen of gezamenlijke hoogte voor beide
  referenties, geen uitsluiting van punten uit de controle en geen vervanging
  door een losgemaakte tekening van lijntjes en tekst.
- Een expliciet tekstanker buiten het native maatvlak blijft een harde fout
  zolang daarvoor geen native representatie is ontworpen. Het wordt niet
  afgeplat. Voor de onderzochte tien expliciete ankers is dat niet nodig.
- De bron-pictureblocks blijven een afzonderlijke dekkingsgrens. Alle 267
  hebben zo'n verwijzing. Het geplande graphics-contract moet die weergave
  afzonderlijk bewaren; alleen referentie-Z toevoegen maakt de huidige
  conversie niet volledig verliesvrij. Bestaande verliesmeldingen en Reject
  blijven ook voor caches, unsupported bronvelden en relaties van kracht.
- Associaties blijven verwijzingen. Laden of opslaan vervangt de opgeslagen
  referenties niet door actuele geometrie uit een andere entity.

## Formaat en uitvoering

Dit vraagt een begrensde logische contractuitbreiding, gevolgd door gedeelde
validatie, API, IFCX-schema/mapping, CBOR-mapping en conversie in beide
richtingen. Een aanpassing uitsluitend in de importer kan de ontbrekende
native gegevens niet duurzaam bewaren.

De referentiearrays veranderen van XY naar verplicht XYZ; het maatlijnpunt
blijft XY. De actieve experimentele 0.1.0-schema's worden gezamenlijk
bijgewerkt. Oudere XY-referenties worden expliciet geweigerd, niet stil
aangevuld. `conformance/next` legt dit vast; geen genummerde collectie verandert.
Dit is een expliciete wijziging van het nog niet uitgebrachte profiel.

CBOR blijft op verzoek van de gebruiker één wijzigbare experimentele versie 1,
net als het nog niet uitgebrachte IFCCAD-profiel. Het actuele
[mappingcontract](../../schemas/ifccad/cbor-mapping-1.md#envelope-and-revision)
en de fixtures worden rechtstreeks bijgewerkt; er zijn geen historische mappings
of migratieroutes. Zstd behoudt transportversie 1. Er wordt geen benchmark gestart.

Aanbevolen uitvoering:

1. Leg het logische contract en de evolutie van beide encodings vast.
2. Voeg de native referentie-Z, gedeelde eindigheids-/bereikvalidatie,
   expliciete punt-API en ongewijzigde geprojecteerde meting toe.
3. Implementeer opslag en productie-readback, inclusief nonzero Z, defaults,
   null/unknown-afwijzing en conformance voor beide Dimension-soorten.
4. Pas de neutrale CAD-voorbereiding en import/export aan, met volledig
   3D-bewijs en behoud van bestaande semantische verliesdiagnostiek.
5. Test afzonderlijke Z's, niet-nul maatlijnhoogte, schuine/negatieve normalen,
   expliciete OCS-tekst, Model/Paper/lokale blokken, versterkte nested
   voorkomens, exacte tolerantie en werkelijke ASCII/binaire DXF-/DWG-readback.
   Vergelijk native waarden en semantische punten, niet alleen meettekst.
6. Diagnoseer daarna het volledige oorspronkelijke test.dxf in één nieuwe
   rapportmap, zonder voorafgaande uitsluitingen. Controleer alle 267 handles
   en eventuele vervolgproblemen; fouten in andere families blijven zichtbaar.
   Rond de uiteindelijke kandidaat af met de voorgeschreven volledige gate.

De bestaande regressie `independent_numeric_excess_is_fatal_under_both_policies`
bevestigt momenteel dat een tweede punt met afwijkende Z wordt geweigerd.
Bij implementatie moet daarvoor een test komen die de nieuwe ruimtelijke
referentie bewaart, plus afzonderlijke regressies die echte nauwkeurigheids-
overschrijding onder beide verliesbeleiden blijven afwijzen.

Deze reparatie raakt het definitie-/coördinatencontract dat de geplande
Dimension-graphics gebruikt. Aanbevolen is haar vóór, of expliciet samen met,
de coördinatenmapping van die slice te behandelen. De eigendomskeuze voor
graphics en de roadmap voor verdere maatsoorten worden hiermee niet gewijzigd.
Onderzochte issues: [#10](https://github.com/OpenAEC-Foundation/ifccad/issues/10)
bevestigt de tolerantie; [#11](https://github.com/OpenAEC-Foundation/ifccad/issues/11)
selecteert geen nieuwe AA-plaatsing; [#13](https://github.com/OpenAEC-Foundation/ifccad/issues/13)
houdt associatieve updates afzonderlijk. Een AA-wijziging of update-engine
is voor deze oplossing geen vereiste.

## Reproductie en onderzoeksgrens

Bron: `test.dxf`, 33.288.016 bytes, SHA-256
`74d8d7b861cda0e3859879c8149df2344a457f5c9cefce6a144a7cf52a4fbe9a`.
De eerder voltooide diagnose gebruikte de gepinde codec `325fa98` met de
gedocumenteerde Dimension-polariteitspatch. Er is geen nieuwe codecpatch nodig
om de oorzaak in de oorspronkelijke coördinaten aan te wijzen.

De lokale map `.superpowers/dimension-spatial/` bij deze werkmap bevat
`investigate.py` en `evidence.json`: oorspronkelijke tags en afzonderlijke
reconstructiebewijzen voor alle 267 handles. Het eerdere volledige rapport
blijft behouden in `.superpowers/conversion-diagnostics/test-dxf-20261019-v2/`.
Het oorspronkelijke oorzakenonderzoek wijzigde de productiecode niet. De
daaropvolgende implementatie en kwalificatie staan hieronder. Er is geen
benchmark, commit of integratie uitgevoerd.

## Implementatie en hercontrole

De native definitie, gedeelde validatie, exacte ruimtelijke plaatsing,
CAD-import/export, actieve IFCX-schema's en de experimentele CBOR-mapping zijn aangepast.
Beide referenties bewaren XYZ; maatlijnpunt en expliciete tekst blijven XY
in het geplaatste maatvlak. De bestaande meetformules en toleranties blijven
ongewijzigd. Oude XY-referenties worden geweigerd.

De volledige diagnose op hetzelfde oorspronkelijke `test.dxf` is voltooid,
zonder uitsluitingen of benchmark. Alle 54.598 bronentiteiten zijn teruggevonden
en vergeleken met de eerdere inventarisatie; geen bronhandle ontbreekt.

| Uitkomst | Vóór | Na |
| --- | ---: | ---: |
| Eerder falende linear-maatvoeringen | 249 fout | 249 geslaagd |
| Eerder falende aligned-maatvoeringen | 18 fout | 18 geslaagd |
| Andere harde fouten | 243 | 243, identieke foutrecords |
| Geslaagde lokale entity-voorbereidingen | 48.304 | 48.571 |
| Gewone profielomissies, met diagnostiek | 5.784 | 5.784 |

Er zijn geen nieuwe harde fouten of extra omissies. De 243 resterende fouten
zijn 154 LWPOLYLINE-voorbereidingsfouten, 78 HATCH-nauwkeurigheidsoverschrijdingen
en 11 HATCH-spleetfouten. Ook hun foutdetails en oorspronkelijke toleranties
zijn ongewijzigd. Van de blokvoorkomens slagen 2.155 controles; 115 blijven
afhankelijk van falende broninhoud. Daardoor blijft de tekening als geheel
geblokkeerd; er wordt geen gedeeltelijke geconverteerde tekening gecertificeerd.

Voor de 267 herstelde maatvoeringen is de hoogste gerapporteerde gecertificeerde
geometrische afrondingsgrens `1.589468121522158e-17` mm, tegenover de ongewijzigde
default `1e-9` mm. Eventuele afronding blijft expliciet gemeld; hoogte wordt
niet uit het geometrische bewijs verwijderd. Bestaande semantische meldingen,
waaronder de ontbrekende CAD-pictureweergave, blijven afzonderlijk van kracht.

Het nieuwe volledige rapport heeft SHA-256
`79ab85263e7c14762cfacc210d8cff76032f09de2e3809da10fc3ae6454608ce` en staat
lokaal in `.superpowers/dimension-spatial/test-dxf-20261020-xyz-v1/`.
`dimension-comparison.json` bevat alle 267 bronhandles met oude fout en nieuwe
geslaagde status; `failures.json` bevat de 243 resterende fouten.

De volledige gate slaagt: workspacecontrole, formattering, Clippy en
`cargo test --workspace` (967 geslaagd, nul fout, één bestaande genegeerde
performancetest). Native IFCX/CBOR-readback, verschillende referentiehoogten,
gekantelde/omgekeerde normalen, expliciete OCS-tekst, Model/Paper/lokale blokken,
werkelijke ASCII/binaire DXF en AC1032 DWG, en genest versterkte XY/Z-afronding
zijn gericht gecontroleerd. Daarnaast slagen zeven optionele Zstd-tests en
137 frontendtests, met vier bestaande skips. De bron/dependency/configuratie-
identiteit, uitvoerbare bestanden en gatebewijzen zijn lokaal vastgelegd.

### Vervolgaanpassing: wijzigbare CBOR-versie 1

De diagnose en verificatie hierboven zijn uitgevoerd voordat de gebruiker vroeg
het versienummer op 1 te houden. Hun oorspronkelijke provenance en snapshots
blijven ongewijzigd (toen met envelopnummer 3). De vervolgaanpassing verandert
uitsluitend de CBOR-versiemarkering, het mappingregister en bijbehorende fixtures
en documentatie. Maatgeometrie, toleranties en CAD-conversie veranderen niet;
de inventarisatie van de 267 maatvoeringen blijft daarvoor geldig. De aangepaste
CBOR-indeling krijgt afzonderlijke verificatie, zonder nieuwe benchmark.
