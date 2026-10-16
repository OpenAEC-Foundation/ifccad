import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
import {renderDimensionLabel} from '../src/dimension-view.mjs';

function fixture(format) {
 const wide='9007199254740993',style='/cad/d1/dimensionStyle/4',key=format==='ocdraw'?'entity:1':'/cad/d1/e1';
 const geometry={type:'dimension',variant:'Aligned',definition:{kind:'aligned',firstPoint:[0,0],secondPoint:[48.25,0]},labelDefinition:{kind:'generated'},textPlacement:{kind:'styleDriven'},rawMeasurement:{kind:'distance',value:48.25},label:{state:'visible',runs:[{text:'4\'-0 1/4" <script>&'}],primary:'4\'-0 1/4"',fragments:[{kind:'run',run:{text:'4\'-0 <script>& '}},{kind:'fraction',numerator:'1',denominator:'4',layout:'diagonal',textScale:0.75},{kind:'run',run:{text:'"'}}]},effectiveSettings:{text:{height:2.5},numbers:{primary:{fractionLayout:'diagonal',fractionScale:0.75}}},associationUpdate:'external',displayEvaluation:'unassessed'};
 const p=format==='ocdraw'?{format,dimensionStyles:[{id:'4',name:'Building',settings:{text:{style:'2'}}}],textStyles:[{id:'2',name:'Face'}],entities:[{id:'1',styleId:'4',geometry:{...geometry,associations:[{role:'secondPoint',sourceEntityId:wide,selector:{kind:'end'}}]}},{id:wide,geometry:{type:'line'}}],layouts:[{id:0,scopeId:0,name:'Model'}],scopes:[{id:0,entities:['1',wide]}]}:{format,graph:{data:[{path:key,attributes:{'ifccad::entity':{},'ifccad::dimension':{style,definition:geometry.definition,label:geometry.labelDefinition,textPlacement:geometry.textPlacement,associations:[{role:'secondPoint',source:'/cad/d1/e'+wide,selector:{kind:'end'}}]}}},{path:style,attributes:{'ifccad::dimensionStyle':{name:'Building',settings:{text:{style:'/cad/d1/textStyle/2'}}}}},{path:'/cad/d1/e'+wide,attributes:{'ifccad::entity':{},'ifccad::geom::line':{}}}]},dimensions:[{path:key,geometry}]};
 return {model:createInspection({validation:{strictAvailable:true},presentation:p}),key,wide};
}
function panel(format,language,inspector='properties') {
 const f=fixture(format);
 return {...f,...renderInspection(f.model,{language,view:'drawing',grouping:'type',selection:f.key,expanded:new Set(),inspector})};
}
test('Dimension properties use localized labels and separate stored and calculated data in both formats',()=>{
 for(const format of ['ocdraw','ifccad'])for(const language of ['nl','en']){
  const {details,model,key}=panel(format,language);
  assert.match(details,language==='nl'?/Bemating/:/Dimension/);
  assert.match(details,language==='nl'?/Berekende maat/:/Calculated measurement/);
  assert.match(details,language==='nl'?/Effectieve instellingen/:/Effective settings/);
  assert.match(details,language==='nl'?/Teksthoogte/:/Text height/);
  assert.doesNotMatch(details,/plural\.dimension|dimensionInspection|>textPlacement</);
  assert.equal(model.nodes.get(key).values.dimensionInspection,undefined);
 }
});
test('stacked label preview is escaped and keeps full-width source references',()=>{
 for(const format of ['ocdraw','ifccad']){
  const {details,model,key,wide}=panel(format,'nl');
  assert.match(details,/dimension-fraction diagonal/);
  assert.match(details,/&lt;script&gt;&amp;/);
  assert.doesNotMatch(details,/<script>/);
  const target=format==='ocdraw'?'entity:'+wide:'/cad/d1/e'+wide;
  assert.ok(model.nodes.get(key).outgoing.some(e=>e.target===target));
  const source=panel(format,'en','source').details;
  assert.match(source,/definition/);
  assert.doesNotMatch(source,/dimension-fraction/);
 }
});

test('horizontal stacks keep the relative scale, accessible text and literal decorations',()=>{
 const html=renderDimensionLabel({state:'visible',runs:[{text:'note 1/2'}],fragments:[{kind:'run',run:{text:'note ',underline:true,overline:true,strikeThrough:true}},{kind:'fraction',layout:'horizontal',textScale:0.8,numerator:'1',denominator:'2'}]},'en');
 assert.match(html,/dimension-fraction horizontal/);
 assert.match(html,/font-size:0.8em/);
 assert.match(html,/text-decoration:underline overline line-through/);
 assert.match(html,/aria-label="note 1\/2"/);
});
