import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,open_drawing,convert_cad_to_drawing,export_drawing} from '../wasm-build/ocdraw_browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';

initSync({module:await readFile(new URL('../wasm-build/ocdraw_browser_bg.wasm',import.meta.url))});
const wasm={open_drawing,convert_cad_to_drawing,export_drawing};
for(const name of ['ordered-scopes','fractional-polylines','paper-viewport']){
 const bytes=await readFile(new URL(`../../conformance/next/ocdraw/valid/${name}.ocdraw.json`,import.meta.url));
 const source={kind:'drawing',name,files:[{path:name+'.ocdraw.json',bytes:Uint8Array.from(bytes).buffer}]};
 const opened=processBrowserRequest(source,wasm);
 assert.equal(opened.failure,null,name);assert.equal(opened.validation.strictAvailable,true,name);
 assert.ok(opened.presentation?.scopes?.length>0,name);
 for(const format of ['ocdraw','dxf','dwg']){
  const exported=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);
  assert.equal(exported.failure,null,`${name} ${format}: ${JSON.stringify(exported.failure)}`);
  assert.equal(exported.validation.strictAvailable,true);
  const download=exported.export.download;assert.equal(download.format,format);
  const returned=processBrowserRequest({kind:format==='ocdraw'?'drawing':'cad',name:'returned',files:[{path:'returned.'+(format==='ocdraw'?'ocdraw.json':format),bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,`${name} ${format}: ${JSON.stringify(returned.failure)}`);
  assert.equal(returned.validation.strictAvailable,true);
 }
}
console.log('Browser WASM standalone OCDraw, DXF/DWG export and production readback verified');
