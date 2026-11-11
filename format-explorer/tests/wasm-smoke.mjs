import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,conversion_capabilities,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_ifccad_binary,export_ifccad_with_options,convert_cad_to_ifccad_with_options} from '../wasm-build/browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';
import {nativeExportRequest} from '../src/native-request.mjs';
import {createInspection} from '../src/inspection-model.mjs';
import {examples} from '../src/examples.mjs';

initSync({module:await readFile(new URL('../wasm-build/browser_bg.wasm',import.meta.url))});
const wasm={conversion_capabilities,open_ifccad,convert_cad_to_ifccad,export_ifccad,export_ifccad_binary,export_ifccad_with_options,convert_cad_to_ifccad_with_options};
const capabilities=JSON.parse(conversion_capabilities()).ifccad;
assert.equal(capabilities.backend,'opencaddocument');
const pins=JSON.parse(await readFile(new URL('../ocs-source.json',import.meta.url)));
assert.equal(capabilities.revision,pins.explorerDocumentCommit,'built processor matches recorded Document revision');
assert.equal(pins.documentCommit,pins.explorerDocumentCommit,'Studio and Explorer use the same IFCX mapping');
assert.equal(pins.codecCommit,pins.explorerCodecCommit,'Studio and Explorer use the same CAD model');
assert.equal(capabilities.profileVersion,pins.profile);
assert.equal(capabilities.cborMappingRevision,pins.cborRevision);
assert.equal(capabilities.developmentPatch,pins.localDevelopment);
for(const key of ['partialOpening','splinePreservation','adjustableTolerance','coordinateToleranceDefault','coordinateToleranceFallback'])assert.equal(capabilities[key],false,key);
assert.equal(capabilities.numericalAssessment,'unassessed');
const bytesOf=file=>Uint8Array.from(Buffer.from(file.base64,'base64'));
const request=(name,bytes,kind='ifccad')=>({kind,name,files:[{path:name,bytes:Uint8Array.from(bytes).buffer}]});
function checked(source){const value=processBrowserRequest(source,wasm);assert.equal(value.failure,null,JSON.stringify(value.failure));assert.equal(value.validation.strictAvailable,true);return value;}
function catalogueAutomaticText(result){
 const dimensions=result.presentation.graph.data.filter(n=>n.attributes?.['ifccad::dimension']);
 assert.equal(dimensions.length,9);
 for(const node of dimensions){
  const a=node.attributes;
  assert.equal(a['ifccad::dimension'].textPlacement?.kind??'styleDriven','styleDriven');
  assert.equal(a['ifccad::annotation'].contexts.length,2);
  assert.ok(a['ifccad::annotation'].contexts.every(c=>c.presentation==null),'membership-only contexts must not acquire artificial text positions');
 }
}
function catalogueLeaders(result){
 const leaders=result.presentation.leaders;
 assert.deepEqual(leaders.map(v=>v.kind).sort(),['Leader','Leader','MultiLeader','Tolerance']);
 const targets=leaders.filter(v=>v.kind==='Leader').map(v=>v.definition.annotationEntity);
 assert.ok(targets[0]);assert.equal(targets[0],targets[1],'classic leaders retain one shared ordinary target');
 const multi=leaders.find(v=>v.kind==='MultiLeader');
 assert.ok(multi.ownedNodes.some(v=>v.attributes?.['ifccad::multiLeaderLine']));
 assert.equal(multi.graphics.commandCount,5);
 const selection=result.preview.viewerSelection;
 const owner=selection.entities.find(v=>v.path===multi.path);
 assert.ok(owner);
 assert.ok(selection.ownedPartOwners.some(v=>v.ownerPath===multi.path&&v.handle===owner.handle));
}

