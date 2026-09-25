import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { buildModel, visibleGraph, revealNode } from '../src/model.mjs';

async function fixture(name) {
  const base = new URL(`../../conformance/next/packages/valid/${name}/`, import.meta.url);
  const ifcx = JSON.parse(await readFile(new URL('package.ifcx.json', base), 'utf8'));
  const files = {};
  for (const n of ifcx.data) {
    const d = n.attributes?.resource || n.attributes?.preservation;
    if (d?.uri) files[d.uri] = JSON.parse(await readFile(new URL(d.uri, base), 'utf8'));
  }
  return { name, ifcx, files, blobs: {} };
}

test('paper layouts connect to their IFCDR scopes and viewport references stay distinct', async () => {
  const f=await fixture('layout-viewport-plot'),before=JSON.stringify(f),m=buildModel(f);
  const paper='ifcx:layout-1',model='ifcx:layout-0';
  assert.ok(m.byId.get(paper).y>m.byId.get(model).y);
  assert.ok(m.byId.get('ifcx:layer-0').y>m.byId.get(paper).y);
  assert.ok(m.edges.some(e=>e.source===paper&&e.target==='scope:geometry:1'&&e.relation==='scopeId'));
  assert.ok(m.edges.some(e=>e.source===model&&e.target==='scope:geometry:0'&&e.relation==='scopeId'));
  assert.equal(m.byId.get(paper).raw.attributes.plotSettings.media.mediaName,'A4');
  const collection=m.byId.get('collection:geometry:viewport');
  assert.equal(collection.raw.count,2);
  assert.equal(m.byId.has('collection:geometry:line'),false);
  assert.equal(m.byId.has('collection:geometry:blockInstance'),false);
  assert.equal(m.byId.get('entity:geometry:1').entity.kind,'viewport');
  assert.equal(m.byId.get('entity:geometry:3').entity.geometry.paperClip.boundaryEntityId,2);
  assert.ok(m.edges.some(e=>e.source==='entity:geometry:1'&&e.target==='scope:geometry:0'&&e.relation==='viewScopeId'));
  assert.ok(m.edges.some(e=>e.source==='entity:geometry:3'&&e.target==='entity:geometry:2'&&e.relation==='paperClip.boundaryEntityId'));
  assert.deepEqual(m.byId.get('entity:geometry:1').entity.geometry.layerOverrides,[
    {layerId:0,frozen:true,appearanceOverrideId:1},
    {layerId:1,frozen:false,appearanceOverrideId:2},
  ]);
  assert.equal(m.missing.length,0);
  assert.equal(JSON.stringify(f),before);
});

test('a newly declared object stream is browsable without a family-specific adapter', async () => {
  // Presentation-only probe: the current production reader does not claim this stream.
  const f=await fixture('unrepresented-packed'),body=f.files['drawing.ifcdr.json'];
  const count=12,ids=Array.from({length:count},(_,i)=>100+i);
  body.streamDirectory.streams.push({name:'futureShape',schema:'example.futureShape.v1',role:'object',count,columns:['entityId','scopeId','radius','placement','layerId','appearanceId']});
  body.streams.futureShapeStream={count,entityId:ids,scopeId:ids.map(()=>0),radius:ids.map((_,i)=>i+1),placement:ids.map((_,i)=>({origin:{x:i,y:0,z:0}})),layerId:ids.map(()=>0),appearanceId:ids.map(()=>0)};
  const before=JSON.stringify(f),model=buildModel(f),collection='collection:drawing-main:futureShape',last='entity:drawing-main:111';
  assert.ok(model.byId.has(collection));
  assert.equal(model.byId.get(collection).raw.count,count);
  assert.ok(model.paging.has(last));
  assert.equal(model.byId.has(last),false);
  const collapsed=new Set(model.defaultCollapsed);
  assert.ok(revealNode(model,collapsed,last));
  assert.equal(model.byId.get(last).entity.kind,'futureShape');
  assert.deepEqual(model.byId.get(last).entity.geometry,{radius:12,placement:{origin:{x:11,y:0,z:0}}});
  assert.equal(model.byId.get('field:'+last+':radius').raw,12);
  assert.ok(model.edges.some(e=>e.source===last&&e.target==='scope:drawing-main:0'&&e.relation==='scopeId'));
  assert.ok(model.edges.some(e=>e.source===last&&e.target==='ifcx:layer-0'&&e.relation==='layerBinding'));
  assert.ok(visibleGraph(model,collapsed).nodes.some(n=>n.id===last));
  assert.ok(visibleGraph(model,collapsed).nodes.filter(n=>n.kind==='entity'&&n.entity?.kind==='futureShape').length<=10);
  assert.equal(model.missing.length,0);
  assert.equal(JSON.stringify(f),before);
});

