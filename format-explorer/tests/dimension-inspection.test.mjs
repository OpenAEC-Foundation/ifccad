import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from './support/inspection-fixture.mjs';

test('IFCCAD dimensions expose typed inspection and complete source paths',()=>{
 const dim='/cad/d7/e1',source='/cad/d7/e9007199254740993',style='/cad/d7/dimensionStyle/1';
 const graph={data:[{path:dim,attributes:{'ifccad::entity':{},'ifccad::dimension':{style,associations:[{role:'secondPoint',source,selector:{kind:'end'}}]}}},{path:source,attributes:{'ifccad::entity':{},'ifccad::geom::line':{end:[4500,0,0]}}},{path:style,attributes:{'ifccad::dimensionStyle':{name:'Building',settings:{text:{style:'/cad/d7/textStyle/1'}}}}}]};
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph,dimensions:[{path:dim,geometry:{rawMeasurement:{kind:'distance',value:4000},displayEvaluation:'unassessed'}}]}});
 const d=model.nodes.get(dim);assert.equal(d.type,'dimension');assert.equal(d.derived.rawMeasurement.value,4000);assert.ok(d.outgoing.some(e=>e.target===source));assert.ok(d.outgoing.some(e=>e.target===style));assert.ok(model.roots.includes('group:dimensionStyle'));
});

test('dimension component pattern selections link explicit mode values in both formats',()=>{
 const pattern='/cad/d1/linePattern/5',style='/cad/d1/dimensionStyle/1',entity='/cad/d1/e2';
 const value={dimensionLine:{appearance:{linePattern:{mode:'explicit',value:pattern}}}};
 const ifccad=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data:[{path:pattern,attributes:{'ifccad::linePattern':{}}},{path:style,attributes:{'ifccad::dimensionStyle':{settings:value}}},{path:entity,attributes:{'ifccad::entity':{},'ifccad::dimension':{style,overrides:value}}}]}}});
 assert.ok(ifccad.nodes.get(style).outgoing.some(e=>e.target===pattern && e.field.endsWith('.appearance.linePattern.value')));
 assert.ok(ifccad.nodes.get(entity).outgoing.some(e=>e.target===pattern && e.field.endsWith('.appearance.linePattern.value')));
});