for(const example of examples){
 const name=example.path.split('/').at(-1),bytes=await readFile(new URL('../../'+example.path,import.meta.url)),source=request(name,bytes);
 const opened=checked(source);
 if(example.name==='Entity catalogue'){catalogueAutomaticText(opened);catalogueLeaders(opened);}
 assert.deepEqual(Buffer.from(opened.nativeFile.base64,'base64'),bytes,'preview retains actual source bytes');
 assert.ok(opened.nativeFile.fileName.endsWith('.ifcx'));
 assert.equal(opened.preview.rendering,'unassessed');
 assert.equal(opened.preview.viewerSelection.qualification,'pinnedStudioIfcxImport');
 assert.ok(opened.preview.viewerSelection.entities.length);
 const model=createInspection(opened,opened.nativeSourceText);
 assert.ok(model.valid);assert.ok(model.nodes.size>0);
 const labels={
  'House floor plan':['10000','6100','3900','12000'],
  'Entity catalogue':['18','21.63','15','30','90','90','9','23.56','15'],
  'Block attributes':[],
  'Native splines':[],
 }[example.name].sort();
 assert.deepEqual(opened.presentation.dimensions.map(d=>d.geometry.label.primary).sort(),labels);
 if(example.name==='Block attributes'){
  const attributes=opened.presentation.blockAttributes;
  assert.equal(attributes.definitions.length,9);
  assert.equal(attributes.values.length,32);
  assert.equal(attributes.evaluation,'notPerformed');
  // The eight values in the nested block definition have no independent
  // top-level host selection. Directly owned instance children do.
  const selection=opened.preview.viewerSelection;
  assert.equal(selection.attributeOwners.length,24);
  for(const child of selection.attributeOwners){
   const parent=selection.entities.find(row=>row.path===child.ownerPath);
   assert.equal(child.handle,parent.handle);
   assert.ok(attributes.values.some(row=>row.path===child.path&&row.ownerPath===child.ownerPath));
  }
  assert.ok(attributes.values.some(row=>row.value.mText));
  assert.ok(attributes.values.some(row=>row.value.fields));
 }
 if(example.name==='Entity catalogue')for(const type of ['point','lineSegment','circle','arc','ellipse','ellipseArc','planarPolyline','spatialPolyline','text','mText','hatch','dimension','blockInstance','viewport','solid','trace'])assert.ok([...model.nodes.values()].some(node=>node.type===type),type);
 if(example.name==='Native splines'){
  assert.equal(opened.presentation.graph.data.filter(node=>node.attributes?.['ifccad::geom::spline']).length,4);
  const strict={...source,exportConversionOptions:{splineLossPolicy:'reject'}};
  const blocked=processBrowserRequest(nativeExportRequest(strict,opened,{format:'dwg',version:'AC1032'}),wasm);
  assert.equal(Boolean(blocked.export?.download),false);
  assert.ok(blocked.conversion.diagnostics.some(row=>row.message.includes('omits the fit data')));
 }
 for(const encoding of ['cbor','cbor-zstd']){
  const saved=checked(nativeExportRequest(source,opened,{format:'ifccad',encoding})).export.download;
  const binary=request(saved.fileName,bytesOf(saved)),decoded=checked(binary);
  if(example.name==='Entity catalogue')catalogueAutomaticText(decoded);
  assert.equal(decoded.source.representation,'decoded');assert.equal(decoded.nativeFile.representation,'decoded');
  assert.equal(decoded.presentation.entities.length,opened.presentation.entities.length);
  assert.deepEqual(decoded.presentation.polylineWidths,opened.presentation.polylineWidths);
  assert.deepEqual(decoded.presentation.extrusions,opened.presentation.extrusions);
  assert.deepEqual(decoded.presentation.dimensions,opened.presentation.dimensions);
  assert.deepEqual(decoded.presentation.blockAttributes,opened.presentation.blockAttributes);
  assert.deepEqual(decoded.presentation.leaders,opened.presentation.leaders);
  const same=checked(nativeExportRequest(binary,decoded,{format:'ifccad',encoding})).export.download;
  assert.deepEqual(bytesOf(same),bytesOf(saved),'same binary encoding retains validated source bytes');
  const projected=checked(nativeExportRequest(binary,decoded,{format:'ifccad'})).export.download;
  checked(request(projected.fileName,bytesOf(projected)));
 }
 for(const format of ['dxf','dwg']){
  const exported=checked(nativeExportRequest(source,opened,{format,version:'AC1032'}));
  assert.equal(exported.export.fileCheck.cadReadback,true);assert.equal(exported.export.fileCheck.versionMatched,true);
  assert.equal(exported.export.fileCheck.strictNativeReadback,true);
  assert.equal(exported.export.fileCheck.completeSemanticFidelity,'unassessed');
  assert.equal(exported.conversion.geometryAssessment.status,'unassessed');
  const file=exported.export.download,returned=checked(request('returned.'+format,bytesOf(file),'cad'));
  if(example.name==='Entity catalogue'){
   catalogueAutomaticText(returned);
   catalogueLeaders(returned);
   const expected=opened.presentation.extrusions.filter(row=>row.thickness===15);
   const returnedModel=createInspection(returned),remaining=returned.presentation.extrusions.filter(row=>row.thickness!==0);
   assert.equal(expected.length,7,'the five existing primitive samples include both arcs and both planar polylines');
   assert.equal(remaining.length,expected.length,'all catalogue samples retain thickness 15 after physical CAD readback');
   for(const sweep of expected){
    const type=model.nodes.get(sweep.path).type;
    const index=remaining.findIndex(row=>returnedModel.nodes.get(row.path).type===type&&row.thickness===sweep.thickness&&row.direction.every((v,i)=>Math.abs(v-sweep.direction[i])<1e-9));
    assert.notEqual(index,-1,'missing signed sweep '+type+' '+sweep.thickness);remaining.splice(index,1);
   }
  }
  if(example.name==='Native splines'){
   assert.equal(returned.presentation.graph.data.filter(node=>node.attributes?.['ifccad::geom::spline']).length,4);
   assert.equal(exported.conversion.diagnostics.some(row=>row.message.includes('omits the fit data')),format==='dwg');
  }
  assert.equal(returned.conversion.backend,'opencaddocument');
  assert.ok(returned.conversion.diagnostics.some(row=>row.code==='DOCUMENT_MIGRATION_LIMITS'));
  assert.ok(returned.nativeFile.fileName.endsWith('.ifcx'));
  checked(request(returned.nativeFile.fileName,bytesOf(returned.nativeFile)));
 }
 console.log('Native source, binary codecs, explicit preview mapping and real CAD readback: '+name);
}

