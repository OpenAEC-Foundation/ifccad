import test from 'node:test';import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';
const bytes=new TextEncoder().encode('demo').buffer;
test('spline preservation is an explicit OCDraw import option and survives CAD export summaries',()=>{
 let selected;const wasm={convert_cad_to_drawing(){selected=false;return '{"validation":{"strictAvailable":true},"presentation":{"opaqueEntityCount":0}}';},
  convert_cad_to_drawing_with_preservation(name,format,input,capture){selected=capture;return '{"conversion":{"preservation":{"entries":[{"result":"capturedTyped"}]}},"validation":{"strictAvailable":true},"presentation":{"opaqueEntityCount":1},"export":{"download":{"base64":"e30="}}}';},
  export_drawing(){return '{"conversion":{"preservation":{"entries":[{"result":"restoredTyped"}]}},"validation":{"strictAvailable":true}}';}};
 const request={kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}]};
 assert.equal(processBrowserRequest(request,wasm).presentation.opaqueEntityCount,0);assert.equal(selected,false);
 const result=processBrowserRequest({...request,preserveSplines:true,export:{format:'dxf'}},wasm);
 assert.equal(selected,true);assert.equal(result.conversion.preservation.entries[0].result,'capturedTyped');
 assert.equal(result.conversion.restoration.preservation.entries[0].result,'restoredTyped');
});
test('standalone opening uses one local file, without a package entry',()=>{let received;const wasm={open_drawing(...args){received=args;return '{"validation":{"strictAvailable":true},"presentation":{"entityId":9007199254740993}}';}};const result=processBrowserRequest({kind:'drawing',name:'demo',files:[{path:'demo.ocdraw.json',bytes}]},wasm);assert.equal(received[0],'demo');assert.equal(result.presentation.entityId,'9007199254740993');});
test('CAD conversion supplies checked standalone bytes to CAD export',()=>{let received;const wasm={convert_cad_to_drawing(){return JSON.stringify({source:{format:'dxf'},conversion:{diagnostics:['loss']},validation:{strictAvailable:true},export:{download:{base64:'e30='}}});},export_drawing(...args){received=args;return '{"validation":{"strictAvailable":true},"export":{"format":"dwg"}}';}};const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],export:{format:'dwg',version:'AC1032'}},wasm);assert.equal(new TextDecoder().decode(received[1]),'{}');assert.equal(received[2],'dwg');assert.deepEqual(result.conversion.diagnostics,['loss']);});
test('failed validation never supplies downloadable bytes',()=>{const wasm={open_drawing(){return '{"validation":{"strictAvailable":false}}';},export_drawing(){throw Error('must not export');}};const result=processBrowserRequest({kind:'drawing',files:[{path:'a.ocdraw.json',bytes}],export:{format:'dxf'}},wasm);assert.equal(result.export,undefined);});
test('retired package selections and unsafe paths are rejected',()=>{for(const request of [{kind:'package',files:[{path:'package.ifcx.json',bytes}]},{kind:'drawing',files:[{path:'../a.ocdraw.json',bytes}]},{kind:'drawing',files:[{path:'a.ocdraw.json',bytes},{path:'b.ocdraw.json',bytes}]}])assert.throws(()=>processBrowserRequest(request,{}));});
test('IFCCAD opening and CAD export use the direct IFCCAD adapter',()=>{
 let exported;
 const wasm={open_ifccad(){return JSON.stringify({validation:{strictAvailable:true},presentation:{format:'ifccad'}});},export_ifccad(...args){exported=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},open_drawing(){throw Error('OCDraw route must not run');}};
 const request={kind:'ifccad',name:'hello.ifcx',files:[{path:'hello.ifcx',bytes}]};
 assert.equal(processBrowserRequest(request,wasm).presentation.format,'ifccad');
 processBrowserRequest({...request,export:{format:'dxf',version:'AC1032'}},wasm);
 assert.equal(exported[2],'dxf');assert.deepEqual([...exported[1]],[...new Uint8Array(bytes)]);
});
test('CAD can roundtrip via IFCCAD without an OCDraw intermediate',()=>{
 let received;
 const wasm={convert_cad_to_ifccad(){return JSON.stringify({source:{format:'dwg'},conversion:{diagnostics:['ifcx-loss']},validation:{strictAvailable:true},export:{format:'ifccad',download:{base64:'e30='}}});},export_ifccad(...args){received=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},convert_cad_to_drawing(){throw Error('wrong converter');}};
 const request={kind:'cad',drawingFormat:'ifccad',name:'a.dwg',files:[{path:'a.dwg',bytes}]};
 assert.equal(processBrowserRequest({...request,export:{format:'ifccad'}},wasm).export.format,'ifccad');
 const result=processBrowserRequest({...request,export:{format:'dxf'}},wasm);
 assert.equal(new TextDecoder().decode(received[1]),'{}');assert.deepEqual(result.conversion.diagnostics,['ifcx-loss']);
});
