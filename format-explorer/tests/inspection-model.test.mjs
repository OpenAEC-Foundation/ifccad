import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from './support/inspection-fixture.mjs';


test('IFCCAD marker paths keep full-width source identities and independent bounds evidence',()=>{
 const source='/cad/d1/symbolSource/9007199254740993',style='/cad/d1/textStyle/7',pattern='/cad/d1/linePattern/3',layout='/cad/d1/layout/1';
 const evidence={scopePath:layout,storedQuality:'enclosing',quality:'partial',enclosureVerified:false,safeForNegativeQuery:false,linePatternReasons:['SymbolFootprintUnavailable']};
 const graph={data:[{path:source,attributes:{'ifccad::symbolSource':{kind:'shx',fileName:'Custom.SHX'}}},{path:style,attributes:{'ifccad::textStyle':{name:'Requested'}}},{path:pattern,attributes:{'ifccad::linePattern':{name:'Mixed',pattern:[{length:0,marker:{kind:'text',textStyle:style}},{length:1,marker:{kind:'symbol',symbolSource:source,shapeNumber:42}}]}}},{path:layout,attributes:{'ifccad::layout':{kind:'Model'}}}]};
 const model=createInspection({presentation:{format:'ifccad',graph,boundsCompleteness:[evidence]}});
 assert.deepEqual(model.nodes.get(pattern).outgoing.map(e=>e.target),[style,source]);
 assert.deepEqual(model.nodes.get('group:symbolSource').children,[source]);
 assert.deepEqual(model.nodes.get(layout).values.boundsAssessment,evidence);
 assert.equal(model.nodes.get(source).title,'Custom.SHX');
});
const adapter=await import('../src/inspection-model.mjs').catch(()=>({}));
const graph={header:{},data:[{path:'/cad/d1',children:{model:'/cad/d1/layout/1'},attributes:{'ifccad::drawing':{lengthUnit:'mm'}}},{path:'/cad/d1/layout/1',children:{'0':'/cad/d1/e9007199254740993'},attributes:{'ifccad::layout':{kind:'Model'}}},{path:'/cad/d1/e9007199254740993',attributes:{'ifccad::entity':{layer:'/cad/d1/layer/1'},'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]}}},{path:'/cad/d1/layer/1',attributes:{'ifccad::layer':{name:'Walls'}}},{path:'/foreign',children:{ref:'/cad/d1/e9007199254740993',self:'/foreign'},attributes:{note:'This arbitrary /cad/string is not a reference'}}]};
test('IFCCAD text style paths and glyph bounds evidence stay independent',()=>{
 const g=structuredClone(graph),entity=g.data[2],style='/cad/d1/textStyle/9007199254740997';
 delete entity.attributes['ifccad::geom::lineSegment'];entity.attributes['ifccad::mText']={style,height:2,content:[{inlines:[{kind:'run',text:'Literal <script>'}]}]};
 g.data.push({path:style,attributes:{'ifccad::textStyle':{name:'Requested',font:{family:'Face'}}}});
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:g,boundsCompleteness:[{scopePath:'/cad/d1/layout/1',quality:'estimated',enclosureVerified:false,safeForNegativeQuery:false}]}},JSON.stringify(g));
 assert.equal(model.nodes.get(entity.path).type,'mText');assert.equal(model.nodes.get(entity.path).outgoing.some(e=>e.target===style&&e.kind==='style'),true);
 assert.equal(model.nodes.get('/cad/d1/layout/1').values.boundsAssessment.quality,'estimated');assert.equal(model.nodes.get('/cad/d1/layout/1').values.boundsAssessment.safeForNegativeQuery,false);assert.equal(model.roots.includes('group:textStyle'),true);
});

test('Paper coordinates never inherit the physical drawing unit',()=>{
 const paperGraph=structuredClone(graph);
 paperGraph.data[1].attributes['ifccad::layout'].kind='Paper';
 const native=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:paperGraph}},JSON.stringify(paperGraph));
 assert.equal(native.nodes.get('/cad/d1/layout/1').unit,null);
 assert.equal(native.nodes.get('/cad/d1/e9007199254740993').unit,null);

});
test('failed IFCCAD validation retains its format and never fabricates validated CAD records',()=>{
 const model=adapter.createInspection({source:{format:'ifccad'},validation:{strictAvailable:false},presentation:null},JSON.stringify(graph));
 assert.equal(model.format,'ifccad');assert.equal(model.valid,false);assert.equal(model.nodes.has('/cad/d1/e9007199254740993'),false);assert.equal(model.nodes.has('drawing'),false);
});
test('one IFCCAD record underlies drawing and node views, with ordered source contributions',()=>{
 assert.equal(typeof adapter.createInspection,'function');
 const source=JSON.stringify({...graph,data:[...graph.data,{path:'/cad/d1/e9007199254740993',attributes:{'example::note':'later'}}]});
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph}},source);
 const node=model.nodes.get('/cad/d1/e9007199254740993');
 assert.equal(node.fragments.length,2);assert.equal(node.fragments[1].index,5);
 assert.deepEqual(model.nodes.get('/cad/d1/layout/1').children,['/cad/d1/e9007199254740993']);
 assert.equal(node.incoming.some(link=>link.source==='/foreign'),true);
 assert.equal(model.nodes.get('/foreign').outgoing.length,2);
 assert.equal(model.nodeRoots.includes(node.key),true);
 assert.equal(node.drawPosition,0);assert.equal(node.owner,'/cad/d1/layout/1');
 const owner=model.nodes.get('/cad/d1/layout/1');assert.equal(owner.groupedChildren.length,1);assert.deepEqual(model.nodes.get(owner.groupedChildren[0]).children,owner.children);
});


