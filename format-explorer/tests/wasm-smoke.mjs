import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {initSync,open_cad,open_package,export_package} from '../wasm-build/ocdraw_browser.js';
import {processBrowserRequest} from '../src/browser-worker.mjs';
import {decodeBundle} from '../src/bundle.mjs';
import {buildModel} from '../src/model.mjs';

initSync({module:await readFile(new URL('../wasm-build/ocdraw_browser_bg.wasm',import.meta.url))});
const paths=['package.ifcx.json','resources/drawing.ifcdr.json'];
const contents=await Promise.all(paths.map(path=>readFile(new URL(`../examples/blocks-demo/${path}`,import.meta.url))));
const packageResult=JSON.parse(open_package('blocks-demo',paths,contents));
assert.equal(packageResult.validation.strictAvailable,true);
assert.equal(packageResult.failure,null);

const exported=JSON.parse(export_package('blocks-demo',paths,contents,'drawing-0','dxf','AC1032'));
assert.equal(exported.failure,null);
assert.equal(exported.export.fileCheck.readable,true);
const dxf=Buffer.from(exported.export.download.base64,'base64');
const reopened=JSON.parse(open_cad('roundtrip.dxf','dxf',dxf,'2026-09-25T12:00:00Z'));
assert.equal(reopened.validation.strictAvailable,true);
assert.equal(reopened.failure,null);
const imported=decodeBundle(reopened.presentation);
assert.equal(imported.ifcx.header.ifccadSchemaVersion,'0.14.0');
assert.ok(Object.values(imported.files).some(body=>body?.header?.version==='0.12.0'));
console.log('Browser WASM package, DXF export, and DXF import verified');

for(const name of ['workspace-model','workspace-paper']){
 const root=new URL(`../../conformance/next/packages/valid/${name}/`,import.meta.url);
 const ifcx=JSON.parse(await readFile(new URL('package.ifcx.json',root),'utf8'));
 const drawing=ifcx.data.find(node=>node.type==='openaec:Drawing');
 const uri=ifcx.data.find(node=>node.type==='openaec:DrawingRepresentation').attributes.resource.uri;
 const files=['package.ifcx.json',uri],bytes=await Promise.all(files.map(file=>readFile(new URL(file,root))));
 const opened=JSON.parse(open_package(name,files,bytes));
 assert.equal(opened.failure,null,`${name}: ${JSON.stringify(opened.failure)}`);
 assert.equal(opened.validation.strictAvailable,true,name);
 assert.ok(opened.presentation?.documents?.length>=2,name);
 const selected=files.map((path,index)=>({path,bytes:Uint8Array.from(bytes[index]).buffer}));
 const browserOpened=processBrowserRequest({kind:'package',name,files:selected},{open_package,export_package});
 assert.equal(browserOpened.validation.strictAvailable,true,name);
 const displayed=buildModel(decodeBundle(browserOpened.presentation));
 assert.equal(displayed.missing.length,0,name);
 assert.ok(displayed.nodes.some(node=>node.kind==='workspace-group'),name);
 const browserArchive=processBrowserRequest({kind:'package',name,files:selected,export:{format:'ifccad'}},{open_package,export_package});
 assert.equal(browserArchive.export?.fileCount,2,name);
 assert.ok(browserArchive.export?.download?.byteLength>0,name);
 const packageExport=JSON.parse(export_package(name,files,bytes,'','ifccad','AC1032'));
 assert.equal(packageExport.failure,null,name);
 assert.equal(packageExport.export.packageReady,true,name);
 for(const format of ['dxf','dwg']){
  const converted=JSON.parse(export_package(name,files,bytes,drawing.path,format,'AC1032'));
  assert.equal(converted.failure,null,`${name} ${format}: ${JSON.stringify(converted.failure)}`);
  assert.equal(converted.export?.fileCheck?.readable,true,`${name} ${format}`);
  assert.equal(converted.export?.download?.format,format,`${name} ${format}`);
 }
}
console.log('IFCDR 0.12 workspace packages load and export to DXF, DWG and IFCCAD');
