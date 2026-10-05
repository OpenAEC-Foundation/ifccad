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

for(const name of ['viewport-circle-clip','viewport-ellipse-clip','viewport-bulged-clip','viewport-forward-clip']){
 const original=JSON.parse(await readFile(new URL(`../../conformance/next/ocdraw/valid/${name}.ocdraw.json`,import.meta.url),'utf8'));
 const boundaryStream=Object.keys(original.streams).find(key=>key!=='viewportStream');
 for(const enabled of [false,true]){
  const drawing=structuredClone(original);
  drawing.streams.viewportStream.paperClip[0].enabled=enabled;
  drawing.streams.viewportStream.view[0].twist=Math.PI/6;
  const bytes=new TextEncoder().encode(JSON.stringify(drawing));
  const source={kind:'drawing',name,files:[{path:name+'.ocdraw.json',bytes:bytes.buffer}]};
  const exported=processBrowserRequest({...source,export:{format:'dxf',version:'AC1032'}},wasm);
  assert.equal(exported.failure,null,`${name} ${enabled}: ${JSON.stringify(exported.failure)}`);
  assert.equal(exported.export.diagnostics.some(d=>d.code==='DXF_VIEWPORT_CLIP_LOSS'),false);
  const returned=processBrowserRequest({kind:'cad',name:'clipped',files:[{path:'clipped.dxf',bytes:Uint8Array.from(Buffer.from(exported.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,`${name} ${enabled}: ${JSON.stringify(returned.failure)}`);
  assert.equal(returned.validation.strictAvailable,true);
  const viewport=returned.presentation.streams.viewportStream;
  assert.equal(viewport.count,1);
  assert.equal(viewport.paperClip[0].enabled,enabled);
  assert.ok(returned.presentation.streams[boundaryStream].entityId.includes(viewport.paperClip[0].boundaryEntityId));
  assert.ok(Math.abs(viewport.view[0].twist-Math.PI/6)<1e-12);
 }
}
console.log('Browser WASM patched DXF active/dormant clips, boundary links and radians verified');

const graph=JSON.parse(await readFile(new URL('../../examples/ifcx-native-cad/hello-line-patterns.ifcx',import.meta.url),'utf8'));
// Exercise the fixture's nonzero base through both real codecs without repair.
assert.deepEqual(graph.data.find(n=>n.path==='/cad/d1/block/1').attributes['ifccad::blockDefinition'].basePoint,[2,0,0]);
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
 assert.deepEqual(returned.presentation.blockDefinitions.find(n=>n.attributes['ifccad::blockDefinition']).attributes['ifccad::blockDefinition'].basePoint,[2,0,0]);
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

const viewportGraph=JSON.parse(await readFile(new URL('../../examples/ifcx-native-cad/hello-viewports.ifcx',import.meta.url),'utf8'));
const view=viewportGraph.data.find(node=>node.attributes?.['ifccad::viewport']).attributes['ifccad::viewport'];
for(const enabled of [false,true]){
 view.viewEnabled=enabled;view.visible=!enabled;view.viewLocked=true;
 view.paperClip.enabled=enabled;view.view.twist=Math.PI/6;
 const bytes=new TextEncoder().encode(JSON.stringify(viewportGraph));
 const source={kind:'ifcx',name:'viewports.ifcx',files:[{path:'viewports.ifcx',bytes:bytes.buffer}]};
 for(const format of ['ifcx','dxf','dwg']){
  const exported=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);
  assert.equal(exported.failure,null,JSON.stringify(exported.failure));
  const returned=processBrowserRequest({kind:format==='ifcx'?'ifcx':'cad',drawingFormat:'ifcx',name:'viewports.'+format,
   files:[{path:'viewports.'+format,bytes:Uint8Array.from(Buffer.from(exported.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));
  assert.equal(returned.validation.strictAvailable,true);
  const views=returned.presentation.entities.filter(node=>node.attributes['ifccad::viewport']);
  assert.equal(views.length,1);
  const restored=views[0].attributes['ifccad::viewport'];
  assert.equal(restored.viewEnabled,enabled);assert.equal(restored.visible,!enabled);
  assert.equal(restored.viewLocked,true);assert.equal(restored.paperClip.enabled,enabled);
  assert.equal(restored.view.projection,'Perspective');assert.equal(restored.view.lensLengthMm,50);
  assert.deepEqual(restored.view.direction,[0,0,100]);
  assert.ok(Math.abs(restored.view.twist-Math.PI/6)<1e-12);
  const boundary=returned.presentation.entities.find(node=>node.path===restored.paperClip.boundary);
  assert.equal(boundary.attributes['ifccad::geom::circle'].radius,50);
  const model=returned.presentation.layouts.find(node=>node.attributes['ifccad::layout'].kind==='Model');
  assert.equal(restored.model,model.path);
  assert.deepEqual(restored.frozenLayers.map(path=>returned.presentation.layers.find(node=>node.path===path).attributes['ifccad::layer'].name),['Notes']);
 }
}
console.log('Browser WASM IFCX-CAD perspective, active/dormant circle clips and independent display states verified');

// Keep full-width counter literals in text; JSON.parse/stringify would round them.
const largeText=(await readFile(new URL('../../examples/ifcx-native-cad/hello-line-patterns.ifcx',import.meta.url),'utf8'))
 .replace(/("next(?:Entity|Layer|Layout|Block|LinePattern)Id"\s*:\s*)\d+/g,(_,prefix)=>prefix+'9007199254740993');
const largeBytes=new TextEncoder().encode(largeText);
const largeSource={kind:'ifcx',name:'large.ifcx',files:[{path:'large.ifcx',bytes:largeBytes.buffer}]};
const largeOpened=processBrowserRequest(largeSource,wasm);
assert.equal(largeOpened.validation.strictAvailable,true);
const largeExported=processBrowserRequest({...largeSource,export:{format:'ifcx'}},wasm);
assert.equal(largeExported.failure,null);
const largeReturned=Buffer.from(largeExported.export.download.base64,'base64');
assert.deepEqual(largeReturned,Buffer.from(largeBytes));
assert.equal(processBrowserRequest({kind:'ifcx',name:'again.ifcx',files:[{path:'again.ifcx',bytes:Uint8Array.from(largeReturned).buffer}]},wasm).validation.strictAvailable,true);
console.log('Browser WASM full-width IFCX-CAD allocation state preserved in native download');