test('IFCCAD opaque rows expose preservation links and retain mixed draw order',()=>{
 const g=structuredClone(graph),e=g.data.find(n=>n.attributes?.['ifccad::entity']);
 e.attributes={'ifccad::opaqueEntity':{preservationRecord:'/cad/d1/preservation/r1',visible:true}};
 g.data.push({path:'/cad/d1/preservation',children:{r1:'/cad/d1/preservation/r1'},attributes:{'ifccad::preservation':{version:1,sources:[]}}},{path:'/cad/d1/preservation/r1',attributes:{'ifccad::preservationRecord':{subject:{role:'entity',path:e.path},bindings:[],conditions:[],payload:{schema:'future',version:1,kind:'adapterSnapshot',bytes:'AA=='}}}});
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:g}},JSON.stringify(g));
 assert.equal(model.nodes.get(e.path).type,'opaque');assert.equal(model.nodes.get(e.path).outgoing.some(l=>l.target==='/cad/d1/preservation/r1'),true);
 assert.equal(model.nodes.get('/cad/d1/preservation/r1').outgoing.some(l=>l.target===e.path),true);
 assert.deepEqual(model.nodes.get('/cad/d1/layout/1').children,[e.path]);assert.equal(model.roots.includes('group:preservationRecord'),true);
});
test('IFCCAD workspace links are explicit and full-width identities remain exact',()=>{
 const u='/cad/d1/ucs/9007199254740993',w='/cad/d1/modelWindow/9007199254740993',paper='/cad/d1/layout/2',viewport='/cad/d1/e3';
 const data=[{path:'/cad/d1',attributes:{'ifccad::drawing':{lengthUnit:'m',modelWindows:[w]},'ifccad::modelViewState':{currentModelUcs:{kind:'Named',ucs:u},activeModelWindow:w}}},{path:u,attributes:{'ifccad::ucsDefinition':{name:'Reference plane'}}},{path:w,attributes:{'ifccad::modelWindow':{storedUcs:{kind:'Named',ucs:u},useStoredUcs:false}}},{path:paper,attributes:{'ifccad::layout':{kind:'Paper'},'ifccad::paperCanvas':{storedUcs:{kind:'World'},useStoredUcs:false}}},{path:viewport,attributes:{'ifccad::entity':{},'ifccad::viewport':{model:'/cad/d1/layout/1'},'ifccad::viewportWorkspace':{storedUcs:{kind:'Named',ucs:u},useStoredUcs:false},note:u}}];
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',unit:'m',graph:{data}}});
 assert.ok(model.roots.includes('group:ucsDefinition')); assert.ok(model.roots.includes('group:modelWindow'));
 assert.ok(model.nodes.get('/cad/d1').outgoing.some(e=>e.target===w&&e.field==='ifccad::modelViewState.activeModelWindow'));
 assert.ok(model.nodes.get(viewport).outgoing.some(e=>e.target===u&&e.field==='ifccad::viewportWorkspace.storedUcs'));
 assert.ok(!model.nodes.get(viewport).outgoing.some(e=>e.field==='note'));
 assert.equal(model.nodes.get(viewport).workspace.coordinateDomain,'Model');
 assert.equal(model.nodes.get(viewport).workspace.unit,'m');
 assert.equal(model.nodes.get(paper).workspace.coordinateDomain,'Paper');
 assert.equal(model.nodes.get(paper).workspace.unit,null);
 assert.equal(model.nodes.get(paper).workspaceChoices.activeContext,null);
 assert.equal(model.nodes.get(paper).workspaceChoices.currentUcs,null);
});


test('IFCCAD relational override rows expose their own layer and pattern references',()=>{
 const graph={data:[
  {path:'/cad/d1',attributes:{'ifccad::drawing':{lengthUnit:'mm',pointDisplay:{form:{glyph:'plus',circle:true,square:false},size:{kind:'viewportPercent',value:2}}}}},
  {path:'/cad/d1/l1',attributes:{'ifccad::layer':{name:'Layer'}}},
  {path:'/cad/d1/p1',attributes:{'ifccad::linePattern':{name:'Pattern'}}},
  {path:'/cad/d1/e1',attributes:{'ifccad::entity':{layer:'/cad/d1/l1',visible:false},'ifccad::viewport':{model:'/cad/d1/model',plotShadingOverride:'Hidden',layerOverrides:[{layer:'/cad/d1/l1',frozen:true,linePattern:'/cad/d1/p1',color:{rgb:[255,0,0],indexedColor:{system:'ACI',index:1}}}]}}}
 ]};
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph}},JSON.stringify(graph));
 const view=model.nodes.get('/cad/d1/e1');
 assert.ok(view.outgoing.some(e=>e.field==='ifccad::viewport.layerOverrides.0.layer'&&e.target==='/cad/d1/l1'));
 assert.ok(view.outgoing.some(e=>e.field==='ifccad::viewport.layerOverrides.0.linePattern'&&e.target==='/cad/d1/p1'));
 assert.equal(view.values['ifccad::viewport'].plotShadingOverride,'Hidden');
 assert.equal(view.values['ifccad::entity'].visible,false);
 assert.equal(model.nodes.get('/cad/d1').values['ifccad::drawing'].pointDisplay.form.glyph,'plus');
});
