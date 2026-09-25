import { isVector, fieldSummary } from './fields.mjs';
import { columnTable, collectionBrowser, graphPager, pageWindow, pager, collectionWindow } from './browsing.mjs';
const escape=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const sumBytes=(a,b)=>{try{return (BigInt(a)+BigInt(b)).toString();}catch{return '?';}};
const preview=value=>fieldSummary(value);
const rawDetails=(title,data)=>{const full=JSON.stringify(data,null,2)||'';return `<details><summary>${escape(title)}${full.length>12000?' · preview':''}</summary><pre><code>${escape(full.slice(0,12000))}</code></pre>${full.length>12000?'<p class="small-note">Preview ingekort tot 12000 tekens.</p>':''}</details>`;};
const definitionList=rows=>'<dl class="meta-list">'+rows.map(([k,v])=>`<dt>${escape(k)}</dt><dd>${escape(v)}</dd>`).join('')+'</dl>';
const description={
  DrawingSet:'Het vertrekpunt van dit CAD-pakket. Verbindt tekeningen en optionele preservation-resources. Een gebouwmodel is hiervoor niet vereist.',
  Drawing:'De semantische identiteit van een tekening. Verwijst naar layouts en één gedeelde DrawingRepresentation.',
  DrawingLayout:'Een layout selecteert via scopeId een model- of paperspace-scope in IFCDR. Een paper-layout kan eigen effectieve plotinstellingen bevatten; beide gebruiken de DrawingRepresentation van de tekening.',
  DrawingRepresentation:'De brug van de IFCX-graph naar de tekenresource. De resource-ID is de identiteit; de URI is alleen de opslaglocatie.',
  PreservationRepresentation:'De IFCX-verwijzing naar bronbehoud. Verbindt een brondocument en tekenresources met een IFCPR-resource.',
  Layer:'Een gedeelde semantische laagdefinitie in IFCX. IFCDR-entiteiten verwijzen via een compacte layerBinding naar deze node.',
  Appearance:'Een gedeelde definitie van kleur, lijnpatroon, lijndikte en opacity. De eigenschappen kunnen elk een eigen overervingsmodus hebben.',
  group:'Deze groep organiseert de weergave. Het is geen extra node of container in het bestandsformaat.',
  'drawing-resource':'De tekeninhoud zit in typed collecties: scopes, bindings, entiteiten en tekenvolgorde. JSON-kolommen zijn een fysieke mapping van dat logische model.',
  'preservation-resource':'Bewaart broninformatie die niet volledig native is vertegenwoordigd. Records beschrijven de bron; bindings leggen de relatie naar native inhoud vast.',
  scope:'Een afzonderlijk coördinatiedomein met scope-ID, metadata en bounds. Entiteiten en tekenvolgorde horen bij een scope.',
  'block-definition':'Een lokale blockdefinitie in deze IFCDR-resource. De scope bevat de gedeelde entiteiten; dit is geen IFCX-node.',
  collection:'Gelijksoortige entiteiten delen een typed stream. Dezelfde positie in elke eigenschapskolom vormt één rij; entityId bepaalt de identiteit.',
  entity:'Een native entiteit heeft een identiteit, scope, laag, appearance en typespecifieke betekenis. De combinatie van resource-ID en entityId identificeert haar in het pakket.',
  field:'Een uitvergroting van een typespecifiek veld binnen de entiteit. Deze node helpt de gegevens te verkennen; het is geen afzonderlijk IFCX-object of extra entiteit in het bestand.',
  record:'Een bronobject met eigen bronidentiteit. De payload verwijst naar een bytebereik in een blob; dit is geen nieuwe native tekenentiteit.',
  projection:'Koppelt één of meer bronrecords aan native inhoud. Coverage en baselineFingerprint helpen beschrijven welke betekenis de binding afdekt.',
  blob:'De werkelijke bronbytes. Records en attachments verwijzen naar delen daarvan met een offset en lengte; de digest adresseert en controleert de bytes.',
  attachment:'Aanvullende brongegevens, bijvoorbeeld XDATA. Verwijst naar een parent-record en eventueel projecties en een eigen payloadbereik.',
  'concept-record':'Illustratief bronbehoud voor aanvullende, brongebonden eigenschappen. Dit is geen geregistreerd record en geen door de converter geschreven IFCPR-data.',
  'concept-projection':'Een mogelijke koppeling tussen bewaarde broninformatie en een toekomstige native entiteit. De herstelvoorwaarden zijn nog te ontwerpen.',
  'concept-style':'Een mogelijke gedeelde maatstijl met een eigen IFCX-identiteit. Welke eigenschappen en overervingsregels hierbij horen is nog te ontwerpen.',
};

