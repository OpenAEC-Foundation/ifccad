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
  assert.deepEqual(model.entities.filter(e => e.concept).map(e => e.kind), ['circle','dimension']);
  assert.ok(model.nodes.some(n => n.kind === 'concept-record' && n.concept));
  for(const record of model.nodes.filter(n=>n.kind==='concept-record')) {
    assert.ok(model.edges.some(e=>e.target===record.id&&e.structural&&model.byId.get(e.source).domain==='ifcpr'));
  }
  assert.equal(JSON.stringify(f), before);
});

test('illustrative CAD families each have an independent IFCDR collection', async () => {
  const model = buildModel(await fixture('unrepresented-packed'), {concepts:true});
  const collections = model.nodes.filter(n => n.concept && n.kind === 'collection');
  assert.equal(collections.length, 2);
  for (const family of ['circle', 'dimension']) {
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

test('layers and appearances are real visible IFCX roots, without a fabricated group', async () => {
  const model=buildModel(await fixture('unrepresented-packed'));
  assert.equal(model.byId.has('group:definitions'),false);
  const visible=visibleGraph(model,new Set(model.defaultCollapsed));
  assert.ok(visible.nodes.some(n=>n.id==='ifcx:appearance-default-solid'));
  assert.ok(visible.edges.some(e=>e.source==='ifcx:layer-0'&&e.target==='ifcx:appearance-default-solid'));
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
