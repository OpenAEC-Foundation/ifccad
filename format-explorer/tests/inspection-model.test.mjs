import test from 'node:test';
import assert from 'node:assert/strict';
const adapter=await import('../src/inspection-model.mjs').catch(()=>({}));
const graph={header:{},data:[{path:'/cad/d1',children:{model:'/cad/d1/layout/1'},attributes:{'ifccad::drawing':{lengthUnit:'mm'}}},{path:'/cad/d1/layout/1',children:{'0':'/cad/d1/e9007199254740993'},attributes:{'ifccad::layout':{kind:'Model'}}},{path:'/cad/d1/e9007199254740993',attributes:{'ifccad::entity':{layer:'/cad/d1/layer/1'},'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]}}},{path:'/cad/d1/layer/1',attributes:{'ifccad::layer':{name:'Walls'}}},{path:'/foreign',children:{ref:'/cad/d1/e9007199254740993',self:'/foreign'},attributes:{note:'This arbitrary /cad/string is not a reference'}}]};
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
test('OCDraw typed entities preserve scope order and actual pool provenance',()=>{
 assert.equal(typeof adapter.createInspection,'function');
 const model=adapter.createInspection({validation:{strictAvailable:true},presentation:{format:'ocdraw',entities:[{id:'9007199254740993',layerId:0,geometry:{type:'planarPolyline',vertices:[[5,6,0],[7,8,0]]}},{id:2,layerId:0,geometry:{type:'line',start:[0,0,0],end:[1,0,0]}}],layouts:[{id:0,name:'Model',scopeId:9}],scopes:[{id:9,entities:[2,'9007199254740993']}],streams:{planarPolylineStream:{entityId:[13,'9007199254740993'],vertexOffset:[0,3],vertexCount:[3,2],x:[0,0,0,5,7]},lineStream:{entityId:[2]}}}},'{}');
 assert.deepEqual(model.nodes.get('layout:0').children,['entity:2','entity:9007199254740993']);
 assert.deepEqual(model.nodes.get('entity:9007199254740993').storage,{stream:'planarPolylineStream',row:1,offset:3,count:2});
 assert.equal(model.nodes.get('entity:9007199254740993').values.geometry.vertices[0][0],5);
});
