import test from 'node:test';
import assert from 'node:assert/strict';
import {createFontSymbolReport,applyFontSymbolResult} from '../src/font-symbol-sources.mjs';

const request={requestPath:'/cad/d1/symbolSource/9007199254740993#/attributes/ifccad::symbolSource/cadFileName',fontRole:'shx',requestedName:'../fonts/<missing>.shx'};
test('late font resolution cannot attach to a different snapshot',()=>{
 const snapshot={},other={},report=createFontSymbolReport([request],snapshot);
 assert.equal(report.entries[0].status,'notChecked');
 assert.throws(()=>applyFontSymbolResult(report,other,{...request,status:'found'}),/snapshot/i);
 assert.equal(report.entries[0].status,'notChecked');
});
test('substitution is separate from the immutable requested source',()=>{
 const snapshot={},report=createFontSymbolReport([request],snapshot);
 const result=applyFontSymbolResult(report,snapshot,{requestPath:request.requestPath,fontRole:'shx',status:'substituted',usedName:'fallback.shx'});
 assert.equal(result.entries[0].requestedName,request.requestedName);
 assert.equal(result.entries[0].usedName,'fallback.shx');
 assert.equal(report.entries[0].status,'notChecked');
 assert.ok(Object.isFrozen(result.entries[0]));
 for(const status of ['found','missing','unreadable','notChecked'])assert.equal(applyFontSymbolResult(report,snapshot,{requestPath:request.requestPath,fontRole:'shx',status}).entries[0].status,status);
 assert.throws(()=>applyFontSymbolResult(report,snapshot,{requestPath:request.requestPath,fontRole:'shx',status:'substituted'}));
 assert.throws(()=>applyFontSymbolResult(report,snapshot,{requestPath:request.requestPath,fontRole:'family',status:'found'}));
 assert.throws(()=>applyFontSymbolResult(report,snapshot,{requestPath:'unknown',fontRole:'shx',status:'found'}));
 assert.throws(()=>applyFontSymbolResult(report,snapshot,{requestPath:request.requestPath,fontRole:'shx',status:'notChecked',usedName:'stale.shx'}));
});


import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
test('inspector shows authored requests and unchecked status on its own snapshot',()=>{
 const path='/cad/d1/symbolSource/9007199254740993';
 const result={presentation:{graph:{data:[{path,attributes:{'ifccad::symbolSource':{kind:'shx',fileName:'shape.shx'}}}]},fontSymbolRequests:[{requestPath:path+'#/attributes/ifccad::symbolSource/cadFileName',fontRole:'shx',requestedName:'<source>&.shx'}]}};
 const model=createInspection(result);
 assert.equal(model.fontSymbolReport.snapshot,result);
 const view=renderInspection(model,{language:'nl',view:'drawing',selection:path,expanded:new Set(),inspector:'properties'});
 assert.match(view.details,/Niet gecontroleerd/);
 assert.match(view.details,/&lt;source&gt;&amp;\.shx/);
 assert.ok(!view.details.includes('<source>'));
 const next={...result};
 const nextModel=createInspection(next);
 assert.notEqual(nextModel.fontSymbolReport.snapshot,model.fontSymbolReport.snapshot);
 assert.equal(result.presentation.fontSymbolRequests[0].status,undefined);
});