test('local blocks distinguish owning scope, definition scope and stored transform defaults', async () => {
  const f=await fixture('block-empty-defaults'),before=JSON.stringify(f),m=buildModel(f);
  const e=m.byId.get('entity:drawing-main:3');
  assert.equal(e?.entity.kind,'blockInstance');
  assert.equal(m.byId.get('scope:drawing-main:7').label,'Model space');
  assert.equal(m.byId.get('scope:drawing-main:21').label,'Block definition scope');
  assert.equal(m.byId.get('block-definition:drawing-main:21').raw.name,'Door');
  assert.ok(m.edges.some(r=>r.source===e.id&&r.target==='scope:drawing-main:7'&&r.relation==='scopeId'));
  assert.ok(m.edges.some(r=>r.source===e.id&&r.target==='scope:drawing-main:21'&&r.relation==='definitionScopeId'));
  assert.deepEqual(e.entity.geometry.transform,{});
  assert.equal(m.missing.length,0);
  assert.ok(revealNode(m,new Set(m.defaultCollapsed),'block-definition:drawing-main:21'));
  assert.equal(JSON.stringify(f),before);
});

test('many IFCX definitions stay bounded and can be revealed from references', async () => {
 const f=await fixture('unrepresented-packed');
 const layer=f.ifcx.data.find(n=>n.type==='openaec:Layer');
 const appearance=f.ifcx.data.find(n=>n.type==='openaec:Appearance');
 for(let i=0;i<250;i++){
  f.ifcx.data.push({...layer,path:'extra-layer-'+i,attributes:{...layer.attributes,appearance:'extra-appearance-'+i}});
  f.ifcx.data.push({...appearance,path:'extra-appearance-'+i});
 }
 const original=JSON.stringify(f),model=buildModel(f),collapsed=new Set(model.defaultCollapsed);
 assert.ok(visibleGraph(model,collapsed).nodes.length<30);
 for(const id of ['ifcx:extra-layer-249','ifcx:extra-appearance-249','ifcx:extra-layer-10']){
  assert.ok(revealNode(model,collapsed,id));
  const shown=visibleGraph(model,collapsed).nodes;
  assert.ok(shown.some(n=>n.id===id));
  assert.ok(shown.filter(n=>n.kind==='Layer').length<=10);
  assert.ok(shown.filter(n=>n.kind==='Appearance').length<=10);
 }
 assert.equal(JSON.stringify(f),original);
});

