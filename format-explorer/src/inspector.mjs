import { isVector } from './fields.mjs';
import { columnTable, collectionBrowser, graphPager, pageWindow, pager, collectionWindow } from './browsing.mjs';
const escape=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const sumBytes=(a,b)=>{try{return (BigInt(a)+BigInt(b)).toString();}catch{return '?';}};
const preview=value=>JSON.stringify(value,null,2)?.slice(0,12000);
const rawDetails=(title,data)=>{const full=JSON.stringify(data,null,2)||'';return `<details><summary>${escape(title)}${full.length>12000?' · preview':''}</summary><pre><code>${escape(full.slice(0,12000))}</code></pre>${full.length>12000?'<p class="small-note">Preview ingekort tot 12000 tekens.</p>':''}</details>`;};
const definitionList=rows=>'<dl class="meta-list">'+rows.map(([k,v])=>`<dt>${escape(k)}</dt><dd>${escape(v)}</dd>`).join('')+'</dl>';
const description={
  DrawingSet:'Het vertrekpunt van dit CAD-pakket. Verbindt tekeningen en optionele preservation-resources. Een gebouwmodel is hiervoor niet vereist.',
  Drawing:'De semantische identiteit van een tekening. Verwijst naar layouts en één gedeelde DrawingRepresentation.',
  DrawingLayout:'Een layout selecteert een scope binnen de IFCDR-resource. Drawing en layout verwijzen naar dezelfde representation-node.',
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

function collectionTable(node,model){
  if(node.concept)return `<div class="structure-note">Een afzonderlijke voorbeeldcollectie voor <code>${escape(node.family)}</code>. Circle en dimension zijn twee willekeurige voorbeelden van extra CAD-entiteittypen. Ieder type vraagt een eigen semantisch contract; de uiteindelijke streamindeling staat nog open.</div>${definitionList([['Basisvelden','entityId · scopeId · layerId · appearanceId'],[node.family,node.family==='circle'?'centrum · straal · vlak':'maatsoort · definitiepunten · maatlijnpositie · stijlref']])}`;
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
    html+=`<div class="structure-heading"><h3>Punten in volgorde</h3><span>${value.length} punten</span></div>${shared?'':pager(node.id+':points',w)}<div class="relation-list collection-list">${value.slice(w.start,w.end).map((v,i)=>`<button class="relation" data-select="${escape(node.id+'.'+(w.start+i))}"><span><small>punt ${w.start+i}</small><code>${escape(tuple(v))}</code></span><i aria-hidden="true">↗</i></button>`).join('')}</div>`;
  }
  const top=node.fieldPath[0];
  if((top==='placement'||top==='plane')&&node.fieldPath.length===2){
    const meaning={origin:'De oorsprong O: het nulpunt van het lokale vlak in XYZ.',X:'De lokale X-as: de richting waarin de lokale x-coördinaat toeneemt.',Y:'De lokale Y-as: de richting waarin de lokale y-coördinaat toeneemt.'};
    if(meaning[node.label])html+=`<p class="structure-note">${meaning[node.label]}</p>`;
  }
  if(top==='placement'&&node.fieldPath.length===1&&entity.kind==='polyline'){
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
  if(node.concept&&node.kind==='collection')desc='Een mogelijke eigen typed collectie voor dit entiteittype. Andere CAD-entiteittypen kunnen op dezelfde manier hun eigen plaats krijgen; deze voorbeelden zijn geen uitputtende lijst.';
  if(node.entity?.kind==='circle')desc='Een mogelijke native circle bewaart centrum, straal en vlak als betekenis. Deze informatie kan in een eigen typed collectie passen; veldnamen en schema zijn hier illustratief.';
  if(node.entity?.kind==='dimension')desc='Een mogelijke native dimension bewaart maatsoort, definitiepunten, maatlijnpositie en stijl. Alleen afgeleide lijnen en tekst bewaren zou de maatbetekenis verliezen.';
  content.innerHTML=`<h2>${escape(node.label)}</h2><div class="selection-id">${escape(node.fieldPath?.join('.')||node.raw?.path||node.resourceId||node.subtitle)}</div>${node.concept?'<div class="concept-banner">◇ Concept — illustratief, geen vastgesteld schema</div>':''}<p class="selection-description">${escape(desc)}</p>${node.expandable?`<button class="expand-action" data-toggle="${escape(id)}">${collapsed.has(id)?'＋ Structuur uitklappen':'− Structuur inklappen'}</button>`:''}`;
  let html=node.kind==='collection'?'':graphPager(node,model),details='';
  if(node.kind==='group')html+=collectionBrowser(node,model);
  if(node.kind==='DrawingSet')details+=rawDetails('IFCX-documentheader',model.fixture.ifcx.header)+rawDetails('IFCX-imports',model.fixture.ifcx.imports);
  if(node.item){const {body:b,descriptor:d,storage,source}=node.item;html+=definitionList([['Versie',b.header.version],['resourceId',b.header.resourceId],['Opslag',storage],['Locatie',source]]);if(node.kind==='drawing-resource'){
      html+=`<div class="structure-heading"><h3>Resourceonderdelen</h3><span>IFCDR</span></div><div class="resource-parts">${[['header','identiteit & eenheid'],['scopeTable',b.scopeTable.length+' scope(s)'],['blockDefinitionTable',(b.blockDefinitionTable||[]).length+' definities'],['layerBindings',b.layerBindings.length+' verwijzingen naar IFCX'],['appearanceBindings',b.appearanceBindings.length+' uiterlijk-bindings'],['streamDirectory','schema’s en streamrollen'],['streams','typed entiteitkolommen & volgorde']].map(([k,v])=>`<div><code>${k}</code><span>${escape(v)}</span></div>`).join('')}</div>`;
      details+=rawDetails('Streamdirectory',b.streamDirectory)+rawDetails('Tekenvolgorde', {entityOrderStream:b.streams.entityOrderStream,entityOrderEntryStream:b.streams.entityOrderEntryStream})+rawDetails('Laag- en appearance-bindings',{layerBindings:b.layerBindings,appearanceBindings:b.appearanceBindings});
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
  links.innerHTML=`<details open><summary>Verbindingen <span class="relation-count">${relations.length}</span></summary><div class="relation-list">${relations.slice(0,100).map(e=>{const outgoing=e.source===id,targetId=outgoing?e.target:e.source,other=model.byId.get(targetId)||{id:targetId,label:targetId+' · '+(model.paging?.has(targetId)?'Nog niet getoond':'Onopgeloste verwijzing')};return `<button class="relation" data-select="${escape(other.id)}"><span><small>${outgoing?'→':'←'} ${escape(e.relation)}${e.concept?' · concept':''}</small>${escape(other.label)}${other.kind==='entity'?' · '+escape(other.resourceId):''}</span><i aria-hidden="true">↗</i></button>`;}).join('')}</div></details>`;
  structure.innerHTML=html;data.innerHTML=details;
  document.getElementById('selection-announcement').textContent='Geselecteerd: '+node.label+(node.concept?' · concept':'');
}
