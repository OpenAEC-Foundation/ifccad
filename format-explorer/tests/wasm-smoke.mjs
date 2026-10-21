import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,conversion_capabilities,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_ifccad_binary,export_ifccad_with_options,convert_cad_to_ifccad_with_options} from '../wasm-build/browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';
import {createInspection} from '../src/inspection-model.mjs';
import {renderInspection} from '../src/inspection-view.mjs';
import {examples} from '../src/examples.mjs';
import {hatchValues} from '../src/hatch-view.mjs';

initSync({module:await readFile(new URL('../wasm-build/browser_bg.wasm',import.meta.url))});
const wasm={conversion_capabilities,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_ifccad_binary,export_ifccad_with_options,convert_cad_to_ifccad_with_options};

for(const example of examples){
 const kind='ifccad',name=example.path.split('/').at(-1),bytes=Uint8Array.from(await readFile(new URL('../../'+example.path,import.meta.url))),source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 const opened=processBrowserRequest(source,wasm);assert.equal(opened.failure,null,name+': '+JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);
 const expectedDimensions=(example.name==='House floor plan'?['10000','6100','3900','12000']:['58','56.92']).sort();
 assert.deepEqual(opened.presentation.dimensions.map(d=>d.geometry.label.primary).sort(),expectedDimensions,name+' has compact, unscaled dimension labels');
 for(const encoding of ['cbor','cbor-zstd']){
  const exported=processBrowserRequest({...source,export:{format:'ifccad',encoding}},wasm);
  assert.equal(exported.failure,null,name+': '+JSON.stringify(exported.failure));
  const file=exported.export.download,output=Buffer.from(file.base64,'base64');
  assert.equal(output.length,file.byteLength);
  assert.ok(file.fileName.endsWith(encoding==='cbor'?'.ifccad.cbor':'.ifccad.cbor.zst'));
  if(encoding==='cbor-zstd'){assert.deepEqual([...output.subarray(0,4)],[0x28,0xb5,0x2f,0xfd]);assert.equal(output[4]&4,4,'transport requires checksum');}
  else assert.ok(output.includes(Buffer.from('IFCCAD-CBOR')));
  const binary={kind:'ifccad',name:file.fileName,files:[{path:file.fileName,bytes:Uint8Array.from(output).buffer}]};
  const reopened=processBrowserRequest(binary,wasm);
  assert.equal(reopened.failure,null,JSON.stringify(reopened.failure));assert.equal(reopened.validation.strictAvailable,true);
  assert.equal(reopened.source.encoding,encoding);assert.equal(reopened.source.representation,'decoded');
  assert.equal(reopened.presentation.drawingId,opened.presentation.drawingId);
  // JSON defaults, XY/XYZ spelling and foreign IFCX attributes are source syntax.
  // Compare the typed assessments and identities here; native tests also compare
  // the entire validated documents after binary and reconstructed JSON readback.
  assert.deepEqual(reopened.presentation.entities.map(node=>node.path),opened.presentation.entities.map(node=>node.path));
  for(const key of ['unit','linePatternScale','boundsCompleteness','dimensions','dimensionStyles','hatchEntityCount','textEntityCount','mTextEntityCount','opaqueEntityCount'])assert.deepEqual(reopened.presentation[key],opened.presentation[key],key);
  const inspection=createInspection(reopened,reopened.nativeSourceText);assert.equal(inspection.binary,true);
  const drawing=Array.from(inspection.nodes.values()).find(node=>node.type==='drawing');
  const rendered=renderInspection(inspection,{language:'nl',selection:drawing.key,inspector:'source',expanded:new Set(),fieldExpanded:new Set(),fieldCollapsed:new Set()});
  assert.match(rendered.details,/Gedecodeerde knoop/);assert.doesNotMatch(rendered.details,/>Fragment \d/);
  for(const target of ['ifccad','dxf','dwg']){
   const saved=processBrowserRequest({...binary,export:{format:target,version:'AC1032'}},wasm);
   assert.equal(saved.failure,null,JSON.stringify(saved.failure));
   if(target==='ifccad'){
    const json=Buffer.from(saved.export.download.base64,'base64');
    const read=processBrowserRequest({kind:'ifccad',name:saved.export.download.fileName,files:[{path:saved.export.download.fileName,bytes:Uint8Array.from(json).buffer}]},wasm);
    assert.equal(read.failure,null);assert.deepEqual(read.presentation.entities,reopened.presentation.entities);
   }
  }
 }
 const model=createInspection(opened,new TextDecoder().decode(bytes)),nodes=Array.from(model.nodes.values());
 if(example.name==='Entity catalogue'){
  for(const type of ['point','lineSegment','circle','arc','ellipse','ellipseArc','planarPolyline','spatialPolyline','text','mText','hatch','dimension','blockInstance','viewport','solid','trace'])assert.ok(nodes.some(node=>node.type===type),name+' contains '+type);
  assert.deepEqual(nodes.filter(node=>node.type==='hatch').map(node=>hatchValues(node).fill.kind).sort(),['gradient','linePattern','solid']);
 }
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,name+' '+format+': '+JSON.stringify(output.failure));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'curated.'+format,files:[{path:'curated.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,name+': '+JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);
  assert.deepEqual(returned.presentation.dimensions.map(d=>d.geometry.label.primary).sort(),expectedDimensions,name+' retains dimension labels through '+format);
  if(example.name==='Entity catalogue'){
   assert.equal(returned.presentation.hatchEntityCount,3);
   const picture=returned.presentation.dimensions.find(d=>d.geometry.graphics?.partCount===6);assert.ok(picture,'complete stored catalogue picture survives '+format);assert.equal(picture.geometry.graphics.origin,'sourceSnapshot');
   for(const [kind,count] of [['solid',2],['trace',3]])assert.equal(returned.presentation.graph.data.filter(n=>n.attributes?.['ifccad::entity']&&n.attributes?.['ifccad::geom::'+kind]).length,count,'retains '+kind+' through '+format);
   const surfaces=output.export.geometryAssessment.filledSurfaces;assert.ok(surfaces.length>=5);
   for(const r of surfaces){assert.equal(r.surfaceCoverage,'Proven');assert.equal(r.positionScope,'FourBaseAndFourTop');assert.equal(r.displayEvaluation,'unassessed');}
  }

 }
}
console.log('Two curated IFCCAD examples open and pass DXF/DWG production strict readback');