test('large block tables stay paged and reveal a referenced scope and definition', async () => {
 const f=await fixture('block-empty-defaults');
 const drawing=f.files['drawing.ifcdr.json'];
 const scope=drawing.scopeTable.find(item=>item.kind===2);
 const definition=drawing.blockDefinitionTable[0];
 for(let i=0;i<35;i++){
  drawing.scopeTable.push({...scope,id:100+i});
  drawing.blockDefinitionTable.push({...definition,scopeId:100+i,name:`Block ${i}`});
 }
 const before=JSON.stringify(f),model=buildModel(f),collapsed=new Set(model.defaultCollapsed);
 const scopes='view:ifcdr:drawing-main:scopeTable',definitions='view:ifcdr:drawing-main:blockDefinitionTable';
 assert.ok(model.byId.has(scopes));
 assert.ok(model.byId.has(definitions));
 assert.ok(collapsed.has(scopes)&&collapsed.has(definitions));
 assert.equal(visibleGraph(model,collapsed).nodes.filter(n=>n.kind==='scope'||n.kind==='block-definition').length,0);
 assert.equal(model.missing.length,0);
 assert.equal(model.edges.filter(e=>e.relation==='scopeId'&&e.source.startsWith('block-definition:')).length,drawing.blockDefinitionTable.length);
 assert.ok(revealNode(model,collapsed,'block-definition:drawing-main:134'));
 assert.ok(revealNode(model,collapsed,'scope:drawing-main:134'));
 const shown=visibleGraph(model,collapsed).nodes;
 assert.ok(shown.some(n=>n.id==='block-definition:drawing-main:134'));
 assert.ok(shown.some(n=>n.id==='scope:drawing-main:134'));
 assert.ok(shown.filter(n=>n.kind==='scope').length<=10);
 assert.ok(shown.filter(n=>n.kind==='block-definition').length<=10);
 assert.ok(model.edges.some(e=>e.source==='block-definition:drawing-main:134'&&e.target==='scope:drawing-main:134'&&e.relation==='scopeId'));
 assert.equal(JSON.stringify(f),before);
});

test('focus view keeps package structure and resource bridges while bounding references', async () => {
 const {graphConnections}=await import('../src/model.mjs');
 assert.equal(typeof graphConnections,'function');
 const structural={source:'resource:main',target:'view:scopes',relation:'scopeTable',structural:true};
 const bridge={source:'resource:preservation',target:'resource:main',relation:'linkedDrawingResources',structural:false};
 const references=Array.from({length:15},(_,i)=>({source:`entity:${i}`,target:'ifcx:layer',relation:'layerBinding',structural:false}));
 const visible={edges:[structural,bridge,...references]};
 const overview=graphConnections(visible,'resource:main');
 assert.deepEqual(overview.edges,[structural,bridge]);
 const focused=graphConnections(visible,'ifcx:layer');
 assert.deepEqual(focused.edges.slice(0,2),[structural,bridge]);
 assert.equal(focused.edges.length,10);
 assert.equal(focused.hiddenReferences,7);
 assert.deepEqual(graphConnections(visible,'ifcx:layer','all').edges,visible.edges);
});

test('external and inline resources retain identical resource-qualified entity identities', async () => {
  const external = buildModel(await fixture('unrepresented-packed'));
  const inline = buildModel(await fixture('inline-both'));
  assert.deepEqual(external.entities.map(e => e.id), inline.entities.map(e => e.id));
  assert.equal(external.entities.length, 4);
  assert.equal(inline.resources[0].storage, 'inline');
  assert.equal(external.entities[0].layer, 'ifcx:layer-0');
});

test('same entity ID in two resources remains distinct and projections resolve to both', async () => {
  const model = buildModel(await fixture('multi-drawing-projections'));
  assert.equal(model.entities.filter(e => e.entityId === 1).length, 2);
  assert.equal(new Set(model.entities.map(e => e.id)).size, 8);
  const targets = model.edges.filter(e => e.relation === 'modelTarget').map(e => e.target);
  assert.ok(targets.includes('entity:drawing-main:1'));
  assert.ok(targets.includes('entity:geometry-second:2'));
});

test('placed polyline evaluates O + xX + yY without adding scope base metadata', async () => {
  const f = await fixture('tilted-plane');
  f.files['drawing.ifcdr.json'].scopeTable[0].baseX = 1000;
  const model = buildModel(f);
  assert.deepEqual(model.entities.find(e => e.entityId === 3).points, [[0,0,0],[0,10,0],[0,10,5],[0,0,5]]);
});

test('folding and revealing resources preserves shared graph identities', async () => {
  const model = buildModel(await fixture('unrepresented-packed'));
  const collapsed = new Set(model.defaultCollapsed);
  assert.ok(!visibleGraph(model, collapsed).nodes.some(n => n.id === 'entity:drawing-main:1'));
  revealNode(model, collapsed, 'entity:drawing-main:1');
  const shown = visibleGraph(model, collapsed);
  assert.ok(shown.nodes.some(n => n.id === 'entity:drawing-main:1'));
  assert.equal(shown.nodes.filter(n => n.id === 'ifcx:drawing-representation-main').length, 1);
  assert.ok(shown.edges.filter(e => e.target === 'ifcx:drawing-representation-main').length >= 2);
});

