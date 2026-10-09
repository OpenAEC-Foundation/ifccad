import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,conversion_capabilities,open_drawing,convert_cad_to_drawing,convert_cad_to_drawing_with_preservation,export_drawing,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_drawing_with_options,convert_cad_to_drawing_with_options,export_ifccad_with_options,convert_cad_to_ifccad_with_options} from '../wasm-build/browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';

initSync({module:await readFile(new URL('../wasm-build/browser_bg.wasm',import.meta.url))});
const wasm={conversion_capabilities,open_drawing,convert_cad_to_drawing,convert_cad_to_drawing_with_preservation,export_drawing,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_drawing_with_options,convert_cad_to_drawing_with_options,export_ifccad_with_options,convert_cad_to_ifccad_with_options};

for(const [kind,path]of [['ifccad','../../examples/ifccad/hello-hatch-solid.ifcx'],['drawing','../../examples/ocdraw/hello-hatch-solid.ocdraw.json'],['ifccad','../../examples/ifccad/hello-hatch-pattern.ifcx'],['drawing','../../examples/ocdraw/hello-hatch-pattern.ocdraw.json']]){
 const name=path.split('/').at(-1),bytes=Uint8Array.from(await readFile(new URL(path,import.meta.url))),source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 const opened=processBrowserRequest(source,wasm);assert.equal(opened.failure,null,JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);assert.equal(opened.presentation.hatchEntityCount,1);
 const model=createInspection(opened,new TextDecoder().decode(bytes)),hatch=Array.from(model.nodes.values()).find(node=>node.type==='hatch');assert.ok(hatch);assert.ok(hatch.outgoing.some(edge=>edge.kind==='hatchSource'));
 const view=renderInspection(model,{language:'en',view:'drawing',selection:hatch.key,expanded:new Set(),inspector:'properties'});assert.match(view.details,/Fill.*not assessed/i);
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,JSON.stringify(output.failure));assert.ok(output.export.viewerSelection.entities.some(entry=>entry.path===hatch.key));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:kind==='ifccad'?'ifccad':'ocdraw',name:'hatch.'+format,files:[{path:'hatch.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);assert.equal(returned.presentation.hatchEntityCount,1);
  const resultModel=createInspection(returned,returned.nativeSourceText),resultHatch=Array.from(resultModel.nodes.values()).find(node=>node.type==='hatch');assert.ok(resultHatch.outgoing.some(edge=>edge.kind==='hatchSource'));
 }
}
console.log('Browser WASM Hatch inspection, source links, qualified selection and DXF/DWG strict readback verified for both formats');

for(const [kind,path]of [['drawing','../../examples/ocdraw/workspace-state.ocdraw.json'],['ifccad','../../examples/ifccad/workspace-state.ifcx']]){
 const name=path.split('/').at(-1),bytes=new Uint8Array(await readFile(new URL(path,import.meta.url)));
 const source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 const opened=processBrowserRequest(source,wasm);assert.equal(opened.failure,null,JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);
 if(kind==='ifccad'){
  assert.equal(opened.presentation.ucsDefinitions[0].path,'/cad/d1/ucs/9007199254740993');
  assert.equal(opened.presentation.modelWindows[0].path,'/cad/d1/modelWindow/9007199254740993');
  const native=processBrowserRequest({...source,export:{format:'ifccad'}},wasm);assert.equal(native.failure,null);assert.deepEqual(Buffer.from(native.export.download.base64,'base64'),Buffer.from(bytes));
 }else{assert.equal(opened.presentation.paperCanvases[0].activeContext,undefined);assert.equal(opened.presentation.viewportWorkspaces[0].useStoredUcs,false);}
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,JSON.stringify(output.failure));
  assert.ok(output.export.diagnostics.some(d=>String(d.location).includes('grid.adaptive')));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:kind==='drawing'?'ocdraw':'ifccad',name:'workspace.'+format,files:[{path:'workspace.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);
  if(kind==='drawing'){assert.equal(returned.presentation.paperCanvases[0].activeContext,undefined);assert.equal(returned.presentation.paperCanvases[0].frame.center.z,3);}
  else{const canvas=returned.presentation.graph.data.find(n=>n.attributes?.['ifccad::paperCanvas']).attributes['ifccad::paperCanvas'];assert.equal(canvas.activeContext,undefined);assert.equal(canvas.frame.center[2],3);}
 }
}
console.log('Browser WASM workspace fields, classified grid limitations and full-width references verified');