// Keep this inventory aligned with the registry's object streams. A test fails
// when a new native family is registered without its own explanation.
export const entityDescriptions={
  point:'Een punt op de oorsprong van zijn plaatsing; de assen bepalen alleen de oriëntatie van een eventuele markering.',
  line:'Een lijn tussen twee XYZ-eindpunten binnen haar scope.',
  polyline:'Een polylijn van geordende lokale XY-punten. closed bepaalt of zij sluit; placement bepaalt haar ligging in XYZ.',
  planarPolyline:'Een vlakke polylijn van lokale XY-punten. Bulges kunnen segmenten krommen; placement legt het vlak in XYZ.',
  spatialPolyline:'Een ruimtelijke polylijn van geordende XYZ-punten, zonder vlakplaatsing of bulges.',
  circle:'Een cirkel met een middelpunt, vlak en straal.',
  arc:'Een cirkelboog met een straal en een begin- en zwaaiparameter in radialen.',
  ellipse:'Een ellips met een grote en kleine halve as in haar plaatsingsvlak.',
  ellipseArc:'Een ellipsboog met twee halve assen en een begin- en zwaaiparameter.',
  blockInstance:'Een block instance plaatst een gedeelde blockdefinitie met een eigen positie, rotatie en schaal.',
  viewport:'Een viewport staat in paperspace en toont een deel van modelspace binnen een kader.',
};

export const collectionDescriptions={
  point:'Punten staan in pointStream. Elke rij heeft een entityId en een placement; de oorsprong is de XYZ-positie, terwijl de assen alleen een mogelijke markering oriënteren.',
  line:'Lijnen staan samen in de typed lineStream. Elke lijn neemt één positie in alle eigenschapskolommen in: entityId, scopeId, twee XYZ-eindpunten, layerId en appearanceId. De rij-index is niet de identiteit; entityId is dat binnen de resource.',
  polyline:'Polylijnen staan samen in polylineStream. Elke rij heeft een entityId en gebruikt vertexOffset en vertexCount om haar lokale XY-punten uit de gedeelde x/y-pools te kiezen. closed en een eventuele placement staan in dezelfde rij.',
  planarPolyline:'Vlakke polylijnen staan in planarPolylineStream. vertexOffset en vertexCount kiezen lokale XY-punten en optionele bulges uit gedeelde vertexkolommen; closed en placement horen bij de entiteitsrij.',
  spatialPolyline:'Ruimtelijke polylijnen staan in spatialPolylineStream. vertexOffset en vertexCount kiezen XYZ-punten uit gedeelde kolommen; closed staat bij de entiteitsrij. Er is geen placement of bulge.',
  circle:'Cirkels staan in circleStream. Elke rij bewaart entityId, placement en radius naast de gemeenschappelijke scope-, laag- en appearanceverwijzingen.',
  arc:'Cirkelbogen staan in arcStream. Elke rij bewaart placement, radius, startParameter en een getekende sweepParameter in radialen.',
  ellipse:'Ellipsen staan in ellipseStream. Elke rij bewaart placement, semiMajorRadius en semiMinorRadius; de assen van placement bepalen hun richting.',
  ellipseArc:'Ellipsbogen staan in ellipseArcStream. Elke rij bewaart de halve assen en de begin- en zwaaiparameter van een gedeeltelijke ellips.',
  blockInstance:'Block instances staan samen in blockInstanceStream. Elke rij verwijst via definitionScopeId naar gedeelde blockinhoud en bewaart een eigen transform voor plaatsing, rotatie en schaal.',
  viewport:'Viewports staan in viewportStream. scopeId wijst naar paperspace, viewScopeId naar modelspace. frame en view beschrijven kader en zicht; layerOverrideOffset en layerOverrideCount selecteren rijen uit viewportLayerOverrideStream.',
};

const placementAxes={
  origin:'De oorsprong van het lokale vlak in XYZ. Zij staat in placement.origin, of volgt uit de logische standaardwaarde als placement ontbreekt.',
  X:'De lokale X-as geeft de richting van toenemende x aan. Deze vector staat in placement.X, of volgt uit de logische standaardwaarde.',
  Y:'De lokale Y-as geeft de richting van toenemende y aan. Deze vector staat in placement.Y, of volgt uit de logische standaardwaarde.',
};

