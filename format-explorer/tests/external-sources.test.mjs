import test from 'node:test';
import assert from 'node:assert/strict';
import {createInspection} from '../src/inspection-model.mjs';
import {createDocumentSession} from '../src/ocs-messages.mjs';
import {createCadPreviewController} from '../src/cad-preview.mjs';

test('XREF source navigation keeps full-width identities and authored URI separate',()=>{
 const id='9007199254740993',source='/cad/d1/externalSource/'+id,entity='/cad/d1/e4';
 const graph={data:[{path:source,attributes:{'ifccad::externalSource':{name:'Ground',location:{kind:'uriReference',uri:'../Refs/ground.dwg'},source:{format:'dwg',basePoint:[10,20,0]}}}},{path:entity,attributes:{'ifccad::entity':{},'ifccad::xref':{source,transform:{placement:{origin:[0,0,0],xAxis:[1,0,0],yAxis:[0,1,0]},scale:[500,500,500]}} }}]};
 const model=createInspection({presentation:{graph,externalSources:[{sourcePath:source,id,status:'notChecked'}]}});
 assert.equal(model.nodes.get(entity).type,'xref');
 assert.equal(model.nodes.get(entity).outgoing.some(e=>e.target===source&&e.kind==='externalSource'),true);
 assert.deepEqual(model.nodes.get('group:externalSource').children,[source]);
 assert.equal(model.nodes.get(source).externalSource.id,id);
 assert.equal(graph.data[0].attributes['ifccad::externalSource'].location.uri,'../Refs/ground.dwg');
});

test('choosing source bytes leaves native main bytes and location untouched',async()=>{
 const calls=[],native={format:'ifccad',fileName:'main.ifcx',base64:'e30=',byteLength:2};
 const controller=createCadPreviewController({openSession:async()=>({replaceGenerated:async(...a)=>calls.push(['main',...a]),registerExternalSource:async(...a)=>{calls.push(['source',...a]);return {sourceStatus:'Loaded'};}})});
 const opening={validation:{strictAvailable:true},nativeFile:native,presentation:{externalSources:[{id:'1',location:{kind:'uriReference',uri:'../Refs/original.dwg'}}]}};
 controller.setSource({kind:'ifccad',name:'main.ifcx'},true,opening);
 await controller.loadExternalSource('1',{name:'chosen.dwg',arrayBuffer:async()=>new Uint8Array([1,2]).buffer});
 assert.deepEqual(calls,[['main','e30=','main.ifcx'],['source','1','chosen.dwg','AQI=']]);
 assert.equal(opening.nativeFile.base64,'e30=');
 assert.equal(opening.presentation.externalSources[0].location.uri,'../Refs/original.dwg');
});

test('DXF and native source choices reach the existing drawing-scoped registration',async()=>{
 for(const [format,name] of [['dxf','chosen.dxf'],['ifccad','chosen.ifccad'],['ifccad','chosen.ifccad.cbor'],['dxf','extensionless']]){
  const calls=[],native={format:'ifccad',fileName:'main.ifccad',base64:'e30=',byteLength:2};
  const controller=createCadPreviewController({openSession:async()=>({replaceGenerated:async()=>{},registerExternalSource:async(...args)=>{calls.push(args);return {sourceStatus:'Loaded'};}})});
  const opening={validation:{strictAvailable:true},nativeFile:native,presentation:{externalSources:[{id:'1',source:{format},location:{kind:'uriReference',uri:'../Refs/original'}}]}};
  controller.setSource({kind:'ifccad',name:'main.ifccad'},true,opening);
  await controller.loadExternalSource('1',{name,arrayBuffer:async()=>new Uint8Array([1,2]).buffer});
  assert.deepEqual(calls,[['1',name,'AQI=']],format);
  assert.equal(opening.nativeFile.base64,'e30=');
  assert.equal(opening.presentation.externalSources[0].location.uri,'../Refs/original');
 }
});

test('source bytes bind to the selected generated document and stale requests are refused',async()=>{
 let current=1,next=2;const docs=new Map([[1,{id:1,dirty:false}]]),loads=[];
 const control=async request=>{
  if(request.op==='state')return {ok:true,document_id:current,documents:[...docs.values()]};
  if(request.op==='open'){current=next++;docs.set(current,{id:current,dirty:false});return {ok:true};}
  if(request.op==='activate'){current=request.document_id;return {ok:true};}
  if(request.op==='action'){docs.delete(current);return {ok:true};}
  if(request.op==='xref_source_register'){loads.push(request);return {ok:true,result:{source_status:'loaded',other_drawing:true}};}
  throw Error(request.op);
 };
 const session=createDocumentSession(control);
 await session.replaceGenerated('e30=','main.ifcx');const documentId=session.ids().generatedId;
 const loaded=await session.registerExternalSource('9007199254740993','ground.dwg','e30=',documentId);
 assert.equal(loaded.source_status,'loaded');
 assert.equal(loaded.other_drawing,true);
 assert.equal(loads[0].document_id,documentId);
 assert.equal(loads[0].source_id,'9007199254740993');
 await session.replaceGenerated('e30=','other.ifcx');
 await assert.rejects(()=>session.registerExternalSource('9007199254740993','ground.dwg','e30=',documentId),/changed/i);
 assert.equal(loads.length,1);
});


test('XREF inspection exposes all owner scopes without inventing visible occurrences',()=>{
 const source='/cad/d1/externalSource/1',transform={placement:{origin:[10,20,30],xAxis:[-1,0,0],yAxis:[0,0,1]},rotation:.5,scale:[-2,3,4]};
 const xrefs=['model','paper','block'].map((ownerKind,i)=>({entityPath:'/cad/d1/e'+(i+1),ownerPath:ownerKind==='block'?'/cad/d1/block/1':'/cad/d1/layout/'+i,ownerKind,sourceId:'1',sourcePath:source,transform,referenceType:'attach'}));
 const graph={data:[{path:source,attributes:{'ifccad::externalSource':{name:'Shared',location:{kind:'uriReference',uri:'detail.dwg'},source:{format:'dwg',basePoint:[1,2,3]}}}},...xrefs.map(x=>({path:x.entityPath,attributes:{'ifccad::entity':{},'ifccad::xref':{source,transform}}}))]};
 const model=createInspection({presentation:{graph,xrefs,externalSources:[{id:'1',sourcePath:source,status:'notChecked'}]}});
 for(const x of xrefs){const n=model.nodes.get(x.entityPath);assert.equal(n.derived.ownerKind,x.ownerKind);assert.deepEqual(n.derived.transform,transform);assert.ok(n.outgoing.some(e=>e.target===source));}
 assert.equal(model.nodes.get(source).externalSource.status,'notChecked');
 assert.equal(model.nodes.get(source).incoming.length,3);
});
