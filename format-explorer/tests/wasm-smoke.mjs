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
for(const key of ['partialOpening','splinePreservation','adjustableTolerance','coordinateToleranceDefault','coordinateToleranceFallback'])assert.equal(capabilities[key],false,key);
assert.equal(capabilities.numericalAssessment,'unassessed');
const bytesOf=file=>Uint8Array.from(Buffer.from(file.base64,'base64'));
const request=(name,bytes,kind='ifccad')=>({kind,name,files:[{path:name,bytes:Uint8Array.from(bytes).buffer}]});
function checked(source){const value=processBrowserRequest(source,wasm);assert.equal(value.failure,null,JSON.stringify(value.failure));assert.equal(value.validation.strictAvailable,true);return value;}

for(const example of examples){
 const name=example.path.split('/').at(-1),bytes=await readFile(new URL('../../'+example.path,import.meta.url)),source=request(name,bytes);
 const opened=checked(source);
 assert.deepEqual(Buffer.from(opened.nativeFile.base64,'base64'),bytes,'preview retains actual source bytes');
 assert.ok(opened.nativeFile.fileName.endsWith('.ifcx'));
 assert.equal(opened.preview.rendering,'unassessed');
 assert.equal(opened.preview.viewerSelection.qualification,'pinnedStudioIfcxImport');
 assert.ok(opened.preview.viewerSelection.entities.length);
 const model=createInspection(opened,opened.nativeSourceText);
 assert.ok(model.valid);assert.ok(model.nodes.size>0);
 const labels=(example.name==='House floor plan'?['10000','6100','3900','12000']:['18','21.63','15','30','90','90','9','23.56','15']).sort();
 assert.deepEqual(opened.presentation.dimensions.map(d=>d.geometry.label.primary).sort(),labels);
 if(example.name==='Entity catalogue')for(const type of ['point','lineSegment','circle','arc','ellipse','ellipseArc','planarPolyline','spatialPolyline','text','mText','hatch','dimension','blockInstance','viewport','solid','trace'])assert.ok([...model.nodes.values()].some(node=>node.type===type),type);
 for(const encoding of ['cbor','cbor-zstd']){
  const saved=checked(nativeExportRequest(source,opened,{format:'ifccad',encoding})).export.download;
  const binary=request(saved.fileName,bytesOf(saved)),decoded=checked(binary);
  assert.equal(decoded.source.representation,'decoded');assert.equal(decoded.nativeFile.representation,'decoded');
  assert.equal(decoded.presentation.entities.length,opened.presentation.entities.length);
  assert.deepEqual(decoded.presentation.polylineWidths,opened.presentation.polylineWidths);
  assert.deepEqual(decoded.presentation.dimensions,opened.presentation.dimensions);
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
 const old=new TextDecoder().decode(bytes).replaceAll('0.2.0','0.1.0');
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
assert.ok(splineOpened.conversion.diagnostics.some(row=>row.code==='DOCUMENT_CONVERSION_LOSS'));
assert.equal(splineOpened.presentation.opaqueEntityCount,0,'no opaque preservation is claimed');
assert.throws(()=>processBrowserRequest({...request('spline.dxf',spline,'cad'),preserveSplines:true},wasm),/preservation|unavailable/i);
console.log('Shared-route limits, exact native downloads and native inspection verified; numerical/visual fidelity remains unassessed.');