const blockPlacementAxes={
  origin:'De oorsprong van de blockplaatsing in XYZ. Zij staat in transform.placement.origin, of volgt uit de logische standaardwaarde als placement ontbreekt.',
  X:'De lokale X-as van de blockplaatsing. Deze vector staat in transform.placement.X, of volgt uit de logische standaardwaarde.',
  Y:'De lokale Y-as van de blockplaatsing. Deze vector staat in transform.placement.Y, of volgt uit de logische standaardwaarde.',
};

const planeFieldDescriptions={
  'placement.origin':'De oorsprong van de plaatsing in XYZ binnen de eigen scope.',
  'placement.X':'De lokale X-richting van het plaatsingsvlak.',
  'placement.Y':'De lokale Y-richting van het plaatsingsvlak.',
};
const curvePlacement='De oorsprong bepaalt het middelpunt; X en Y bepalen de richting van de curve in haar vlak.';
const curveStart='De beginparameter in radialen, gemeten van de lokale X-as naar de Y-as.';
const curveSweep='De getekende zwaaiparameter in radialen; het teken bepaalt de draairichting.';
const spatialVertices='Geordende XYZ-punten uit de x-, y- en z-puntenpools, geselecteerd door vertexOffset en vertexCount.';
const closedPolyline='closed verbindt het laatste punt impliciet met het eerste, zonder een extra punt op te slaan.';

