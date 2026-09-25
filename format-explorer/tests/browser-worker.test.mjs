import test from 'node:test';
import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';

const bytes=new TextEncoder().encode('demo').buffer;
test('CAD opening calls the WASM reader with local bytes',()=>{
 let passed;
 const wasm={open_cad(...args){passed=args;return JSON.stringify({validation:{strictAvailable:true}});}};
 const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}]},wasm);
 assert.equal(result.validation.strictAvailable,true);
 assert.equal(passed[1],'dxf');assert.deepEqual([...passed[2]],[...new Uint8Array(bytes)]);
});

test('CAD export uses the converted package and retains import diagnostics',()=>{
 let packagePaths;
 const wasm={
  open_cad(){return JSON.stringify({source:{name:'a.dxf'},reader:{status:'ok'},conversion:{assessment:{conclusion:'LossDetected'}},validation:{strictAvailable:true},presentation:{documents:[{path:'package.ifcx.json',text:'{}'}]}});},
  export_package(name,paths){packagePaths=paths;return JSON.stringify({validation:{strictAvailable:true},export:{fileCheck:{readable:true}}});}
 };
 const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes}],export:{format:'dxf',drawing:'drawing-0',version:'AC1032'}},wasm);
 assert.deepEqual(packagePaths,['package.ifcx.json']);
 assert.equal(result.conversion.assessment.conclusion,'LossDetected');
 assert.equal(result.export.fileCheck.readable,true);
});
