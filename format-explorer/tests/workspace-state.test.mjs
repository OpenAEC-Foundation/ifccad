import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {buildModel,revealNode,visibleGraph,graphConnections,prepareWorkspaceExample} from '../src/model.mjs';
import {workspaceSummary} from '../src/workspace-state.mjs';
import {renderInspector} from '../src/inspector.mjs';
import {t} from '../src/i18n.mjs';

async function packageWithWorkspace(){
 const root=new URL('../../conformance/next/packages/valid/layout-viewport-plot/',import.meta.url);
 const ifcx=JSON.parse(await readFile(new URL('package.ifcx.json',root),'utf8'));
 const representation=ifcx.data.find(node=>node.type==='openaec:DrawingRepresentation');
 const uri=representation.attributes.resource.uri;
 const drawing=JSON.parse(await readFile(new URL(uri,root),'utf8'));
 const named={kind:1,ucsId:7},world={kind:0};
 const frame={origin:{x:1,y:2,z:0},X:{x:1,y:0,z:0},Y:{x:0,y:1,z:0}};
 ifcx.data.find(node=>node.type==='openaec:Drawing').attributes.workspaceState={currentLayer:'layer-1'};
 ifcx.data.push({path:'workspace-0',type:'openaec:PackageWorkspaceState',attributes:{activeDrawing:'drawing-0',activeLayout:'layout-1'}});
 drawing.drawingViewState={currentModelUcs:named,activeModelWindowId:24};
 drawing.ucsDefinitionTable=[{ucsId:7,name:'Working',frame,elevation:0}];
 drawing.modelWindowTable=Array.from({length:24},(_,i)=>({modelWindowId:i+1,rectangle:{minX:0,minY:0,maxX:1,maxY:1},view:{height:100},aspectRatio:1,renderMode:0,grid:{enabled:false},snap:{enabled:true},storedUcs:i===23?named:world,useStoredUcs:true}));
 drawing.paperCanvasTable=[{scopeId:1,view:{height:100},grid:{enabled:false},snap:{enabled:true},storedUcs:world,currentUcs:named,activeContext:{kind:1,viewportEntityId:1}}];
 drawing.viewportWorkspaceTable=[{viewportEntityId:1,grid:{enabled:false},snap:{enabled:true},storedUcs:named,useStoredUcs:true}];
 return {name:'workspace-preview',ifcx,files:{[uri]:drawing},blobs:{}};
}

const hasEdge=(model,source,target,relation)=>model.edges.some(edge=>edge.source===source&&edge.target===target&&edge.relation===relation);

test('package resume and current layer remain real IFCX references',async()=>{
 const fixture=await packageWithWorkspace(),before=JSON.stringify(fixture),model=buildModel(fixture);
 assert.ok(model.roots.includes('ifcx:workspace-0'));
 assert.ok(hasEdge(model,'ifcx:workspace-0','ifcx:drawing-0','activeDrawing'));
 assert.ok(hasEdge(model,'ifcx:workspace-0','ifcx:layout-1','activeLayout'));
 assert.ok(hasEdge(model,'ifcx:drawing-0','ifcx:layer-1','workspaceState.currentLayer'));
 assert.ok(model.edges.filter(edge=>edge.target==='ifcx:workspace-0').every(edge=>!edge.structural));
 assert.equal(JSON.stringify(fixture),before);
});

test('the active layout stays inspectable without a long default graph reference',async()=>{
 const model=buildModel(await packageWithWorkspace()),collapsed=new Set(model.defaultCollapsed);
 collapsed.delete('view:ifcx:drawing-0:Layouts');
 const visible=visibleGraph(model,collapsed);
 const activeLayout=edge=>edge.source==='ifcx:workspace-0'&&edge.target==='ifcx:layout-1'&&edge.relation==='activeLayout';
 assert.ok(model.edges.some(activeLayout));
 assert.ok(visible.edges.some(activeLayout));
 assert.ok(!graphConnections(visible,'ifcx:workspace-0').edges.some(activeLayout));
 assert.ok(graphConnections(visible,'ifcx:workspace-0','all').edges.some(activeLayout));
});

test('IFCDR workspace tables are collapsed, paged and link to native scopes and viewports',async()=>{
 const model=buildModel(await packageWithWorkspace()),rid='geometry',group=`view:ifcdr:${rid}:workspace`;
 assert.ok(model.byId.has(group));
 assert.ok(model.defaultCollapsed.has(group));
 assert.ok(!visibleGraph(model,new Set(model.defaultCollapsed)).nodes.some(node=>node.id.startsWith(`workspace:${rid}:`)));
 assert.ok(hasEdge(model,`workspace:${rid}:drawing-view-state`,`workspace:${rid}:model-window:24`,'activeModelWindowId'));
 assert.ok(hasEdge(model,`workspace:${rid}:drawing-view-state`,`workspace:${rid}:ucs:7`,'currentModelUcs.ucsId'));
 assert.ok(hasEdge(model,`workspace:${rid}:paper-canvas:1`,`scope:${rid}:1`,'scopeId'));
 assert.ok(hasEdge(model,`workspace:${rid}:paper-canvas:1`,`entity:${rid}:1`,'activeContext.viewportEntityId'));
 assert.ok(hasEdge(model,`workspace:${rid}:viewport:1`,`entity:${rid}:1`,'viewportEntityId'));
 const last=`workspace:${rid}:model-window:24`,collapsed=new Set(model.defaultCollapsed);
 assert.ok(model.paging.has(last));
 assert.ok(revealNode(model,collapsed,last));
 const shown=visibleGraph(model,collapsed).nodes;
 assert.ok(shown.some(node=>node.id===last));
 assert.ok(shown.filter(node=>node.kind==='model-window').length<=10);
 assert.equal(model.missing.length,0);
});

