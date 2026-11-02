import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
test('extrusion properties distinguish signed plane sweep from widths and display qualification',()=>{
 const path='/cad/d1/e1',extrusion={path,thickness:-3,direction:[0,-.6,.8],unit:'mm',displayEvaluation:'unassessed'};
 const model=createInspection({validation:{strictAvailable:true},presentation:{extrusions:[extrusion],graph:{data:[{path,attributes:{'ifccad::entity':{},'ifccad::geom::circle':{radius:5,thickness:-3}}}]}}});
 assert.deepEqual(model.nodes.get(path).extrusion,extrusion);
 for(const language of ['en','nl']){
  const {details}=renderInspection(model,{language,selection:path,inspector:'properties',expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set()});
  assert.match(details,/-3 mm/);assert.match(details,/0, -0.6, 0.8/);
  assert.match(details,language==='nl'?/Extrusiedikte/:/Extrusion thickness/);
  assert.match(details,language==='nl'?/weergave.*niet beoordeeld/i:/display.*unassessed/i);
 }
});