export const fieldDescriptions={
  point:{placement:'De oorsprong van placement is de puntpositie; X en Y oriënteren alleen een mogelijke markering.',...planeFieldDescriptions},
  line:{
    start:'Het beginpunt van de lijn in XYZ binnen haar scope. De waarden komen uit x1, y1 en z1 van dezelfde rij in lineStream; ontbrekende z1 is 0.',
    end:'Het eindpunt van de lijn in XYZ binnen haar scope. De waarden komen uit x2, y2 en z2 van dezelfde rij in lineStream; ontbrekende z2 is 0.',
  },
  polyline:{
    vertices:'De geordende lokale XY-punten van de polylijn. vertexOffset en vertexCount wijzen naar haar bereik in de gedeelde x- en y-puntenpools van polylineStream.',
    'vertices.*':'Dit lokaal XY-hoekpunt staat op deze positie in de gedeelde x/y-puntenpools. De veldnode is een uitsplitsing in de viewer, geen afzonderlijk entiteitsrecord.',
    closed:'Als closed waar is, wordt het laatste punt met het eerste verbonden. De waarde staat per rij in de closed-kolom van polylineStream.',
    placement:'Deze plaatsing zet lokale XY-punten om naar XYZ met een oorsprong en twee assen. Zij staat per rij in de placement-kolom van polylineStream; zonder waarde geldt het standaard XY-vlak.',
    'placement.origin':placementAxes.origin,
    'placement.X':placementAxes.X,
    'placement.Y':placementAxes.Y,
  },
  planarPolyline:{
    vertices:'Geordende lokale XY-punten uit de x- en y-puntenpools, geselecteerd door vertexOffset en vertexCount.',
    'vertices.*':'Een lokaal XY-punt op deze positie in de gedeelde puntenpool.',
    closed:closedPolyline,
    placement:'Deze plaatsing zet lokale XY-punten om naar XYZ met een oorsprong en twee assen; zonder kolomwaarde geldt het standaard XY-vlak.',
    ...planeFieldDescriptions,
    bulges:'Eén bulge per vertex. Een niet-nulwaarde is tan(booghoek/4); de laatste waarde van een open polylijn blijft bewaard maar tekent geen segment.',
    'bulges.*':'De getekende buiging van het segment dat bij dit punt begint; bij een open laatste punt blijft de waarde alleen bewaard.',
  },
  spatialPolyline:{vertices:spatialVertices,'vertices.*':'Een XYZ-punt op deze positie in de gedeelde puntenpool.',closed:closedPolyline},
  circle:{placement:curvePlacement,...planeFieldDescriptions,radius:'De positieve straal van de cirkel in de eenheid van de tekenresource.'},
  arc:{placement:curvePlacement,...planeFieldDescriptions,radius:'De positieve straal van de cirkelboog in de eenheid van de tekenresource.',startParameter:curveStart,sweepParameter:curveSweep},
  ellipse:{placement:curvePlacement,...planeFieldDescriptions,semiMajorRadius:'De grote halve as langs de lokale X-richting.',semiMinorRadius:'De kleine halve as langs de lokale Y-richting.'},
  ellipseArc:{placement:curvePlacement,...planeFieldDescriptions,semiMajorRadius:'De grote halve as langs de lokale X-richting.',semiMinorRadius:'De kleine halve as langs de lokale Y-richting.',startParameter:curveStart,sweepParameter:curveSweep},
  blockInstance:{
    definitionScopeId:'Verwijst naar de scope met de gedeelde blockdefinitie. definitionScopeId staat per instantie in blockInstanceStream; de geometrie wordt niet gekopieerd.',
    transform:'De plaatsing, rotatie en schaal van deze block instance. transform staat per instantie in blockInstanceStream; ontbrekende onderdelen gebruiken logische standaardwaarden.',
    'transform.placement':'De oorsprong en lokale X- en Y-as van deze blockplaatsing. Deze waarden staan in transform.placement; ontbrekende onderdelen volgen de logische standaardwaarden.',
    'transform.placement.origin':blockPlacementAxes.origin,
    'transform.placement.X':blockPlacementAxes.X,
    'transform.placement.Y':blockPlacementAxes.Y,
    'transform.rotation':'De rotatie in radialen binnen de lokale plaatsing. Zij staat in transform.rotation; zonder waarde is de rotatie 0.',
    'transform.scale':'De schaal per as van deze block instance. Zij staat in transform.scale; zonder waarde is elke schaalfactor 1. Een negatieve factor spiegelt de definitie.',
  },
  viewport:{
    viewScopeId:'De modelspace-scope die door deze viewport wordt bekeken. viewScopeId staat in dezelfde rij van viewportStream; scopeId is de eigen paperspace-scope.',
    frame:'De positie en afmetingen van het viewportkader in de eigen paperspace-scope. frame staat per viewport in viewportStream.',
    view:'De blik op modelspace met centrum, richting, hoogte, projectie en clipping. view staat per viewport in viewportStream.',
    renderMode:'De weergavemodus voor deze viewport. renderMode staat als code in viewportStream.',
    viewEnabled:'Geeft aan of het modelbeeld in deze viewport actief is. viewEnabled staat per rij in viewportStream.',
    viewLocked:'Geeft aan of de viewportweergave vergrendeld is. viewLocked staat per rij in viewportStream.',
    paperClip:'Een eventuele afsnijding door een gesloten grensentiteit in dezelfde paperspace-scope. paperClip staat in viewportStream; boundaryEntityId verwijst naar die entiteit.',
    plotShadingOverride:'Een optionele viewport-afwijking voor plotshading. De waarde staat per rij in viewportStream; null gebruikt de layoutinstelling.',
    layerOverrides:'Laagafwijkingen zijn opeenvolgende rijen in viewportLayerOverrideStream. layerOverrideOffset en layerOverrideCount van deze viewport kiezen het bereik; de rijen zijn geen afzonderlijke CAD-entiteiten.',
    'layerOverrides.*':'Een child-rij met layerId, frozen en eventueel appearanceOverrideId. Deze rij hoort bij de viewport en is geen zelfstandige entiteit.',
  },
};

const newStreamDescriptions={
  collection:'Deze geregistreerde objectstream toont de opgeslagen kolommen en rijen. Een typespecifieke uitleg volgt zodra het contract voor dit type is vastgesteld.',
  entity:'Dit object toont zijn opgeslagen velden en verwijzingen. Een typespecifieke uitleg volgt zodra het contract voor dit type is vastgesteld.',
  field:'Deze opgeslagen veldwaarde wordt zonder vastgestelde typespecifieke betekenis getoond.',
};

function collectionTable(node,model){
  if(node.concept)return `<div class="structure-note">Een illustratieve collectie voor <code>${escape(node.family)}</code>. Voor dit entiteittype zijn semantiek en streamindeling nog niet vastgesteld.</div>${definitionList([['Basisvelden','entityId · scopeId · layerId · appearanceId'],[node.family,'maatsoort · definitiepunten · maatlijnpositie · stijlref']])}`;
  return columnTable(node,model);
}

