import test from 'node:test';
import assert from 'node:assert/strict';
import {processBrowserRequest} from '../src/browser-worker.mjs';

const bytes=new TextEncoder().encode('{}').buffer;
const file=name=>({path:name,bytes});

test('CAD import defaults to IFCCAD without advertising or invoking OCDraw',()=>{
 let invoked=false;
 const wasm={convert_cad_to_ifccad(){invoked=true;return '{"validation":{"strictAvailable":true},"source":{"format":"ifccad"}}';}};
 const result=processBrowserRequest({kind:'cad',name:'a.dxf',files:[file('a.dxf')]},wasm);
 assert.equal(invoked,true);
 assert.equal(result.source.format,'ifccad');
});

for(const request of [
 {kind:'drawing',name:'old.ocdraw.json'},
 {kind:'cad',name:'a.dxf',drawingFormat:'ocdraw'},
 {kind:'ifccad',name:'old.ocdraw.json'},
])test(`parked OCDraw input is explicit and never dispatched: ${JSON.stringify(request)}`,()=>{
 let invoked=false;
 const wasm=new Proxy({}, {get(){invoked=true;throw Error('must not dispatch');}});
 const result=processBrowserRequest({...request,files:[file(request.name)]},wasm);
 assert.equal(result.failure.code,'OCDRAW_PARKED');
 assert.equal(result.validation?.strictAvailable??false,false);
 assert.equal(invoked,false);
});