// Existing native inspection remains broader than the host projection.
for(const fixture of ['hello-cad.ifcx','hello-geometry.ifcx','hello-hatch-solid.ifcx','hello-hatch-pattern.ifcx','hello-hatch-gradient.ifcx','hello-line-patterns.ifcx','workspace-state.ifcx']){
 const bytes=await readFile(new URL('../../tests/fixtures/ifccad/'+fixture,import.meta.url));
 const source=request(fixture,bytes),opened=checked(source),saved=checked(nativeExportRequest(source,opened,{format:'ifccad'}));
 assert.deepEqual(Buffer.from(saved.export.download.base64,'base64'),bytes);
 assert.equal(opened.presentation.hatchExportPreflight,undefined,'no reference-codec preflight is attributed to the new route');
 if(fixture==='workspace-state.ifcx')assert.equal(opened.presentation.ucsDefinitions[0].path,'/cad/d1/ucs/9007199254740993');
}

{
 const bytes=await readFile(new URL('../../tests/fixtures/ifccad/dimension-kinds-gallery.ifcx',import.meta.url)),source=request('dimension-kinds.ifcx',bytes),opened=checked(source);
 const dimensions=opened.presentation.dimensions;
 assert.equal(dimensions.length,9);
 assert.equal(new Set(dimensions.map(d=>d.geometry.definition.kind)).size,9);
 assert.ok(dimensions.every(d=>d.geometry.graphics.state==='stored'));
 assert.equal(opened.preview.viewerSelection.entities.length,9);
 assert.deepEqual(Buffer.from(bytesOf(opened.nativeFile)),bytes,'all-nine host preview receives the reviewed native snapshot');
 for(const encoding of ['cbor','cbor-zstd']){
  const file=checked(nativeExportRequest(source,opened,{format:'ifccad',encoding})).export.download;
  assert.deepEqual(checked(request(file.fileName,bytesOf(file))).presentation.dimensions,dimensions);
 }
 for(const format of ['dxf','dwg']){
  const file=checked(nativeExportRequest(source,opened,{format,version:'AC1032'})).export.download;
  const returned=checked(request(file.fileName,bytesOf(file),'cad'));
  assert.equal(returned.presentation.dimensions.length,9);
  assert.equal(new Set(returned.presentation.dimensions.map(d=>d.geometry.definition.kind)).size,9);
 }
 const old=new TextDecoder().decode(bytes).replaceAll(pins.profile,'0.3.0');
 assert.ok(processBrowserRequest(request('old-profile.ifcx',new TextEncoder().encode(old)),wasm).failure,'old alpha profile must not silently use the new reader');
 console.log('All nine Dimension kinds: actual native preview snapshot, owner bindings, stored graphics, CBOR/Zstd and physical CAD downloads.');
}