function payloadStrip(preservation,blob,model){
  const b=preservation.body,rid=preservation.resourceId;
  const ranges=(b.records||[]).filter(r=>r.payload?.blobId===blob.id).map(r=>({label:'record '+r.recordId,id:`record:${rid}:${r.recordId}`,...r.payload})).concat((b.structuredAttachments||[]).filter(a=>a.payload?.blobId===blob.id).map(a=>({label:a.kind,id:`attachment:${rid}:${a.attachmentId}`,...a.payload}))).sort((a,b)=>a.offset-b.offset);
  return `<div class="structure-heading"><h3>Bytebereiken</h3><span>${blob.byteLength} bytes</span></div><div class="payload-strip">${ranges.slice(0,50).map(r=>`<button class="payload-part" data-select="${escape(r.id)}" style="flex:${r.length}" aria-label="${escape(r.label)}: offset ${r.offset}, lengte ${r.length}">${escape(r.label)}</button>`).join('')}</div><div class="payload-caption">${ranges.slice(0,50).map(r=>`${escape(r.label)} [${r.offset}, ${sumBytes(r.offset,r.length)})`).join(' · ')}</div>`;
}

function fieldDetails(node,model){
  const owner=model.byId.get(node.ownerId),entity=owner.entity,value=node.raw;
  const vector=v=>Array.isArray(v)?v:[v.x,v.y,v.z];
  const tuple=v=>'('+vector(v).join(', ')+')';
  let html=`<button class="relation field-owner" data-select="${escape(owner.id)}"><span><small>Onderdeel van</small>${escape(owner.label)} · ${escape(owner.resourceId)}</span><i aria-hidden="true">↗</i></button>`;
  html+=definitionList([['Veldpad',node.fieldPath.join('.')],['Herkomst',node.concept?'Illustratief concept':node.implicit?'Standaardwaarde · niet opgeslagen':'Entiteitdata uit het pakket']]);
  if(node.implicit)html+='<p class="structure-note">Deze rij heeft geen expliciete placement. De viewer toont daarom het standaard XY-vlak: oorsprong (0, 0, 0), X = (1, 0, 0), Y = (0, 1, 0). Er wordt geen placement aan het bronbestand toegevoegd.</p>';
  if(isVector(value))html+=definitionList(vector(value).map((v,i)=>[['x','y','z'][i],v]));
  else if(value===null||typeof value!=='object')html+=`<div class="field-value"><code>${escape(JSON.stringify(value))}</code></div>`;
  else if(Array.isArray(value)){
    const shared=collectionWindow(model,node.id),w=shared||pageWindow(model,node.id+':points',value.length);
    const points=node.fieldPath[0]==='vertices';
    html+=`<div class="structure-heading"><h3>${points?'Punten in volgorde':'Items in volgorde'}</h3><span>${value.length} ${points?'punten':'items'}</span></div>${shared?'':pager(node.id+':points',w)}<div class="relation-list collection-list">${value.slice(w.start,w.end).map((v,i)=>`<button class="relation" data-select="${escape(node.id+'.'+(w.start+i))}"><span><small>${points?'punt':'item'} ${w.start+i}</small><code>${escape(points?tuple(v):JSON.stringify(v))}</code></span><i aria-hidden="true">↗</i></button>`).join('')}</div>`;
  }
  const top=node.fieldPath[0];
  if((top==='placement'||top==='plane')&&node.fieldPath.length===2){
    const meaning={origin:'De oorsprong O: het nulpunt van het lokale vlak in XYZ.',X:'De lokale X-as: de richting waarin de lokale x-coördinaat toeneemt.',Y:'De lokale Y-as: de richting waarin de lokale y-coördinaat toeneemt.'};
    if(meaning[node.label])html+=`<p class="structure-note">${meaning[node.label]}</p>`;
  }
  if(top==='placement'&&node.fieldPath.length===1&&['polyline','planarPolyline'].includes(entity.kind)&&value?.origin&&value?.X&&value?.Y){
    const p=value;
    html+=`<div class="structure-heading"><h3>Van lokaal XY naar XYZ</h3></div><div class="placement-formula"><code>P = O + x · X + y · Y</code></div><div class="placement-axes">${[['origin','O · oorsprong'],['X','X · lokale x-as'],['Y','Y · lokale y-as']].map(([k,label])=>`<button class="relation" data-select="${escape(node.id+'.'+k)}"><span><small>${label}</small><code>${escape(tuple(p[k]))}</code></span><i aria-hidden="true">↗</i></button>`).join('')}</div>`;
    html+=`<table class="stream-table placement-table"><thead><tr><th>Punt</th><th>Lokaal (x, y)</th><th>XYZ</th></tr></thead><tbody>${entity.geometry.vertices.slice(0,50).map((v,i)=>`<tr><td>${i}</td><td><code>${escape(tuple(v))}</code></td><td><code>${escape(tuple(entity.points[i]))}</code></td></tr>`).join('')}</tbody></table>`;
    const both=entity.geometry.vertices.findIndex(([x,y])=>x!==0&&y!==0);
    const i=both>=0?both:Math.max(0,entity.geometry.vertices.findIndex(([x,y])=>x!==0||y!==0)),[x,y]=entity.geometry.vertices[i];
    html+=`<p class="small-note">Bij punt ${i}: <code>${escape(tuple(p.origin))} + ${x} · ${escape(tuple(p.X))} + ${y} · ${escape(tuple(p.Y))} = ${escape(tuple(entity.points[i]))}</code>.</p><p class="small-note">XYZ ligt binnen de eigen scope. De scope-base is metadata en wordt niet bij deze plaatsing opgeteld. De lokale punten blijven ongewijzigd opgeslagen.</p>`;
  }
  return html;
}