test('concept entities and preservation are separate from unchanged native fixture data', async () => {
  const f = await fixture('unrepresented-packed');
  const before = JSON.stringify(f);
  const model = buildModel(f, { concepts: true });
  assert.equal(model.entities.filter(e => !e.concept).length, 4);
  assert.deepEqual(model.entities.filter(e => e.concept).map(e => e.kind), ['dimension']);
  assert.ok(model.nodes.some(n => n.kind === 'concept-record' && n.concept));
  for(const record of model.nodes.filter(n=>n.kind==='concept-record')) {
    assert.ok(model.edges.some(e=>e.target===record.id&&e.structural&&model.byId.get(e.source).domain==='ifcpr'));
  }
  assert.equal(JSON.stringify(f), before);
});

test('illustrative dimension remains separate from native IFCDR collections', async () => {
  const model = buildModel(await fixture('unrepresented-packed'), {concepts:true});
  const collections = model.nodes.filter(n => n.concept && n.kind === 'collection');
  assert.equal(collections.length, 1);
  for (const family of ['dimension']) {
    const collection = collections.find(n => n.family === family);
    assert.ok(collection);
    assert.ok(model.edges.some(e => e.structural && e.source === 'resource:drawing-main' && e.target === collection.id));
    const members = model.edges.filter(e => e.structural && e.source === collection.id);
    assert.equal(members.length, 1);
    assert.equal(model.byId.get(members[0].target).entity.kind, family);
  }
});

test('entity fields form collapsed presentation branches with real placement values', async () => {
  const model = buildModel(await fixture('tilted-plane'));
  const entity = model.byId.get('entity:drawing-main:3');
  const fields = model.edges.filter(e => e.source === entity.id && e.structural).map(e => model.byId.get(e.target));
  assert.deepEqual(fields.map(n => n.label), ['vertices', 'closed', 'placement']);
  assert.ok(model.defaultCollapsed.has(entity.id));
  assert.ok(fields.every(n => n.kind === 'field' && n.ownerId === entity.id));
  const placement = fields.find(n => n.label === 'placement');
  assert.deepEqual(placement.raw, entity.entity.geometry.placement);
  const axes = model.edges.filter(e => e.source === placement.id && e.structural).map(e => model.byId.get(e.target));
  assert.deepEqual(axes.map(n => n.label), ['origin', 'X', 'Y']);
  assert.deepEqual(axes[1].raw, {x:0,y:1,z:0});
  const collapsed = new Set(model.defaultCollapsed);
  revealNode(model, collapsed, axes[1].id);
  assert.ok(visibleGraph(model, collapsed).nodes.some(n => n.id === axes[1].id));
  collapsed.add(entity.id);
  assert.ok(!visibleGraph(model, collapsed).nodes.some(n => n.id === axes[1].id));
});

test('implicit placement and concept fields stay distinct from stored native data', async () => {
  const model = buildModel(await fixture('unrepresented-packed'), {concepts:true});
  for (const entity of model.entities) {
    const fields = model.edges.filter(e => e.structural && e.source === entity.id).map(e => model.byId.get(e.target));
    assert.deepEqual(fields.map(n => n.label), Object.keys(entity.geometry));
    assert.ok(fields.every(n => Boolean(n.concept) === Boolean(entity.concept)));
  }
  const placement = model.nodes.find(n => n.ownerId === 'entity:drawing-main:3' && n.label === 'placement');
  assert.equal(placement.implicit, true);
  assert.deepEqual(placement.raw, {origin:{x:0,y:0,z:0},X:{x:1,y:0,z:0},Y:{x:0,y:1,z:0}});
});