for(const [kind,path]of [['ifccad','../../examples/ifccad/overview.ifcx']]){
 const name=path.split('/').at(-1),bytes=Uint8Array.from(await readFile(new URL(path,import.meta.url))),source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,JSON.stringify(output.failure));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'foreground.'+format,files:[{path:'foreground.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);
  const native=JSON.parse(returned.nativeSourceText);
  const color=kind==='ifccad'?native.data.map(node=>node.attributes?.['ifccad::layer']).find(layer=>layer?.name==='0').appearance.color:native.layers.find(layer=>layer.name==='0').color;
  assert.deepEqual(color,{kind:'Foreground'},`${kind} ${format} must retain background-adaptive Foreground through CAD ACI 7`);
 }
}
console.log('Example Foreground survives DXF/DWG ACI 7 production readback in IFCCAD');

for(const [kind,path]of [['ifccad','../../tests/fixtures/ifccad/hello-hatch-solid.ifcx'],['ifccad','../../tests/fixtures/ifccad/hello-hatch-pattern.ifcx']]){
 const name=path.split('/').at(-1),bytes=Uint8Array.from(await readFile(new URL(path,import.meta.url))),source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 const opened=processBrowserRequest(source,wasm);assert.equal(opened.failure,null,JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);assert.equal(opened.presentation.hatchEntityCount,1);
 const model=createInspection(opened,new TextDecoder().decode(bytes)),hatch=Array.from(model.nodes.values()).find(node=>node.type==='hatch');assert.ok(hatch);assert.ok(hatch.outgoing.some(edge=>edge.kind==='hatchSource'));
 const view=renderInspection(model,{language:'en',view:'drawing',selection:hatch.key,expanded:new Set(),inspector:'properties'});assert.match(view.details,/Fill.*not assessed/i);
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,JSON.stringify(output.failure));assert.ok(output.export.viewerSelection.entities.some(entry=>entry.path===hatch.key));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'hatch.'+format,files:[{path:'hatch.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);assert.equal(returned.presentation.hatchEntityCount,1);
  const resultModel=createInspection(returned,returned.nativeSourceText),resultHatch=Array.from(resultModel.nodes.values()).find(node=>node.type==='hatch');assert.ok(resultHatch.outgoing.some(edge=>edge.kind==='hatchSource'));
 }
}
console.log('Browser WASM Hatch inspection, source links, qualified selection and DXF/DWG strict readback verified for IFCCAD');