export function renderInspector(model,id,collapsed){
  const node=model.byId.get(id);if(!node)return;
  const content=document.getElementById('selection-content'),structure=document.getElementById('selection-structure'),links=document.getElementById('selection-links'),data=document.getElementById('selection-data'),domain=document.getElementById('selection-domain');
  domain.textContent=node.domain.toUpperCase();domain.className='domain-pill '+node.domain;
  let desc=Object.hasOwn(description,node.kind)?description[node.kind]:'Een IFCX-node in de semantische graph.';
  if(node.id.startsWith('view:ifcdr:')&&node.id.endsWith(':scopeTable'))desc='De scopeTable bevat de coördinatiedomeinen van deze tekenresource. Deze groep bladert door de tabel en is geen extra node in het bestandsformaat.';
  if(node.id.startsWith('view:ifcdr:')&&node.id.endsWith(':blockDefinitionTable'))desc='De blockDefinitionTable bevat gedeelde blockdefinities. Elke definitie verwijst via scopeId naar de scope met haar inhoud. Deze groep is alleen onderdeel van de weergave.';
  if(node.kind==='collection'&&!node.concept)desc=collectionDescriptions[node.id.split(':').at(-1)]||newStreamDescriptions.collection;
  if(node.kind==='entity'&&!node.concept)desc=entityDescriptions[node.entity?.kind]||newStreamDescriptions.entity;
  if(node.kind==='field'&&!node.concept){
    const owner=model.byId.get(node.ownerId)?.entity,path=node.fieldPath.map(part=>/^\d+$/.test(part)?'*':part).join('.');
    desc=fieldDescriptions[owner?.kind]?.[path]||newStreamDescriptions.field;
  }
  if(node.concept&&node.kind==='collection')desc='Een mogelijke eigen typed collectie voor dit entiteittype. Andere CAD-entiteittypen kunnen op dezelfde manier hun eigen plaats krijgen; deze voorbeelden zijn geen uitputtende lijst.';
  if(node.concept&&node.entity?.kind==='dimension')desc='Een mogelijke native dimension bewaart maatsoort, definitiepunten, maatlijnpositie en stijl. Alleen afgeleide lijnen en tekst bewaren zou de maatbetekenis verliezen.';
  content.innerHTML=`<h2>${escape(node.label)}</h2><div class="selection-id">${escape(node.fieldPath?.join('.')||node.raw?.path||node.resourceId||node.subtitle)}</div>${node.concept?'<div class="concept-banner">◇ Concept — illustratief, geen vastgesteld schema</div>':''}<p class="selection-description">${escape(desc)}</p>${node.expandable?`<button class="expand-action" data-toggle="${escape(id)}">${collapsed.has(id)?'＋ Structuur uitklappen':'− Structuur inklappen'}</button>`:''}`;
  let html=node.kind==='collection'?'':graphPager(node,model),details='';
  if(node.kind==='group')html+=collectionBrowser(node,model);
  if(node.kind==='DrawingSet')details+=rawDetails('IFCX-documentheader',model.fixture.ifcx.header)+rawDetails('IFCX-imports',model.fixture.ifcx.imports);
  if(node.kind==='Drawing'){
    html+=definitionList([['plotStyleMode',node.raw.attributes?.plotStyleMode??'—']]);
    if(!Object.hasOwn(node.raw.children||{},'Layers')&&!Object.hasOwn(node.raw.children||{},'Appearances'))html+='<p class="structure-note">Deze tekening gebruikt een ouder pakketcontract. Layers en Appearances staan daarin niet als children van Drawing; de graph toont alleen relaties die in IFCX zijn opgeslagen.</p>';
  }
  if(node.kind==='DrawingLayout'){
    const a=node.raw.attributes||{};
    html+=definitionList([['kind',a.kind??'—'],['scopeId',a.scopeId??'—'],['paperSpaceLinetypeScaling',a.paperSpaceLinetypeScaling??'—'],['limitsChecking',a.limitsChecking??'—']]);
    if(a.plotSettings){html+='<div class="structure-heading"><h3>Effectieve plotinstellingen</h3></div><p class="small-note">Inline waarde van deze layout; geen aparte IFCX-node.</p>';for(const key of ['media','area','mapping','output','options'])if(a.plotSettings[key])html+=rawDetails(key,a.plotSettings[key]);}
  }
  if(node.item){const {body:b,descriptor:d,storage,source}=node.item;html+=definitionList([['Versie',b.header.version],['resourceId',b.header.resourceId],['Opslag',storage],['Locatie',source]]);if(node.kind==='drawing-resource'){
      html+=`<div class="structure-heading"><h3>Resourceonderdelen</h3><span>IFCDR</span></div><div class="resource-parts">${[['header','identiteit & eenheid'],['scopeTable',b.scopeTable.length+' scope(s)'],['blockDefinitionTable',(b.blockDefinitionTable||[]).length+' definities'],['layerBindings',b.layerBindings.length+' verwijzingen naar IFCX'],['appearanceBindings',b.appearanceBindings.length+' uiterlijk-bindings'],['streamDirectory','schema’s en streamrollen'],['streams','typed entiteitkolommen & volgorde']].map(([k,v])=>`<div><code>${k}</code><span>${escape(v)}</span></div>`).join('')}</div>`;
      details+=rawDetails('Streamdirectory',b.streamDirectory)+rawDetails('Tekenvolgorde', {entityOrderStream:b.streams.entityOrderStream,entityOrderEntryStream:b.streams.entityOrderEntryStream})+rawDetails('Laag- en appearance-bindings',{layerBindings:b.layerBindings,appearanceBindings:b.appearanceBindings,appearanceOverrides:b.appearanceOverrides});
    }else{html+=definitionList([['Bron',b.source.originalFilename],['Profiel',b.source.profile],['Herkomst',b.source.resourceOrigin]]);for(const blob of b.blobs)html+=payloadStrip(node.item,blob,model);html+='<p class="small-note">Records → bindings → native inhoud. Bronbytes blijven apart van de native semantiek.</p>';}
    details+=rawDetails('Resourceverwijzing in IFCX',d);
  }
  if(node.kind==='collection')html+=collectionTable(node,model);
  if(node.kind==='block-definition'){
    const d=node.raw;
    html+=definitionList([['scopeId',d.scopeId],['name',d.name],['basePoint',JSON.stringify(d.basePoint??{x:0,y:0,z:0})],['description',d.description??''],['anonymous',d.anonymous??false],['insertionUnit',d.insertionUnit??'unitless'],['explodable',d.explodable??true],['scaling',(d.scaling??0)===0?'Any':'Uniform']]);
    html+='<p class="small-note">Ontbrekende velden tonen de logische standaardwaarde. Brongegevens hieronder blijven ongewijzigd.</p>';
  }
  if(node.kind==='field')html+=fieldDetails(node,model);
  if(node.kind==='entity'){
    const e=node.entity;html+=definitionList([['Identiteit',`(${e.resourceId}, ${e.entityId})`],['scopeId',e.scopeId],['layerId',e.layerId],['appearanceId',e.appearanceId],['Typed inhoud',e.kind]]);
    if(e.kind==='blockInstance'){
      const tr=e.geometry.transform;
      html+=definitionList([['definitionScopeId',e.geometry.definitionScopeId],['placement',JSON.stringify(tr.placement??{origin:{x:0,y:0,z:0},X:{x:1,y:0,z:0},Y:{x:0,y:1,z:0}})],['rotation (rad)',tr.rotation??0],['scale',JSON.stringify(tr.scale??{x:1,y:1,z:1})]]);
      html+='<p class="small-note">scopeId is de plaatsingsscope; definitionScopeId verwijst naar de gedeelde inhoud. Trek eerst basePoint af, pas schaal en rotatie toe en plaats daarna in het lokale frame. Ontbrekende transformvelden tonen hun standaardwaarde.</p>';
    }
    html+=`<div class="structure-heading"><h3>${e.concept?'Mogelijke typespecifieke velden':'Typespecifieke velden'}</h3></div><div class="field-list">${Object.entries(e.geometry).map(([k,v])=>`<button class="field-link" data-select="${escape('field:'+e.id+':'+k)}"><code>${escape(k)} <span aria-hidden="true">↗</span></code><span>${escape(k==='placement'&&v==='identity'?'Standaard XY-vlak · impliciet':preview(v))}</span></button>`).join('')}</div>`;
    if(e.kind==='dimension')html+='<p class="small-note">Regelmatige velden kunnen in kolommen. Variantgegevens kunnen een geregistreerde typed payload gebruiken. Die keuze is nog open; dit is geen willekeurig JSON-vangnet.</p>';
  }
  if(node.kind==='record'||node.kind==='attachment'){
    const r=node.raw,b=node.preservation.body,p=r.payload,blob=p&&b.blobs.find(b=>b.id===p.blobId);
    html+=definitionList(node.kind==='record'?[['Bronidentiteit',r.sourceObjectKey],['Brontype',r.sourceType],['Bronhandle',r.sourceHandle||'—']]:[['Attachment',r.kind],['Parent-record',r.parentRecordId]]);
    if(p){html+=definitionList([['Bytebereik',`[${p.offset}, ${sumBytes(p.offset,p.length)})`],['Lengte',p.length+' bytes']]);const bytes=model.fixture.blobs[blob?.uri];if(bytes&&Number(p.offset)+Number(p.length)<=bytes.length){const slice=bytes.slice(Number(p.offset),Number(p.offset)+Number(p.length));const text=new TextDecoder('windows-1252').decode(Uint8Array.from(slice));details+=`<details><summary>Werkelijke bronbytes</summary><pre><code>${escape(text)}</code></pre></details>`;}}
  }
  if(node.kind==='blob')html+=payloadStrip(node.preservation,node.raw,model);
  if(node.kind==='projection'){
    html+=definitionList([['Doeltype',node.raw.targetKind],['Dekking',node.raw.coverage],['Projectieschema',node.raw.projectionSchemaId]]);
    html+='<p class="small-note">De baselineFingerprint beschrijft een semantische baseline, niet de checksum van een bestand.</p>';
  }
  if(node.kind==='concept-projection')html+='<label class="reuse-check"><input type="checkbox" id="native-edited">Simuleer gewijzigde native gegevens</label><div class="reuse-result" id="reuse-result">Brondata hergebruiken vereist een passende baseline, geldig herstelbeleid en gecontroleerde afhankelijkheden.</div>';
  if(node.domain==='ifcpr')details+='<div class="concept-banner">Huidige implementatie: beperkte lezerchecks. Volledige payload-, dependency- en projectionvalidatie en preserve/restore via de converter ontbreken nog.</div>';
  if(node.kind==='field')details+=rawDetails(node.implicit?'Afgeleide standaardwaarde':node.concept?'Illustratieve veldwaarde':'Veldwaarde',node.raw);
  else if(node.raw)details+=rawDetails(node.concept?'Illustratief record — veldnamen niet vastgesteld':'Brongegevens',node.raw);
  const relations=model.edges.filter(e=>e.source===id||e.target===id);
  const relationPage=pageWindow(model,'relations:'+id,relations.length,20);
  links.innerHTML=`<details open><summary>Verbindingen <span class="relation-count">${relations.length}</span></summary>${pager('relations:'+id,relationPage)}<div class="relation-list">${relations.slice(relationPage.start,relationPage.end).map(e=>{const outgoing=e.source===id,targetId=outgoing?e.target:e.source,other=model.byId.get(targetId)||{id:targetId,label:targetId+' · '+(model.paging?.has(targetId)?'Nog niet getoond':'Onopgeloste verwijzing')};return `<button class="relation" data-select="${escape(other.id)}"><span><small>${outgoing?'→':'←'} ${escape(e.relation)}${e.concept?' · concept':''}</small>${escape(other.label)}${other.kind==='entity'?' · '+escape(other.resourceId):''}</span><i aria-hidden="true">↗</i></button>`;}).join('')}</div></details>`;
  structure.innerHTML=html;data.innerHTML=details;
  document.getElementById('selection-announcement').textContent='Geselecteerd: '+node.label+(node.concept?' · concept':'');
}
