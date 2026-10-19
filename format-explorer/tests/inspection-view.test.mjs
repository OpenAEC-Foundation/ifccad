import test from 'node:test';import assert from 'node:assert/strict';
import {createInspection} from './support/inspection-fixture.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
import * as presentation from '../src/presentation-json.mjs';

test('explicit pattern values render as references in both native formats',()=>{
 for(const format of ['ifccad']) {
  const pattern='/cad/d1/linePattern/5',entity='/cad/d1/e1';
  const p=format==='ifccad'?{format,graph:{data:[{path:entity,attributes:{'ifccad::entity':{appearance:{linePattern:{mode:'Explicit',value:pattern}}}}},{path:pattern,attributes:{'ifccad::linePattern':{name:'Mixed',pattern:[]}}}]}}:{format,linePatterns:[{id:5,name:'Mixed',pattern:[]}],entities:[{id:1,appearance:{linePattern:{mode:'Explicit',value:5}},geometry:{type:'line'}}]};
  const model=createInspection({presentation:p});
  const rendered=renderInspection(model,{language:'en',view:'drawing',selection:format==='ifccad'?entity:'entity:1',expanded:new Set(),inspector:'properties'});
  assert.ok(rendered.details.includes('data-select="'+(format==='ifccad'?pattern:'pattern:5')+'"'),rendered.details);
 }
});

test('partial symbol coverage is explained without declaring an external file missing',()=>{
 const model=createInspection({presentation:{format:'ifccad',layouts:[{id:0,scopeId:0,name:'Model',kind:'model'}],scopes:[{id:0,entities:[]}],boundsCompleteness:[{scopeId:0,declaredQuality:'enclosing',quality:'partial',enclosureVerified:false,safeForNegativeQuery:false,linePatternReasons:['SymbolFootprintUnavailable']}]}});
 const rendered=renderInspection(model,{language:'en',view:'drawing',selection:'layout:0',expanded:new Set(),inspector:'properties'});
 assert.match(rendered.details,/Symbol outlines have not been assessed/);
 assert.doesNotMatch(rendered.details,/file is missing/i);
});

test('a selected record beyond the old row budget uses a bounded middle list without losing the tree context',()=>{
 const entities=Array.from({length:300},(_,i)=>({id:i+1,geometry:{type:'line'}}));
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',entities,layouts:[{id:0,scopeId:0,kind:'model',name:'Model'}],scopes:[{id:0,entities:entities.map(e=>e.id)}]}});
 const view=renderInspection(model,{language:'en',view:'drawing',grouping:'type',selection:'entity:300',field:[],expanded:new Set(['group:layout','layout:0','group:types:layout:0:lineSegment']),limit:10,inspector:'properties'});
 assert.match(view.tree,/data-select="entity:300" aria-current="true"/);assert.ok((view.tree.match(/data-select=/g)||[]).length<25);assert.equal(view.more,false);
 assert.equal(view.collection.key,'group:types:layout:0:lineSegment');assert.equal(view.collection.keys.length,300);
});

test('singleton entity groups flatten but structural layout and resource collections remain',()=>{
 const model=createInspection({presentation:{format:'ifccad',layers:[{id:0,name:'Layer 0'}],entities:[{id:1,geometry:{type:'line'}}],layouts:[{id:0,scopeId:0,name:'Model'}],scopes:[{id:0,entities:[1]}]}});
 const view=renderInspection(model,{language:'en',view:'drawing',grouping:'type',selection:'entity:1',expanded:new Set(['group:layout','layout:0']),inspector:'properties'});
 assert.match(view.tree,/data-select="entity:1"/);assert.doesNotMatch(view.tree,/data-select="group:types:layout:0:lineSegment"/);
 assert.match(view.tree,/data-select="group:layout"/);assert.match(view.tree,/data-select="group:layer"/);
});