for(const [kind,path]of [['ifccad','../../tests/fixtures/ifccad/workspace-state.ifcx']]){
 const name=path.split('/').at(-1),bytes=new Uint8Array(await readFile(new URL(path,import.meta.url)));
 const source={kind,name,files:[{path:name,bytes:bytes.buffer}]};
 const opened=processBrowserRequest(source,wasm);assert.equal(opened.failure,null,JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);
 if(kind==='ifccad'){
  assert.equal(opened.presentation.ucsDefinitions[0].path,'/cad/d1/ucs/9007199254740993');
  assert.equal(opened.presentation.modelWindows[0].path,'/cad/d1/modelWindow/9007199254740993');
  const native=processBrowserRequest({...source,export:{format:'ifccad'}},wasm);assert.equal(native.failure,null);assert.deepEqual(Buffer.from(native.export.download.base64,'base64'),Buffer.from(bytes));
 }
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...source,export:{format,version:'AC1032'}},wasm);assert.equal(output.failure,null,JSON.stringify(output.failure));
  assert.ok(!output.export.diagnostics.some(d=>String(d.location).includes('grid.adaptive')));
  const returned=processBrowserRequest({kind:'cad',drawingFormat:'ifccad',name:'workspace.'+format,files:[{path:'workspace.'+format,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}]},wasm);
  assert.equal(returned.failure,null,JSON.stringify(returned.failure));assert.equal(returned.validation.strictAvailable,true);
  const canvas=returned.presentation.graph.data.find(n=>n.attributes?.['ifccad::paperCanvas']).attributes['ifccad::paperCanvas'];assert.equal(canvas.activeContext,undefined);assert.equal(canvas.frame.center[2],3);assert.equal(canvas.grid.adaptive,true);
 }
}
console.log('Browser WASM workspace fields, classified grid limitations and full-width references verified');

