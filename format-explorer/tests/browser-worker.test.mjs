import test from 'node:test';import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';
const bytes=new TextEncoder().encode('demo').buffer;
test('standalone opening uses one local file, without a package entry',()=>{let received;const wasm={open_drawing(...args){received=args;return '{"validation":{"strictAvailable":true},"presentation":{"entityId":9007199254740993}}';}};const result=processBrowserRequest({kind:'drawing',name:'demo',files:[{path:'demo.ocdraw.json',bytes}]},wasm);assert.equal(received[0],'demo');assert.equal(result.presentation.entityId,'9007199254740993');});
test('CAD conversion supplies checked standalone bytes to CAD export',()=>{let received;const wasm={convert_cad_to_drawing(){return JSON.stringify({source:{format:'dxf'},conversion:{diagnostics:['loss']},validation:{strictAvailable:true},export:{download:{base64:'e30='}}});},export_drawing(...args){received=args;return '{"validation":{"strictAvailable":true},"export":{"format":"dwg"}}';}};const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],export:{format:'dwg',version:'AC1032'}},wasm);assert.equal(new TextDecoder().decode(received[1]),'{}');assert.equal(received[2],'dwg');assert.deepEqual(result.conversion.diagnostics,['loss']);});
test('failed validation never supplies downloadable bytes',()=>{const wasm={open_drawing(){return '{"validation":{"strictAvailable":false}}';},export_drawing(){throw Error('must not export');}};const result=processBrowserRequest({kind:'drawing',files:[{path:'a.ocdraw.json',bytes}],export:{format:'dxf'}},wasm);assert.equal(result.export,undefined);});
test('retired package selections and unsafe paths are rejected',()=>{for(const request of [{kind:'package',files:[{path:'package.ifcx.json',bytes}]},{kind:'drawing',files:[{path:'../a.ocdraw.json',bytes}]},{kind:'drawing',files:[{path:'a.ocdraw.json',bytes},{path:'b.ocdraw.json',bytes}]}])assert.throws(()=>processBrowserRequest(request,{}));});
test('IFCX opening and CAD export use the direct IFCX adapter',()=>{
 let exported;
 const wasm={open_ifcx(){return JSON.stringify({validation:{strictAvailable:true},presentation:{format:'ifcx'}});},export_ifcx(...args){exported=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},open_drawing(){throw Error('OCDraw route must not run');}};
 const request={kind:'ifcx',name:'hello.ifcx',files:[{path:'hello.ifcx',bytes}]};
 assert.equal(processBrowserRequest(request,wasm).presentation.format,'ifcx');
 processBrowserRequest({...request,export:{format:'dxf',version:'AC1032'}},wasm);
 assert.equal(exported[2],'dxf');assert.deepEqual([...exported[1]],[...new Uint8Array(bytes)]);
});
test('CAD can roundtrip via IFCX without an OCDraw intermediate',()=>{
 let received;
 const wasm={convert_cad_to_ifcx(){return JSON.stringify({source:{format:'dwg'},conversion:{diagnostics:['ifcx-loss']},validation:{strictAvailable:true},export:{format:'ifcx',download:{base64:'e30='}}});},export_ifcx(...args){received=args;return JSON.stringify({validation:{strictAvailable:true},export:{format:'dxf'}});},convert_cad_to_drawing(){throw Error('wrong converter');}};
 const request={kind:'cad',drawingFormat:'ifcx',name:'a.dwg',files:[{path:'a.dwg',bytes}]};
 assert.equal(processBrowserRequest({...request,export:{format:'ifcx'}},wasm).export.format,'ifcx');
 const result=processBrowserRequest({...request,export:{format:'dxf'}},wasm);
 assert.equal(new TextDecoder().decode(received[1]),'{}');assert.deepEqual(result.conversion.diagnostics,['ifcx-loss']);
});