test('layers and appearances are real Drawing children, without a fabricated group', async () => {
  const model=buildModel(await fixture('unrepresented-packed'));
  assert.equal(model.byId.has('group:definitions'),false);
  assert.ok(!model.roots.includes('ifcx:layer-0'));
  assert.ok(!model.roots.includes('ifcx:appearance-default-solid'));
  const visible=visibleGraph(model,new Set(model.defaultCollapsed));
  assert.ok(visible.nodes.some(n=>n.id==='ifcx:appearance-default-solid'));
  assert.ok(visible.edges.some(e=>e.source==='ifcx:drawing-main'&&e.target==='ifcx:layer-0'&&e.relation==='Layers'&&e.structural));
  assert.ok(visible.edges.some(e=>e.source==='ifcx:drawing-main'&&e.target==='ifcx:appearance-default-solid'&&e.relation==='Appearances'&&e.structural));
  assert.ok(visible.edges.some(e=>e.source==='ifcx:layer-0'&&e.target==='ifcx:appearance-default-solid'));
});

test('a compact plane placement uses its default axes without rewriting stored fields',async()=>{
 const f=await fixture('tilted-plane'),stream=f.files['drawing.ifcdr.json'].streams.planarPolylineStream;
 stream.placement[0]={origin:{x:2,y:3,z:4}};
 const model=buildModel(f),entity=model.byId.get('entity:drawing-main:3').entity;
 assert.deepEqual(entity.points[0],[2,3,4]);
 assert.deepEqual(entity.points[1],[12,3,4]);
 assert.deepEqual(entity.geometry.placement,{origin:{x:2,y:3,z:4}});
});

test('IFCDR 0.11 geometry families expose exact rows and vertex pools',async()=>{
 const curves=buildModel(await fixture('ellipse-family'));
 for(const kind of ['point','circle','arc','ellipse','ellipseArc'])assert.ok(curves.byId.has('collection:drawing-main:'+kind));
 assert.deepEqual(curves.byId.get('entity:drawing-main:5').entity.geometry.placement.origin,{z:6,x:4,y:5});
 assert.equal(curves.byId.get('entity:drawing-main:7').entity.geometry.sweepParameter,-Math.PI/2);
 assert.equal(curves.byId.get('field:entity:drawing-main:9:semiMajorRadius').raw,2);
 const planar=buildModel(await fixture('planar-bulges')).byId.get('entity:drawing-main:3').entity;
 assert.deepEqual(planar.geometry.bulges,[1,0,0,0]);
 const spatial=buildModel(await fixture('spatial-polyline')).byId.get('entity:drawing-main:5').entity;
 assert.deepEqual(spatial.geometry.vertices,[[2,3,1],[6,7,5]]);
 assert.deepEqual(spatial.points,spatial.geometry.vertices);
 assert.equal(spatial.geometry.placement,undefined);
});

test('drawing-listed definitions follow Drawing expansion while an older shape remains standalone', async () => {
  const current=buildModel(await fixture('layout-viewport-plot'));
  for(const id of ['ifcx:layer-0','ifcx:layer-1','ifcx:appearance-0'])assert.ok(!current.roots.includes(id));
  assert.ok(current.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target==='ifcx:layer-0'&&e.relation==='Layers'&&e.structural));
  assert.ok(current.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target==='ifcx:appearance-0'&&e.relation==='Appearances'&&e.structural));
  const collapsed=new Set(current.defaultCollapsed);
  collapsed.add('ifcx:drawing-0');
  const hidden=visibleGraph(current,collapsed);
  assert.ok(!hidden.nodes.some(n=>n.id==='ifcx:layer-0'||n.id==='ifcx:appearance-0'));
  collapsed.delete('ifcx:drawing-0');
  const shown=visibleGraph(current,collapsed);
  assert.ok(shown.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target==='ifcx:layer-0'));
  assert.ok(shown.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target==='ifcx:appearance-0'));
  const oldSource=await fixture('unrepresented-packed');
  const oldDrawing=oldSource.ifcx.data.find(n=>n.type==='openaec:Drawing');
  delete oldDrawing.children.Layers;
  delete oldDrawing.children.Appearances;
  const legacy=buildModel(oldSource);
  assert.ok(legacy.roots.includes('ifcx:layer-0'));
  assert.ok(!legacy.edges.some(e=>e.source==='ifcx:drawing-main'&&e.relation==='Layers'));
});

