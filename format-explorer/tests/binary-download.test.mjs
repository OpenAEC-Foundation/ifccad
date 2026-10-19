import test from 'node:test';
import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';

test('binary downloads encode the checked IFCCAD source for both native and CAD inputs',()=>{
 for(const kind of ['ifccad','cad'])for(const encoding of ['cbor','cbor-zstd']){
  const bytes=new TextEncoder().encode('original').buffer,received=[];
  const wasm={open_ifccad:()=>'{"validation":{"strictAvailable":true}}',convert_cad_to_ifccad:()=>'{"validation":{"strictAvailable":true},"conversion":{"diagnostics":["retained"]},"export":{"download":{"base64":"e30="}}}',export_ifccad_binary(name,input,choice){received.push([name,new TextDecoder().decode(input),choice]);return '{"validation":{"strictAvailable":true},"export":{"download":{"base64":"AQI="}}}';}};
  const name=kind==='cad'?'drawing.dxf':'drawing.ifcx';
  const result=processBrowserRequest({kind,name,files:[{path:name,bytes}],export:{format:'ifccad',encoding}},wasm);
  assert.deepEqual(received,[[name,kind==='cad'?'{}':'original',encoding]]);
  assert.equal(result.export.download.base64,'AQI=');
 }
});

test('binary selection rejects unknown encodings and never exports invalid drawings',()=>{
 const request={kind:'ifccad',name:'a.ifcx',files:[{path:'a.ifcx',bytes:new ArrayBuffer(1)}]};
 const wasm={open_ifccad:()=>'{"validation":{"strictAvailable":false}}',export_ifccad_binary(){throw Error('must not export');}};
 assert.equal(processBrowserRequest({...request,export:{format:'ifccad',encoding:'cbor'}},wasm).export,undefined);
 assert.throws(()=>processBrowserRequest({...request,export:{format:'ifccad',encoding:'zip'}},wasm),/encoding/i);
 assert.throws(()=>processBrowserRequest({...request,export:{format:'dxf',encoding:'cbor'}},wasm),/encoding/i);
});