// A second active Paper tab must survive the complete CAD -> native -> CAD browser route.
const activePaperFixture=JSON.parse(await readFile(new URL('../../examples/ocdraw/workspace-state.ocdraw.json',import.meta.url),'utf8'));
activePaperFixture.header.nextLayoutId=3;
activePaperFixture.layouts.push({id:2,scopeId:2,kind:'paper',name:'Selected second sheet',tabIndex:2});
activePaperFixture.scopes.push({id:2,kind:1,entities:[],bounds:null});
activePaperFixture.drawingWorkspaceState={activeLayoutId:2};
const activePaperInput=new TextEncoder().encode(JSON.stringify(activePaperFixture));
const activePaperCad=processBrowserRequest({kind:'drawing',name:'active-paper',files:[{path:'active-paper.ocdraw.json',bytes:activePaperInput.buffer}],export:{format:'dxf',version:'AC1032'}},wasm);
assert.equal(activePaperCad.failure,null,JSON.stringify(activePaperCad.failure));
assert.equal(activePaperCad.validation.strictAvailable,true,JSON.stringify(activePaperCad.validation));
function activePaperName(format,opened){
 assert.equal(opened.failure,null,JSON.stringify(opened.failure));
 const native=JSON.parse(Buffer.from(opened.export.download.base64,'base64').toString('utf8'));
 if(format==='ocdraw')return native.layouts.find(l=>l.id===native.drawingWorkspaceState?.activeLayoutId)?.name;
 const selected=native.data.find(n=>n.attributes?.['ifccad::drawingWorkspace'])?.attributes['ifccad::drawingWorkspace'].activeLayout;
 return native.data.find(n=>n.path===selected)?.attributes['ifccad::layout'].name;
}
for(const drawingFormat of ['ocdraw','ifccad']){
 const original={kind:'cad',drawingFormat,name:'active-paper',files:[{path:'active-paper.dxf',bytes:Uint8Array.from(Buffer.from(activePaperCad.export.download.base64,'base64')).buffer}]};
 assert.equal(activePaperName(drawingFormat,processBrowserRequest({...original,export:{format:drawingFormat}},wasm)),'Selected second sheet');
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...original,export:{format,version:'AC1032'}},wasm);
  assert.equal(output.failure,null,JSON.stringify(output.failure));
  const returned=processBrowserRequest({kind:'cad',drawingFormat,name:'active-paper-returned',files:[{path:`active-paper.${format}`,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}],export:{format:drawingFormat}},wasm);
  assert.equal(activePaperName(drawingFormat,returned),'Selected second sheet',`${drawingFormat} ${format}`);
 }
}
console.log('Browser WASM multi-sheet active Paper layout retained through both native routes and DXF/DWG');