test('large drawing-listed layer collections page beneath their drawing', async () => {
  const f=await fixture('layout-viewport-plot');
  const drawing=f.ifcx.data.find(n=>n.type==='openaec:Drawing');
  const source=f.ifcx.data.find(n=>n.path==='layer-0');
  for(let i=0;i<12;i++){
    const path='extra-layer-'+i;
    f.ifcx.data.push({...source,path,attributes:{...source.attributes,name:'Extra '+i}});
    drawing.children.Layers.push(path);
  }
  const model=buildModel(f),group='view:definitions:Layer';
  assert.ok(model.byId.has(group));
  assert.ok(!model.roots.includes(group));
  assert.ok(model.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target===group&&e.structural));
  const collapsed=new Set(model.defaultCollapsed);collapsed.add('ifcx:drawing-0');
  assert.ok(!visibleGraph(model,collapsed).nodes.some(n=>n.id===group));
  collapsed.delete('ifcx:drawing-0');
  assert.ok(visibleGraph(model,collapsed).nodes.some(n=>n.id===group));
  assert.ok(revealNode(model,collapsed,'ifcx:extra-layer-11'));
  assert.ok(visibleGraph(model,collapsed).nodes.some(n=>n.id==='ifcx:extra-layer-11'));
  assert.ok(model.edges.some(e=>e.source==='ifcx:drawing-0'&&e.target==='ifcx:extra-layer-11'&&e.relation==='Layers'&&!e.structural));
  collapsed.add(group);
  assert.ok(!visibleGraph(model,collapsed).nodes.some(n=>n.id==='ifcx:extra-layer-11'));
});

test('actual model keeps a large stream lazy and navigates a distant identity',async()=>{
 const f=await fixture('unrepresented-packed'),s=f.files['drawing.ifcdr.json'].streams.lineStream;
 const template=structuredClone(s);s.count=10000;
 for(const key of Object.keys(s))if(Array.isArray(s[key]))s[key]=Array.from({length:10000},(_,i)=>key==='entityId'?i+100:s[key][0]);
 const model=buildModel(f);assert.equal(model.entities.filter(e=>e.kind==='line').length,0);
 const collapsed=new Set(model.defaultCollapsed);assert.ok(revealNode(model,collapsed,'entity:drawing-main:10099'));
 assert.ok(model.byId.has('entity:drawing-main:10099'));assert.equal(model.entities.filter(e=>e.kind==='line').length,10);
 model.paging.expand('collection:drawing-main:line');assert.equal(model.entities.filter(e=>e.kind==='line').length,10);
 model.paging.setPage('collection:drawing-main:line',0);
 assert.equal(visibleGraph(model,collapsed).nodes.filter(n=>n.entity?.kind==='line').length,10);
 assert.ok(!visibleGraph(model,collapsed).nodes.some(n=>n.id==='entity:drawing-main:10099'));
 assert.ok(revealNode(model,collapsed,'field:entity:drawing-main:10099:start'));
 assert.ok(visibleGraph(model,collapsed).nodes.some(n=>n.id==='field:entity:drawing-main:10099:start'));
 assert.equal(visibleGraph(model,collapsed).nodes.filter(n=>n.entity?.kind==='line').length,10);
});

test('permitted unrelated IFCX cycles remain inspectable',async()=>{
 const f=await fixture('inline-both');f.ifcx.data.push({path:'a',type:'extension:A',children:{next:'b'}},{path:'b',type:'extension:B',children:{next:'a'}});
 const m=buildModel(f),visible=visibleGraph(m,new Set(m.defaultCollapsed));assert.ok(visible.nodes.some(n=>n.id==='ifcx:a'));assert.ok(revealNode(m,new Set(),'ifcx:b'));
});

test('extension names and unrelated resource attributes do not impersonate native nodes',async()=>{
 const f=await fixture('inline-both');f.ifcx.data.push({path:'extension',type:'constructor',attributes:{resource:'unrelated'}});
 const m=buildModel(f);assert.ok(m.byId.has('ifcx:extension'));
});