// A second active Paper tab must survive the complete CAD -> native -> CAD browser route.
const activePaperFixture=JSON.parse(await readFile(new URL('../../tests/fixtures/ifccad/workspace-state.ifcx',import.meta.url),'utf8'));
const activeDrawing=activePaperFixture.data.find(n=>n.attributes?.['ifccad::drawing']);
activeDrawing.attributes['ifccad::drawing'].nextLayoutId=4;
activeDrawing.children.paper3='/cad/d1/layout/3';
activeDrawing.attributes['ifccad::drawingWorkspace'].activeLayout='/cad/d1/layout/3';
activePaperFixture.data.push({path:'/cad/d1/layout/3',children:{},attributes:{'ifccad::layout':{kind:'Paper',name:'Selected second sheet',tabIndex:2}}});
const activePaperInput=new TextEncoder().encode(JSON.stringify(activePaperFixture));
const activePaperCad=processBrowserRequest({kind:'ifccad',name:'active-paper',files:[{path:'active-paper.ifcx',bytes:activePaperInput.buffer}],export:{format:'dxf',version:'AC1032'}},wasm);
assert.equal(activePaperCad.failure,null,JSON.stringify(activePaperCad.failure));
assert.equal(activePaperCad.validation.strictAvailable,true,JSON.stringify(activePaperCad.validation));
function activePaperName(format,opened){
 assert.equal(opened.failure,null,JSON.stringify(opened.failure));
 const native=JSON.parse(Buffer.from(opened.export.download.base64,'base64').toString('utf8'));
 const selected=native.data.find(n=>n.attributes?.['ifccad::drawingWorkspace'])?.attributes['ifccad::drawingWorkspace'].activeLayout;
 return native.data.find(n=>n.path===selected)?.attributes['ifccad::layout'].name;
}
for(const drawingFormat of ['ifccad']){
 const original={kind:'cad',drawingFormat,name:'active-paper',files:[{path:'active-paper.dxf',bytes:Uint8Array.from(Buffer.from(activePaperCad.export.download.base64,'base64')).buffer}]};
 assert.equal(activePaperName(drawingFormat,processBrowserRequest({...original,export:{format:drawingFormat}},wasm)),'Selected second sheet');
 for(const format of ['dxf','dwg']){
  const output=processBrowserRequest({...original,export:{format,version:'AC1032'}},wasm);
  assert.equal(output.failure,null,JSON.stringify(output.failure));
  const returned=processBrowserRequest({kind:'cad',drawingFormat,name:'active-paper-returned',files:[{path:`active-paper.${format}`,bytes:Uint8Array.from(Buffer.from(output.export.download.base64,'base64')).buffer}],export:{format:drawingFormat}},wasm);
  assert.equal(activePaperName(drawingFormat,returned),'Selected second sheet',`${drawingFormat} ${format}`);
 }
}
console.log('Browser WASM multi-sheet active Paper layout retained through IFCCAD and DXF/DWG');

const ifccadTextBytes=await readFile(new URL('../../examples/ifccad/text.ifcx',import.meta.url));
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
const splineBytes=await readFile(new URL('../../crates/viewer/tests/fixtures/open-cubic.dxf',import.meta.url));
const splineSource={kind:'cad',name:'spline.dxf',preserveSplines:true,files:[{path:'spline.dxf',bytes:Uint8Array.from(splineBytes).buffer}]};
const graph=JSON.parse(await readFile(new URL('../../tests/fixtures/ifccad/hello-line-patterns.ifcx',import.meta.url),'utf8'));
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

const viewportGraph=JSON.parse(await readFile(new URL('../../tests/fixtures/ifccad/hello-viewports.ifcx',import.meta.url),'utf8'));
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
const largeText=(await readFile(new URL('../../tests/fixtures/ifccad/hello-line-patterns.ifcx',import.meta.url),'utf8'))
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

const geometryBytes=new Uint8Array(await readFile(new URL('../../tests/fixtures/ifccad/hello-geometry.ifcx',import.meta.url)));
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