test('ten resources stay inline while eleven open a separate collection',()=>{
 for(const count of [10,11]){
  const model=createInspection({presentation:{format:'ifccad',layers:Array.from({length:count},(_,i)=>({id:i,name:'Layer '+i}))}});
  const view=renderInspection(model,{language:'en',view:'drawing',selection:'group:layer',expanded:new Set(['group:layer']),inspector:'properties'});
  assert.equal(view.collection?.keys.length,count===11?11:undefined);
  assert.equal((view.tree.match(/data-select="layer:/g)||[]).length,count===10?10:0);
 }
});

test('IFCCAD block collections retain the selected block and nested entity context',()=>{
 const blocks=Array.from({length:12},(_,i)=>({path:'/b'+i,attributes:{'ifccad::blockDefinition':{name:'Block '+i}},children:i===11?{'1':'/e1'}:{}}));
 const model=createInspection({presentation:{format:'ifccad',graph:{data:[...blocks,{path:'/e1',attributes:{'ifccad::entity':{},'ifccad::geom::lineSegment':{}}}]}}});
 const view=renderInspection(model,{language:'en',view:'drawing',grouping:'type',selection:'/e1',expanded:new Set(['group:blockDefinition','/b11']),inspector:'properties'});
 assert.equal(view.collection.keys.length,12);assert.match(view.tree,/data-select="\/b11"/);assert.match(view.tree,/data-select="\/e1"/);assert.doesNotMatch(view.tree,/data-select="\/b0"/);
});

test('composed IFCX node JSON uses path, children and attributes while source fragments retain their order',()=>{
 const key='/cad/d1/e1',attributes={'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]},'vendor::note':{value:'retained'}};
 const node={attributes,children:{note:'/reference/note'},inherits:{reference:'/reference/base'},path:key};
 const fragment={path:key,children:node.children,attributes,inherits:node.inherits};
 const source=JSON.stringify({data:[fragment]});
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[node]}}},source);
 const view=renderInspection(model,{language:'en',view:'nodes',selection:key,field:[],expanded:new Set(),inspector:'source'});
 const values=Array.from(view.details.matchAll(/<pre>([\s\S]*?)<\/pre>/g),m=>JSON.parse(m[1].replace(/&quot;/g,'"').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&')));
 assert.deepEqual(Object.keys(values[0]),['path','children','attributes','inherits']);assert.deepEqual(values[0],node);
 assert.deepEqual(Object.keys(values[1]),['path','children','attributes','inherits']);assert.deepEqual(values[1],fragment);
 assert.deepEqual(Object.keys(node),['attributes','children','inherits','path']);assert.equal(model.sourceText,source);
});

test('node JSON formatting keeps absent children absent and preserves foreign fields and uint64 strings',()=>{
 assert.equal(typeof presentation.ifcxNodeJson,'function');
 const node={attributes:{foreign:{id:'18446744073709551615'}},inherits:{base:'/foreign/base'},path:'/foreign/node',extra:'retained'};
 const formatted=presentation.ifcxNodeJson(node),parsed=JSON.parse(formatted);
 assert.deepEqual(Object.keys(parsed),['path','attributes','inherits','extra']);assert.deepEqual(parsed,node);assert.equal(parsed.children,undefined);
});
test('leaf properties live only in the inspector and do not give records structural expand buttons',()=>{
 const key='/cad/d1/e110';const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[{path:key,attributes:{'ifccad::entity':{appearance:{color:{mode:'ByLayer'}},layer:'/cad/d1/layer/0'}}}]}}});
 const view=renderInspection(model,{language:'en',view:'nodes',selection:key,field:[],expanded:new Set([key]),inspector:'properties'});
 assert.doesNotMatch(view.tree,/data-field|data-toggle="\/cad\/d1\/e110"/);
 assert.match(view.details,/data-field-id=/);assert.match(view.details,/<summary><svg[^>]*aria-hidden="true"[\s\S]*?<span class="property-label">Element properties<\/span>/);
 assert.doesNotMatch(view.details,/<summary>[\s\S]*?\d+ fields[\s\S]*?<\/summary>/);
});
test('primitive properties are text and the inspector retains the element heading and nested expansion',()=>{
 const key='/cad/d1/e110',path=['ifccad::entity','appearance','color','mode'];
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[{path:key,attributes:{'ifccad::entity':{appearance:{color:{mode:'ByLayer'}},layer:'/cad/d1/layer/0'},'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]}}}]}}});
 const state={language:'en',view:'nodes',selection:key,field:path,expanded:new Set([key]),fieldExpanded:new Set([JSON.stringify([key,['ifccad::entity']]),JSON.stringify([key,['ifccad::entity','appearance']]),JSON.stringify([key,['ifccad::entity','appearance','color']])]),inspector:'properties'};
 const view=renderInspection(model,state);assert.match(view.details,/<h2>Line · e110<\/h2>/);
 assert.equal((view.details.match(/data-field-id="[^"]*" open/g)||[]).length,4);
 assert.doesNotMatch(view.details,/data-field=|class="property-choice/);assert.match(view.details,/ByLayer/);
 assert.match(view.tree,/class="tree-row selected"/);
});

test('first property layer opens by default while an explicit collapse survives rendering',()=>{
 const key='/cad/d1/e1',model=createInspection({presentation:{format:'ifccad',graph:{data:[{path:key,attributes:{'ifccad::entity':{appearance:{color:{mode:'ByLayer'}}},'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]}}}]}}});
 const state={language:'en',view:'nodes',selection:key,expanded:new Set(),inspector:'properties',fieldExpanded:new Set()};
 const first=renderInspection(model,state);assert.equal((first.details.match(/data-field-id="[^"]*" open/g)||[]).length,2);
 const collapsed=JSON.stringify([key,['ifccad::entity']]);
 const next=renderInspection(model,{...state,fieldCollapsed:new Set([collapsed])});assert.equal((next.details.match(/data-field-id="[^"]*" open/g)||[]).length,1);
 assert.doesNotMatch(next.tree,/data-toggle="\/cad\/d1\/e1"/);
});



test('workspace choices show unspecified and the construction coordinate space',()=>{
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',unit:'m',graph:{data:[{path:'canvas:7',attributes:{'ifccad::paperCanvas':{storedUcs:{kind:'World'},useStoredUcs:false}}},{path:'workspace:9007199254740993',attributes:{'ifccad::viewportWorkspace':{storedUcs:{kind:'World'},useStoredUcs:false}}}]}}});
 const state={language:'en',view:'drawing',field:[],expanded:new Set(),inspector:'properties'};
 const canvas=renderInspection(model,{...state,selection:'canvas:7'});assert.match(canvas.details,/Unspecified/);assert.match(canvas.details,/Coordinate space/);assert.match(canvas.details,/Paper/);
 const viewport=renderInspection(model,{...state,selection:'workspace:9007199254740993'});assert.match(viewport.details,/Model/);assert.match(viewport.details,/Unit: m/);
 assert.equal(model.nodes.get('canvas:7').values.currentUcs,undefined);
});
