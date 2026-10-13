import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
test('OCDraw dimensions link their own style and a forward full-width source',()=>{
 const wide='9007199254740993';
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ocdraw',dimensionStyles:[{id:'4',name:'Building',settings:{text:{style:'2'}}}],textStyles:[{id:'2',name:'Face'}],entities:[{id:'1',styleId:'4',geometry:{type:'dimension',rawMeasurement:{kind:'distance',value:4000},overrides:{text:{height:2.5}},displayEvaluation:'unassessed',associations:[{role:'secondPoint',sourceEntityId:wide,selector:{kind:'end'}}]}},{id:wide,geometry:{type:'line',end:[4500,0,0]}}],layouts:[{id:0,scopeId:0}],scopes:[{id:0,entities:['1',wide]}]}});
 const d=model.nodes.get('entity:1');assert.equal(d.type,'dimension');assert.equal(d.values.geometry.rawMeasurement.value,4000);assert.equal(d.values.geometry.overrides.text.height,2.5);assert.equal(d.values.geometry.displayEvaluation,'unassessed');
 assert.ok(d.outgoing.some(e=>e.target==='dimensionStyle:4'));assert.ok(!d.outgoing.some(e=>e.target==='textStyle:4'));assert.ok(d.outgoing.some(e=>e.target==='entity:'+wide));assert.ok(model.nodes.get('entity:'+wide).incoming.some(e=>e.source==='entity:1'));
 assert.ok(model.nodes.get('dimensionStyle:4').outgoing.some(e=>e.target==='textStyle:2'));
});
test('IFCCAD dimensions expose typed inspection and complete source paths',()=>{
 const dim='/cad/d7/e1',source='/cad/d7/e9007199254740993',style='/cad/d7/dimensionStyle/1';
 const graph={data:[{path:dim,attributes:{'ifccad::entity':{},'ifccad::dimension':{style,associations:[{role:'secondPoint',source,selector:{kind:'end'}}]}}},{path:source,attributes:{'ifccad::entity':{},'ifccad::geom::line':{end:[4500,0,0]}}},{path:style,attributes:{'ifccad::dimensionStyle':{name:'Building',settings:{text:{style:'/cad/d7/textStyle/1'}}}}}]};
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph,dimensions:[{path:dim,geometry:{rawMeasurement:{kind:'distance',value:4000},displayEvaluation:'unassessed'}}]}});
 const d=model.nodes.get(dim);assert.equal(d.type,'dimension');assert.equal(d.values.dimensionInspection.geometry.rawMeasurement.value,4000);assert.ok(d.outgoing.some(e=>e.target===source));assert.ok(d.outgoing.some(e=>e.target===style));assert.ok(model.roots.includes('group:dimensionStyle'));
});