test('packages without workspace state keep the existing graph',async()=>{
 const fixture=await packageWithWorkspace();
 fixture.ifcx.data=fixture.ifcx.data.filter(node=>node.type!=='openaec:PackageWorkspaceState');
 delete fixture.ifcx.data.find(node=>node.type==='openaec:Drawing').attributes.workspaceState;
 const drawing=Object.values(fixture.files)[0];
 for(const key of ['drawingViewState','ucsDefinitionTable','modelWindowTable','paperCanvasTable','viewportWorkspaceTable'])delete drawing[key];
 const model=buildModel(fixture);
 assert.ok(!model.nodes.some(node=>node.id.includes('workspace')));
});

test('workspace inspection resolves named UCS and distinguishes editor contexts',async()=>{
 const model=buildModel(await packageWithWorkspace()),body=model.resources[0].body;
 model.paging.materialize('workspace:geometry:model-window:24');
 const summary=id=>Object.fromEntries(workspaceSummary(model.byId.get(id),body));
 assert.equal(summary('workspace:geometry:drawing-view-state')['Huidige model-UCS'],'Benoemd · Working (#7)');
 assert.equal(summary('workspace:geometry:model-window:24')['Opgeslagen UCS'],'Benoemd · Working (#7)');
 assert.equal(summary('workspace:geometry:paper-canvas:1')['Actieve context'],'Viewport #1');
 assert.equal(summary('workspace:geometry:viewport:1')['Viewport-entiteit'],'#1');
 assert.equal(t('Werkruimtestatus','en'),'Workspace state');
 assert.equal(t(summary('workspace:geometry:drawing-view-state')['Huidige model-UCS'],'en'),'Named · Working (#7)');
 assert.equal(t('actief modelvenster 1','en'),'active model window 1');
});

test('workspace records expose their stored fields as expandable graph nodes and inspector links',async()=>{
 const fixture=await packageWithWorkspace(),before=JSON.stringify(fixture),model=buildModel(fixture);
 const ucs='workspace:geometry:ucs:7',window='workspace:geometry:model-window:24';
 model.paging.materialize(window);
 for(const owner of [ucs,window,'workspace:geometry:drawing-view-state','workspace:geometry:paper-canvas:1','workspace:geometry:viewport:1']){
  assert.ok(model.defaultCollapsed.has(owner),owner);
  for(const [key,value] of Object.entries(model.byId.get(owner).raw)){
   const field=`field:${owner}:${key}`;
   assert.equal(model.byId.get(field)?.raw,value,field);
   assert.ok(hasEdge(model,owner,field,key),field);
  }
 }
 const nested=`field:${ucs}:frame.origin`;
 assert.deepEqual(model.byId.get(nested)?.raw,{x:1,y:2,z:0});
 const collapsed=new Set(model.defaultCollapsed);
 assert.ok(revealNode(model,collapsed,nested));
 assert.ok(visibleGraph(model,collapsed).nodes.some(node=>node.id===nested));
 const elements=new Map(),previous=globalThis.document;
 globalThis.document={getElementById:id=>{if(!elements.has(id))elements.set(id,{});return elements.get(id);}};
 try{
  renderInspector(model,window,collapsed);
  assert.match(elements.get('selection-structure').innerHTML,/field:workspace:geometry:model-window:24:grid/);
  renderInspector(model,nested,collapsed);
  assert.match(elements.get('selection-content').innerHTML,/frame\.origin/);
  assert.match(elements.get('selection-structure').innerHTML,/Werkruimtestatus uit het pakket/);
 }finally{globalThis.document=previous;}
 assert.equal(model.missing.length,0);
 assert.equal(JSON.stringify(fixture),before);
});

test('workspace example starts with its editor-state branch in view',async()=>{
 const model=buildModel(await packageWithWorkspace()),collapsed=new Set(model.defaultCollapsed);
 const focus=prepareWorkspaceExample(model,collapsed);
 assert.equal(focus.selected,'view:ifcdr:geometry:workspace');
 assert.deepEqual(focus.frame,['view:ifcdr:geometry:workspace','workspace:geometry:drawing-view-state']);
 assert.ok(visibleGraph(model,collapsed).nodes.some(node=>node.id==='workspace:geometry:drawing-view-state'));
});