const textBytes=await readFile(new URL('../../examples/ocdraw/text.ocdraw.json',import.meta.url));
const textSource={kind:'drawing',name:'text.ocdraw.json',files:[{path:'text.ocdraw.json',bytes:Uint8Array.from(textBytes).buffer}]};
const textOpened=processBrowserRequest(textSource,wasm);
assert.equal(textOpened.failure,null,JSON.stringify(textOpened.failure));
assert.equal(textOpened.presentation.entities.filter(e=>e.geometry?.type==='text').length,2);
assert.equal(textOpened.presentation.entities.filter(e=>e.geometry?.type==='mText').length,1);
assert.equal(textOpened.presentation.boundsCompleteness[0].quality,'estimated');
assert.equal(textOpened.presentation.boundsCompleteness[0].enclosureVerified,false);
for(const format of ['dxf','dwg']){
 const output=processBrowserRequest({...textSource,export:{format,version:'AC1032'}},wasm);
 assert.equal(output.failure,null,JSON.stringify(output.failure));
 assert.equal(output.export.geometry.complete,false);
 assert.equal(output.export.text.entries.length,3);
 const returned=processBrowserRequest({kind:'cad',drawingFormat:'ocdraw',name:'text.'+format,files:[{path:'text.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));
 assert.equal(returned.presentation.entities.filter(e=>e.geometry?.type==='mText').length,1);
}
console.log('Browser WASM OCDraw Text/MText native inspection, estimated bounds and actual DXF/DWG exchange verified');
const ifccadTextBytes=await readFile(new URL('../../examples/ifccad/hello-text.ifcx',import.meta.url));
const ifccadTextSource={kind:'ifccad',name:'text.ifcx',files:[{path:'text.ifcx',bytes:Uint8Array.from(ifccadTextBytes).buffer}]};
const ifccadTextOpened=processBrowserRequest(ifccadTextSource,wasm);
assert.equal(ifccadTextOpened.failure,null,JSON.stringify(ifccadTextOpened.failure));
assert.equal(ifccadTextOpened.presentation.textEntityCount,3);assert.equal(ifccadTextOpened.presentation.mTextEntityCount,3);
assert.equal(ifccadTextOpened.presentation.boundsCompleteness.find(s=>s.scopePath==='/cad/d1/layout/1').quality,'estimated');
for(const format of ['dxf','dwg']){
 const output=processBrowserRequest({...ifccadTextSource,export:{format,version:'AC1032'}},wasm);
 assert.equal(output.failure,null,JSON.stringify(output.failure));assert.equal(output.conversion.textAssessment.entries.length,6);
 const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'text.'+format,files:[{path:'text.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.presentation.textEntityCount,3);assert.equal(returned.presentation.mTextEntityCount,3);
}
console.log('Browser WASM IFCCAD Text/MText, estimated bounds and Model/Paper/block DXF/DWG exchange verified');
const splineBytes=await readFile(new URL('../../crates/ocdraw-convert/tests/fixtures/splines/open-cubic.dxf',import.meta.url));
const splineSource={kind:'cad',name:'spline.dxf',preserveSplines:true,files:[{path:'spline.dxf',bytes:Uint8Array.from(splineBytes).buffer}]};
const splineOpened=processBrowserRequest(splineSource,wasm);
assert.equal(splineOpened.failure,null,JSON.stringify(splineOpened.failure));
assert.equal(splineOpened.presentation.opaqueEntityCount,1);assert.equal(splineOpened.conversion.geometry.complete,false);
const splineNative=processBrowserRequest({...splineSource,export:{format:'ocdraw'}},wasm);
const splineReopened=processBrowserRequest({kind:'drawing',name:'saved.ocdraw.json',files:[{path:'saved.ocdraw.json',bytes:Uint8Array.from(Buffer.from(splineNative.export.download.base64,'base64')).buffer}]},wasm);
assert.equal(splineReopened.presentation.opaqueEntityCount,1);
for(const format of ['dxf','dwg']){
 const out=processBrowserRequest({...splineSource,export:{format,version:'AC1032'}},wasm);
 assert.equal(out.failure,null,JSON.stringify(out.failure));
 assert.ok(out.conversion.restoration.preservation.entries.some(e=>e.result==='restoredTyped'));
 const returned=processBrowserRequest({kind:'cad',name:'returned.'+format,preserveSplines:true,files:[{path:'returned.'+format,bytes:Uint8Array.from(Buffer.from(out.export.download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.presentation.opaqueEntityCount,1);
}
console.log('Browser WASM opt-in spline preservation, durable OCDraw readback and DXF/AC1032 DWG exchange verified');
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

const graph=JSON.parse(await readFile(new URL('../../examples/ifccad/hello-line-patterns.ifcx',import.meta.url),'utf8'));
// Exercise the fixture's nonzero base through both real codecs without repair.
assert.deepEqual(graph.data.find(n=>n.path==='/cad/d1/block/1').attributes['ifccad::blockDefinition'].basePoint,[2,0,0]);
const ifcxBytes=new TextEncoder().encode(JSON.stringify(graph));
const ifcxSource={kind:'ifccad',name:'hello.ifcx',files:[{path:'hello.ifcx',bytes:ifcxBytes.buffer}]};
assert.equal(processBrowserRequest(ifcxSource,wasm).presentation.linePatterns.length,3);
for(const format of ['ifccad','dxf','dwg']){
 const exported=processBrowserRequest({...ifcxSource,export:{format,version:'AC1032'}},wasm);
 assert.equal(exported.failure,null,JSON.stringify(exported.failure));
 const download=exported.export.download;
 const returned=processBrowserRequest({kind:format==='ifccad'?'ifccad':'cad',drawingFormat:'ifccad',name:download.fileName,files:[{path:download.fileName,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));
 assert.equal(returned.validation.strictAvailable,true);
 const patterns=returned.presentation.linePatterns.map(n=>n.attributes['ifccad::linePattern']);
 assert.deepEqual(returned.presentation.blockDefinitions.find(n=>n.attributes['ifccad::blockDefinition']).attributes['ifccad::blockDefinition'].basePoint,[2,0,0]);
 assert.deepEqual(patterns.find(p=>p.name==='DashDot').pattern,[0.5,-0.25,0,-0.25]);
 assert.ok(patterns.find(p=>p.name==='UnusedSolid'));
 if(format!=='ifccad'){
  const saved=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'again.'+format,files:[{path:'again.'+format,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}],export:{format:'ifccad'}},wasm);
  assert.equal(saved.export.download.format,'ifccad');
  const reopened=processBrowserRequest({kind:'ifccad',name:'reopened.ifcx',files:[{path:'reopened.ifcx',bytes:Uint8Array.from(Buffer.from(saved.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(reopened.validation.strictAvailable,true);
 }
}
console.log('Browser WASM IFCCAD opening, named patterns and real DXF/DWG roundtrips verified');

const viewportGraph=JSON.parse(await readFile(new URL('../../examples/ifccad/hello-viewports.ifcx',import.meta.url),'utf8'));
const viewportNode=viewportGraph.data.find(node=>node.attributes?.['ifccad::viewport']);
const view=viewportNode.attributes['ifccad::viewport'];
for(const enabled of [false,true]){
 view.viewEnabled=enabled;viewportNode.attributes['ifccad::entity'].visible=!enabled;view.viewLocked=true;
 view.paperClip.enabled=enabled;view.view.twist=Math.PI/6;
 const bytes=new TextEncoder().encode(JSON.stringify(viewportGraph));
 const source={kind:'ifccad',name:'viewports.ifcx',files:[{path:'viewports.ifcx',bytes:bytes.buffer}]};
 for(const format of ['ifccad','dxf','dwg']){
  const exported=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);
  assert.equal(exported.failure,null,JSON.stringify(exported.failure));
  const returned=processBrowserRequest({kind:format==='ifccad'?'ifccad':'cad',drawingFormat:'ifccad',name:'viewports.'+format,
   files:[{path:exported.export.download.fileName,bytes:Uint8Array.from(Buffer.from(exported.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));
  assert.equal(returned.validation.strictAvailable,true);
  const views=returned.presentation.entities.filter(node=>node.attributes['ifccad::viewport']);
  assert.equal(views.length,1);
  const restored=views[0].attributes['ifccad::viewport'];
  assert.equal(restored.viewEnabled,enabled);assert.equal(views[0].attributes['ifccad::entity'].visible??true,!enabled);
  assert.equal(restored.viewLocked,true);assert.equal(restored.paperClip.enabled,enabled);
  assert.equal(restored.view.projection,'Perspective');assert.equal(restored.view.lensLengthMm,50);
  assert.deepEqual(restored.view.direction,[0,0,100]);
  assert.ok(Math.abs(restored.view.twist-Math.PI/6)<1e-12);
  const boundary=returned.presentation.entities.find(node=>node.path===restored.paperClip.boundary);
  assert.equal(boundary.attributes['ifccad::geom::circle'].radius,50);
  const model=returned.presentation.layouts.find(node=>node.attributes['ifccad::layout'].kind==='Model');
  assert.equal(restored.model,model.path);
  assert.deepEqual(restored.layerOverrides.filter(row=>row.frozen).map(row=>row.layer).map(path=>returned.presentation.layers.find(node=>node.path===path).attributes['ifccad::layer'].name),['Notes']);
 }
}
console.log('Browser WASM IFCCAD perspective, active/dormant circle clips and independent display states verified');

// Keep full-width counter literals in text; JSON.parse/stringify would round them.
const largeText=(await readFile(new URL('../../examples/ifccad/hello-line-patterns.ifcx',import.meta.url),'utf8'))
 .replace(/("next(?:Entity|Layer|Layout|Block|LinePattern|Ucs|ModelWindow)Id"\s*:\s*)\d+/g,(_,prefix)=>prefix+'9007199254740993');
const largeBytes=new TextEncoder().encode(largeText);
const largeSource={kind:'ifccad',name:'large.ifcx',files:[{path:'large.ifcx',bytes:largeBytes.buffer}]};
const largeOpened=processBrowserRequest(largeSource,wasm);
assert.equal(largeOpened.validation.strictAvailable,true);
const largeExported=processBrowserRequest({...largeSource,export:{format:'ifccad'}},wasm);
assert.equal(largeExported.failure,null);
const largeReturned=Buffer.from(largeExported.export.download.base64,'base64');
assert.deepEqual(largeReturned,Buffer.from(largeBytes));
assert.equal(processBrowserRequest({kind:'ifccad',name:'again.ifcx',files:[{path:'again.ifcx',bytes:Uint8Array.from(largeReturned).buffer}]},wasm).validation.strictAvailable,true);
console.log('Browser WASM full-width IFCCAD allocation state preserved in native download');

const geometryBytes=new Uint8Array(await readFile(new URL('../../examples/ifccad/hello-geometry.ifcx',import.meta.url)));
const geometrySource={kind:'ifccad',name:'geometry.ifcx',files:[{path:'geometry.ifcx',bytes:geometryBytes.buffer}]};
const geometryOpened=processBrowserRequest(geometrySource,wasm);
assert.equal(geometryOpened.validation.strictAvailable,true);
assert.ok(geometryOpened.presentation.layouts.some(n=>n.attributes['ifccad::layout'].bounds));
for(const format of ['dxf','dwg']){
 const exported=processBrowserRequest({...geometrySource,export:{format,version:'AC1032'}},wasm);
 assert.equal(exported.failure,null,JSON.stringify(exported.failure));
 assert.ok(exported.export.geometryAssessment.domains.length);
 assert.ok(exported.export.fileCheck.geometryAssessment.domains.length);
 const download=exported.export.download;
 const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:download.fileName,
  files:[{path:download.fileName,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
 assert.equal(returned.failure,null,JSON.stringify(returned.failure));
 assert.equal(returned.validation.strictAvailable,true);
 assert.ok(returned.conversion.geometryAssessment.domains.length);
 for(const kind of ['point','arc','ellipse','ellipseArc','planarPolyline','spatialPolyline']){
  assert.ok(returned.presentation.entities.some(n=>n.attributes['ifccad::geom::'+kind]));
 }
}
console.log('Browser WASM expanded IFCCAD geometry, bounds and conversion evidence verified');

for (const format of ['ocdraw','ifccad']) {
 const extension=format==='ocdraw'?'ocdraw.json':'ifcx';
 for (const name of ['layout-medium-only','layout-plot-inch','layout-linetype-scaling']) {
  const bytes=new Uint8Array(await readFile(new URL(`../../conformance/next/${format}/valid/${name}.${extension}`,import.meta.url)));
  const source={kind:format==='ocdraw'?'drawing':'ifccad',name:`${name}.${extension}`,files:[{path:`${name}.${extension}`,bytes:bytes.buffer}]};
  const opened=processBrowserRequest(source,wasm);
  assert.equal(opened.failure,null,JSON.stringify(opened.failure));
  assert.equal(opened.validation.strictAvailable,true);
  for (const target of ['dxf','dwg']) {
   const exported=processBrowserRequest({...source,export:{format:target,version:'AC1032'}},wasm);
   assert.equal(exported.failure,null,JSON.stringify(exported.failure));
   const download=exported.export.download;
   const returned=processBrowserRequest({kind:'cad',drawingFormat:format,name:download.fileName,files:[{path:download.fileName,bytes:Uint8Array.from(Buffer.from(download.base64,'base64')).buffer}]},wasm);
   assert.equal(returned.failure,null,JSON.stringify(returned.failure));
   assert.equal(returned.validation.strictAvailable,true);
  }
 }
}
console.log('Browser WASM medium-only, inch plot and layout linetype-scaling exchange verified for both independent routes');

for(const [fixture,knownMapping] of [['layout-plot-inch',true],['layout-medium-only',false]]){
 const drawing=JSON.parse(await readFile(new URL(`../../conformance/next/ocdraw/valid/${fixture}.ocdraw.json`,import.meta.url),'utf8'));
 drawing.header.unit='mm';
 const source={kind:'drawing',name:fixture,files:[{path:fixture+'.ocdraw.json',bytes:new TextEncoder().encode(JSON.stringify(drawing)).buffer}],conversionOptions:{tolerance:{mode:'custom',value:0.001,unit:'mm'}},export:{format:'dxf',version:'AC1032'}};
 const result=processBrowserRequest(source,wasm);
 if(knownMapping){
  assert.equal(result.failure,null,JSON.stringify(result.failure));
  const paper=result.export.geometry.domains.find(d=>d.coordinateMeaning.kind==='PaperCoordinates');
  assert.deepEqual(paper.coordinateMeaning.physicalOutputFactor,{numerator:'127',denominator:'5000'});
  assert.equal(result.export.geometry.maxDeviationUpperBound,undefined);
 }else{
  assert.equal(result.failure.geometry.domain.kind,'PaperLayout');
  assert.equal(result.failure.geometry.domain.layoutId,'1');
  assert.equal(result.failure.geometry.reason,'PhysicalMappingRequired');
  assert.equal(result.export?.download,undefined);
 }
}
console.log('Browser WASM custom physical tolerance reports per-layout output meaning and refuses missing Paper mapping');

{
 const native=Uint8Array.from(await readFile(new URL('../../examples/ifccad/hello-line-patterns.ifcx',import.meta.url))).buffer;
 const exported=processBrowserRequest({kind:'ifccad',name:'lines.ifcx',files:[{path:'lines.ifcx',bytes:native}],export:{format:'dxf',version:'AC1032'}},wasm);
 assert.equal(exported.failure,null);const bytes=Uint8Array.from(Buffer.from(exported.export.download.base64,'base64')).buffer;
 for(const drawingFormat of ['ifccad','ocdraw']){
  const result=processBrowserRequest({kind:'cad',drawingFormat,name:'lines.dxf',files:[{path:'lines.dxf',bytes}],conversionOptions:{tolerance:{mode:'exact'}},exportConversionOptions:{tolerance:{mode:'custom',value:0.003,unit:'mm',coordinateFallback:1e-9}},export:{format:'dxf',version:'AC1032'}},wasm);
  assert.equal(result.failure,null,JSON.stringify(result.failure));assert.equal(result.conversion.options.tolerance.mode,'exact');assert.equal(result.export.options.tolerance.value,0.003);assert.equal(result.export.options.tolerance.coordinateFallback,1e-9);
 }
}
console.log('Browser WASM uses independent input and output tolerances through both native CAD routes');

const ifccadSpline={...splineSource,drawingFormat:'ifccad'};
const ifccadSplineOpened=processBrowserRequest(ifccadSpline,wasm);
assert.equal(ifccadSplineOpened.failure,null,JSON.stringify(ifccadSplineOpened.failure));
assert.equal(ifccadSplineOpened.presentation.opaqueEntityCount,1);
assert.equal(ifccadSplineOpened.conversion.geometryAssessment.complete,false);
assert.ok(ifccadSplineOpened.conversion.preservation.entries.some(e=>e.result==='capturedTyped'));
const ifccadSplineSaved=processBrowserRequest({...ifccadSpline,export:{format:'ifccad'}},wasm);
const savedIfccad={kind:'ifccad',name:'saved.ifcx',files:[{path:'saved.ifcx',bytes:Uint8Array.from(Buffer.from(ifccadSplineSaved.export.download.base64,'base64')).buffer}]};
assert.equal(processBrowserRequest(savedIfccad,wasm).presentation.opaqueEntityCount,1);
for(const format of ['dxf','dwg']){
 const output=processBrowserRequest({...savedIfccad,export:{format,version:'AC1032'}},wasm);
 assert.equal(output.failure,null,JSON.stringify(output.failure));
 assert.ok(output.conversion.preservation.entries.some(e=>e.result==='restoredTyped'));
 assert.equal(output.export.geometryAssessment.complete,false);
 assert.equal(output.export.fileCheck.opaqueEntityCount,1);
}
console.log('Browser WASM IFCCAD opaque spline capture, durable reopen and actual DXF/DWG exchange verified');
