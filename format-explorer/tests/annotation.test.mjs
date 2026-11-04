import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
test('annotation resources, explicit selection links and dormant state are inspectable in both languages',()=>{
 const scale='/cad/d1/annotationScale/9007199254740993',entity='/cad/d1/e1';
 const model=createInspection({presentation:{graph:{data:[{path:scale,attributes:{'ifccad::annotationScale':{name:'1:2',paperUnits:1,drawingUnits:2}}},{path:entity,attributes:{'ifccad::text':{},'ifccad::annotation':{mode:'disabled',reference:{scaleId:scale},contexts:[{key:{scaleId:scale},presentation:null}]}}}]}}});
 assert.deepEqual(model.nodes.get('group:annotationScale').children,[scale]);
 assert.ok(model.nodes.get(entity).outgoing.some(e=>e.target===scale));
 for(const language of ['nl','en']){
  const {details}=renderInspection(model,{selection:entity,language,inspector:'properties',expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set()});
  assert.match(details,language==='nl'?/Annotatieve contexten/:/Annotation contexts/);
  assert.match(details,language==='nl'?/Uitgeschakeld/:/Disabled/);
  assert.match(details,/1:2/);
 }
});
