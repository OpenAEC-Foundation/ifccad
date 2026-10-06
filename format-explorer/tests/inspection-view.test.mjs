import test from 'node:test';import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
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
