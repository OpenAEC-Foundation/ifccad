import test from 'node:test';import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
import * as presentation from '../src/presentation-json.mjs';

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
test('compound inspector values keep their title and count together above full-width JSON',()=>{
 const key='/cad/d1/e110';const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[{path:key,attributes:{'ifccad::entity':{appearance:{color:{mode:'ByLayer'}},layer:'/cad/d1/layer/0'}}}]}}});
 const view=renderInspection(model,{language:'en',view:'nodes',selection:key,field:[],expanded:new Set(),inspector:'properties'});
 assert.match(view.details,/class="property-composite"/);assert.match(view.details,/<summary><span[^>]*>Element properties<\/span><span class="value-summary">/);assert.match(view.details,/<svg[^>]*aria-hidden="true"/);
});
test('a selected property remains distinct from its owner and retains nested expansion',()=>{
 const key='/cad/d1/e110',path=['ifccad::entity','appearance','color'];
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[{path:key,attributes:{'ifccad::entity':{appearance:{color:{mode:'ByLayer'}},layer:'/cad/d1/layer/0'},'ifccad::geom::lineSegment':{start:[0,0,0],end:[1,0,0]}}}]}}});
 const state={language:'en',view:'nodes',selection:key,field:path,expanded:new Set([key]),fieldExpanded:new Set([JSON.stringify([key,['ifccad::entity']]),JSON.stringify([key,['ifccad::entity','appearance']])]),inspector:'properties'};
 const view=renderInspection(model,state);assert.match(view.details,/<h2>Color<\/h2>/);
 assert.equal((view.tree.match(/data-field-id="[^"]*" open/g)||[]).length,2);
 assert.match(view.tree,/class="property-choice selected"[^>]*aria-current="true"/);
 assert.doesNotMatch(view.tree,/class="tree-row selected"/);
});
