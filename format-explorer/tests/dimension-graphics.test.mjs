import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
import * as reports from '../src/inspection-report.mjs';

test('picture transfer report distinguishes all operation states without a validity claim',()=>{
 assert.equal(typeof reports.dimensionGraphicsSummary,'function');
 const assessment={entries:['absent','explicitEmpty','transferred','notTransferred'].map(state=>({graphicsCoverage:{state,partCount:3}}))};
 for(const language of ['en','nl']) {
  const text=reports.dimensionGraphicsSummary(assessment,language);
  assert.match(text,/1/);assert.doesNotMatch(text,/current|valid|geldig|actueel/);
 }
});

test('owned picture parts appear below the dimension, keep numeric order and resource references',()=>{
 const dim='/cad/d7/e9007199254740993',group=dim+'/graphics',style='/cad/d7/textStyle/9007199254740994',layer='/cad/d7/layer/0';
 const parts=Array.from({length:12},(_,i)=>({path:group+'/p'+i,attributes:{'ifccad::dimensionGraphicsPart':i===0?{layer}:{},...(i===1?{'ifccad::text':{style}}:{'ifccad::geom::line':{start:[0,0,0],end:[1,0,0]}})}}));
 const data=[{path:'/cad/d7/layout/1',attributes:{'ifccad::layout':{}},children:{'0':dim}},{path:dim,attributes:{'ifccad::entity':{},'ifccad::dimension':{}},children:{graphics:group}},{path:group,attributes:{'ifccad::dimensionGraphics':{origin:'sourceSnapshot'}},children:Object.fromEntries(parts.map((p,i)=>[String(i),p.path]).reverse())},...parts,{path:style,attributes:{'ifccad::textStyle':{name:'Face'}}},{path:layer,attributes:{'ifccad::layer':{name:'0'}}}];
 const model=createInspection({validation:{strictAvailable:true},presentation:{format:'ifccad',graph:{data}}});
 assert.deepEqual(model.nodes.get(dim).children,[group]);
 assert.deepEqual(model.nodes.get(group).children,parts.map(p=>p.path));
 assert.equal(model.nodes.get(group+'/p0').drawPosition,undefined);
 assert.equal(model.nodes.get(group+'/p0').owner,dim);
 assert.ok(model.nodes.get(group+'/p0').outgoing.some(e=>e.target===layer));
 assert.ok(model.nodes.get(group+'/p1').outgoing.some(e=>e.target===style));
 assert.ok(!model.nodes.get('group:other'));
 for(const language of ['nl','en']){
  const html=renderInspection(model,{language,selection:group,expanded:new Set([dim,group]),inspector:'properties'});
  assert.doesNotMatch(html.details.replace(/<[^>]*>/g,''),/dimensionGraphics|plural\.dimension|sourceSnapshot/);
 }
});