const large=(await readFile(new URL('../../tests/fixtures/ifccad/hello-line-patterns.ifcx',import.meta.url),'utf8')).replace(/("next(?:Entity|Layer|Layout|Block|LinePattern|Ucs|ModelWindow)Id"\s*:\s*)\d+/g,(_,prefix)=>prefix+'9007199254740993');
const source=request('large.ifcx',new TextEncoder().encode(large)),opened=checked(source);
assert.equal(new TextDecoder().decode(bytesOf(opened.nativeFile)),large);
assert.equal(new TextDecoder().decode(bytesOf(checked(nativeExportRequest(source,opened,{format:'ifccad'})).export.download)),large);

for(const options of [{tolerance:{mode:'exact'}},{tolerance:{mode:'custom',value:1e-9,unit:'drawing'}},{partialOpen:true},{preserveSplines:true},{lossPolicy:'reject'}]){
 const value=JSON.parse(convert_cad_to_ifccad_with_options('unsupported.dxf','dxf',new Uint8Array(),'2026-10-27T10:00:00Z',JSON.stringify(options)));
 assert.equal(value.failure.code,'DOCUMENT_OPTION_UNAVAILABLE');assert.equal(value.export?.download,undefined);
}
const spline=await readFile(new URL('../../crates/viewer/tests/fixtures/open-cubic.dxf',import.meta.url));
const splineOpened=checked(request('spline.dxf',spline,'cad'));
assert.ok(!splineOpened.conversion.diagnostics.some(row=>row.code==='DOCUMENT_CONVERSION_LOSS'));
assert.ok(splineOpened.presentation.graph.data.some(node=>node.attributes?.['ifccad::geom::spline']));
assert.equal(splineOpened.presentation.opaqueEntityCount,0,'the spline is stored as native geometry');
assert.throws(()=>processBrowserRequest({...request('spline.dxf',spline,'cad'),preserveSplines:true},wasm),/preservation|unavailable/i);
console.log('Shared-route limits, exact native downloads and native inspection verified; numerical/visual fidelity remains unassessed.');

const resourceBytes=await readFile(new URL('../../conformance/next/ifccad/valid/font-symbol-source-requests.ifcx',import.meta.url));
const resources=checked(request('font-symbol-sources.ifcx',resourceBytes));
const resourceRows=resources.presentation.fontSymbolRequests;
assert.ok(resourceRows.some(row=>row.requestPath.startsWith('/cad/d1/symbolSource/9007199254740993#')&&row.status==='notChecked'));
assert.ok(resourceRows.some(row=>row.requestedName==='..\\project\\missing.shx'));
const resourceModel=createInspection(resources);
assert.equal(resourceModel.fontSymbolReport.snapshot,resources);
assert.equal(resourceModel.fontSymbolReport.entries.length,resourceRows.length);
console.log('Exact source requests, unchecked application report and full-width resource paths survive the browser open route.');

const contextGraph=JSON.parse(await readFile(new URL('../../conformance/next/ifccad/valid/annotation-dimension-linear-aligned.ifcx',import.meta.url),'utf8'));
const richGraph=JSON.parse(await readFile(new URL('../../conformance/next/ifccad/valid/text-rich.ifcx',import.meta.url),'utf8'));
const richText=richGraph.data.find(n=>n.attributes?.['ifccad::mText']).attributes;
const inlineText=structuredClone(richText['ifccad::mText']);
inlineText.style='/cad/d1/textStyle/0';
inlineText.characterFormat={font:{family:'Saved context family'}};
const contextDimension=contextGraph.data.find(n=>n.attributes?.['ifccad::annotation']);
contextDimension.attributes['ifccad::annotation'].contexts[0].presentation.graphics.parts=[{
 'ifccad::dimensionGraphicsPart':{},'ifccad::mText':inlineText,'ifccad::geom::placement':richText['ifccad::geom::placement']}];
const contextOpened=checked(request('saved-context-font.ifcx',new TextEncoder().encode(JSON.stringify(contextGraph))));
assert.ok(contextOpened.presentation.fontSymbolRequests.some(row=>row.requestedName==='Saved context family'&&row.requestPath.includes('/presentation/graphics/parts/0/ifccad::mText/')));
console.log('Saved Dimension context MText requests are included in browser inspection.');
