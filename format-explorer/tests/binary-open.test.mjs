import test from 'node:test';
import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';

test('binary opening retains the decoded representation and JSON export uses the processor',()=>{
 for(const name of ['drawing.ifccad.cbor','drawing.ifccad.cbor.zst','drawing.CBOR','drawing.zst']){
  const bytes=new Uint8Array([0xa3,0,0,0]).buffer,calls=[];
  const wasm={open_ifccad:()=>JSON.stringify({validation:{strictAvailable:true},source:{encoding:'cbor',representation:'decoded'},nativeSourceText:'{"decoded":true}'}),export_ifccad(...args){calls.push(args);return '{"validation":{"strictAvailable":true},"export":{"download":{"base64":"e30=","fileName":"drawing.ifcx"}}}';}};
  const request={kind:'ifccad',name,files:[{path:name,bytes}]};
  assert.equal(processBrowserRequest(request,wasm).nativeSourceText,'{"decoded":true}');
  const result=processBrowserRequest({...request,export:{format:'ifccad'}},wasm);
  assert.equal(calls.length,1);assert.deepEqual([...calls[0][1]],[0xa3,0,0,0]);
  assert.equal(result.export.download.base64,'e30=');assert.equal(result.nativeSourceText,'{"decoded":true}');
 }
});
