import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,open_drawing,convert_cad_to_drawing,export_drawing,open_ifcx,convert_cad_to_ifcx,export_ifcx} from '../wasm-build/ocdraw_browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';

initSync({module:await readFile(new URL('../wasm-build/ocdraw_browser_bg.wasm',import.meta.url))});
const wasm={open_drawing,convert_cad_to_drawing,export_drawing,open_ifcx,convert_cad_to_ifcx,export_ifcx};
for(const name of ['ordered-scopes','fractional-polylines','paper-viewport','named-line-patterns']){
 const bytes=await readFile(new URL(`../../conformance/next/ocdraw/valid/${name}.ocdraw.json`,import.meta.url));
 const source={kind:'drawing',name,files:[{path:name+'.ocdraw.json',bytes:Uint8Array.from(bytes).buffer}]};
 const opened=processBrowserRequest(source,wasm);
 assert.equal(opened.failure,null,name);assert.equal(opened.validation.strictAvailable,true,name);
 assert.ok(opened.presentation?.scopes?.length>0,name);
 if(name==='named-line-patterns'){
  assert.equal(opened.presentation.linePatterns[1].name,'DashDot');
  assert.deepEqual(opened.presentation.linePatterns[1].pattern,[6,-2,0,-2]);
  assert.equal(opened.presentation.linePatternScale,2);
 }
 for(const format of ['ocdraw','dxf','dwg']){
  const exported=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);
  assert.equal(exported.failure,null,`${name} ${format}: ${JSON.stringify(exported.failure)}`);
  assert.equal(exported.validation.strictAvailable,true);
  if(name==='named-line-patterns'&&format!=='ocdraw'){
   assert.equal(exported.export.diagnostics.some(d=>d.code==='DWG_SPATIAL_PATTERN_GENERATION_LOSS'),format==='dwg');
  }
  const download=exported.export.download;assert.equal(download.format,format);
  const returned=processBrowserRequest({kind:format==='ocdraw'?'drawing':'cad',name:'returned',files:[{path:'returned.'+(format==='ocdraw'?'ocdraw.json':format),bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,`${name} ${format}: ${JSON.stringify(returned.failure)}`);
  assert.equal(returned.validation.strictAvailable,true);
 }
}
console.log('Browser WASM standalone OCDraw, DXF/DWG export and production readback verified');

const graph=JSON.parse(await readFile(new URL('../../examples/ifcx-native-cad/hello-line-patterns.ifcx',import.meta.url),'utf8'));
// Zero base isolates the supported DWG roundtrip from the pinned nonzero marker limitation.
graph.data.find(n=>n.path==='/cad/d1/block/1').attributes['ifccad::blockDefinition'].basePoint=[0,0,0];
const ifcxBytes=new TextEncoder().encode(JSON.stringify(graph));
const ifcxSource={kind:'ifcx',name:'hello.ifcx',files:[{path:'hello.ifcx',bytes:ifcxBytes.buffer}]};
assert.equal(processBrowserRequest(ifcxSource,wasm).presentation.linePatterns.length,3);
for(const format of ['ifcx','dxf','dwg']){
 const exported=processBrowserRequest({...ifcxSource,export:{format,version:'AC1032'}},wasm);
 assert.equal(exported.failure,null,JSON.stringify(exported.failure));
 const download=exported.export.download;
 const returned=processBrowserRequest({kind:format==='ifcx'?'ifcx':'cad',drawingFormat:'ifcx',name:'returned.'+format,files:[{path:'returned.'+format,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));
 assert.equal(returned.validation.strictAvailable,true);
 const patterns=returned.presentation.linePatterns.map(n=>n.attributes['ifccad::linePattern']);
 assert.deepEqual(patterns.find(p=>p.name==='DashDot').pattern,[0.5,-0.25,0,-0.25]);
 assert.ok(patterns.find(p=>p.name==='UnusedSolid'));
 if(format!=='ifcx'){
  const saved=processBrowserRequest({kind:'cad',drawingFormat:'ifcx',name:'again.'+format,files:[{path:'again.'+format,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}],export:{format:'ifcx'}},wasm);
  assert.equal(saved.export.download.format,'ifcx');
  const reopened=processBrowserRequest({kind:'ifcx',name:'reopened.ifcx',files:[{path:'reopened.ifcx',bytes:Uint8Array.from(Buffer.from(saved.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(reopened.validation.strictAvailable,true);
 }
}
console.log('Browser WASM IFCX-CAD opening, named patterns and real DXF/DWG roundtrips verified');