for (const format of ['ifccad']) {
 const extension='ifcx';
 for (const name of ['layout-medium-only','layout-plot-inch','layout-linetype-scaling']) {
  const bytes=new Uint8Array(await readFile(new URL(`../../conformance/next/${format}/valid/${name}.${extension}`,import.meta.url)));
  const source={kind:'ifccad',name:`${name}.${extension}`,files:[{path:`${name}.${extension}`,bytes:bytes.buffer}]};
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
console.log('Browser WASM medium-only, inch plot and layout linetype-scaling exchange verified for IFCCAD');

for(const [fixture,knownMapping] of [['layout-plot-inch',true],['layout-medium-only',false]]){
 const drawing=JSON.parse(await readFile(new URL(`../../conformance/next/ifccad/valid/${fixture}.ifcx`,import.meta.url),'utf8'));
 drawing.data.find(n=>n.attributes?.['ifccad::drawing']).attributes['ifccad::drawing'].lengthUnit='mm';
 const source={kind:'ifccad',name:fixture,files:[{path:fixture+'.ifcx',bytes:new TextEncoder().encode(JSON.stringify(drawing)).buffer}],conversionOptions:{tolerance:{mode:'custom',value:0.001,unit:'mm'}},export:{format:'dxf',version:'AC1032'}};
 const result=processBrowserRequest(source,wasm);
 if(knownMapping){
  assert.equal(result.failure,null,JSON.stringify(result.failure));
  const paper=result.export.geometryAssessment.domains.find(d=>d.coordinateMeaning.kind==='PaperCoordinates');
  assert.deepEqual(paper.coordinateMeaning.physicalOutputFactor,{numerator:'127',denominator:'5000'});
  assert.equal(result.export.geometryAssessment.maxDeviationUpperBound,undefined);
 }else{
  assert.equal(result.failure.geometry.domain.kind,'PaperLayout');
  assert.equal(result.failure.geometry.domain.layoutId,'2');
  assert.equal(result.failure.geometry.reason,'PhysicalMappingRequired');
  assert.equal(result.export?.download,undefined);
 }
}
console.log('Browser WASM custom physical tolerance reports per-layout output meaning and refuses missing Paper mapping');

{
 const native=Uint8Array.from(await readFile(new URL('../../tests/fixtures/ifccad/hello-line-patterns.ifcx',import.meta.url))).buffer;
 const exported=processBrowserRequest({kind:'ifccad',name:'lines.ifcx',files:[{path:'lines.ifcx',bytes:native}],export:{format:'dxf',version:'AC1032'}},wasm);
 assert.equal(exported.failure,null);const bytes=Uint8Array.from(Buffer.from(exported.export.download.base64,'base64')).buffer;
 for(const drawingFormat of ['ifccad']){
  const result=processBrowserRequest({kind:'cad',drawingFormat,name:'lines.dxf',files:[{path:'lines.dxf',bytes}],conversionOptions:{tolerance:{mode:'exact'}},exportConversionOptions:{tolerance:{mode:'custom',value:0.003,unit:'mm',coordinateFallback:1e-9}},export:{format:'dxf',version:'AC1032'}},wasm);
  assert.equal(result.failure,null,JSON.stringify(result.failure));assert.equal(result.conversion.options.tolerance.mode,'exact');assert.equal(result.export.options.tolerance.value,0.003);assert.equal(result.export.options.tolerance.coordinateFallback,1e-9);
 }
}
console.log('Browser WASM uses independent input and output tolerances through IFCCAD');

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

// Exercise an actual recoverable source error through the worker and WASM,
// including production strict readback and a subsequent CAD export.
{
 const input=await readFile(new URL('../../tests/fixtures/ifccad/hello-geometry.ifcx',import.meta.url));
 const cad=JSON.parse(export_ifccad('geometry.ifcx',input,'dxf','AC1032'));
 assert.equal(cad.failure,null,JSON.stringify(cad.failure));
 const lines=Buffer.from(cad.export.download.base64,'base64').toString('utf8').trimEnd().split(/\r?\n/);
 let inCircle=false,changed=false;
 for(let i=0;i<lines.length;i+=2){
  const code=Number(lines[i]),value=lines[i+1]?.trim();
  if(code===0)inCircle=value==='CIRCLE';
  if(inCircle&&code===40){lines[i+1]='-1.0';changed=true;break;}
 }
 assert.equal(changed,true,'geometry fixture must contain a circle');
 const broken=Uint8Array.from(Buffer.from(lines.join('\n')+'\n'));
 const request={kind:'cad',name:'invalid-circle.dxf',files:[{path:'invalid-circle.dxf',bytes:broken.buffer}]};
 const normal=processBrowserRequest(request,wasm);assert.equal(normal.failure.partialOpenAvailable,true);assert.notEqual(normal.validation?.strictAvailable,true);
 const partial={...request,conversionOptions:{partialOpen:true}};
 const opened=processBrowserRequest(partial,wasm);assert.equal(opened.failure,null,JSON.stringify(opened.failure));assert.equal(opened.validation.strictAvailable,true);assert.equal(opened.partialOpening.errorCount,1);
 const exported=processBrowserRequest({...partial,export:{format:'dxf',version:'AC1032'}},wasm);assert.equal(exported.failure,null,JSON.stringify(exported.failure));assert.equal(exported.partialOpening.errorCount,1);assert.ok(exported.export.download.base64);
 console.log('Browser WASM normal error, explicit partial opening, omission report and CAD readback verified');
}
