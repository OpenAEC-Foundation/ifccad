import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {readExamples,exampleRoot} from '../scripts/fixtures.mjs';
import {validateUpload} from '../scripts/upload-paths.mjs';
import {drawingChoices,downloadName} from '../src/export-files.mjs';
import {createJobManager} from '../scripts/jobs.mjs';

test('export preserves exact example files including checksummed resources',async()=>{
 for(const example of await readExamples()){
  assert.ok(example.exportFiles?.length);
  for(const file of example.exportFiles){
   const original=await readFile(new URL(file.path,exampleRoot(example.name)));
   assert.deepEqual(Buffer.from(file.base64,'base64'),original);
  }
 }
});
test('drawing selection uses IFCX identity and downloads have safe format-specific names',()=>{
 assert.deepEqual(drawingChoices({ifcx:{data:[{path:'draw-1',type:'openaec:Drawing',attributes:{name:'Model'}},{path:'future',type:'concept:Drawing'}]}}),[{id:'draw-1',label:'Model'}]);
 assert.equal(downloadName('plan.dwg','dxf'),'plan-ifccad.dxf');
 assert.equal(downloadName('a/b\\c.dxf','dwg'),'a_b_c-ifccad.dwg');
 assert.equal(downloadName('plan.dwg','ifccad'),'plan-ifccad.zip');
});
test('job manager forwards export selection and cleans source staging before returning a download',async()=>{
 let options;
 const manager=createJobManager({worker:async value=>{options=value;return {source:{},export:{download:{base64:'YWJj'}}};}});
 try{
  const {id}=manager.create({kind:'cad',name:'source.dwg',files:[{path:'source.dwg',base64:'YWJj'}],export:{format:'dxf',drawing:'drawing-0'}});
  while(manager.get(id).status==='running')await new Promise(r=>setTimeout(r,5));
  assert.deepEqual(options.export,{format:'dxf',drawing:'drawing-0'});
  assert.equal(manager.get(id).result.export.download.base64,'YWJj');
  await assert.rejects(readFile(options.input),{code:'ENOENT'});
  manager.cancel(id);assert.equal(manager.get(id),null);
 }finally{await manager.close();}
});
test('export options survive validation and reject unexpected format or drawing',()=>{
 const request={kind:'package',files:[{path:'package.ifcx.json',base64:'e30='}],export:{format:'dwg',drawing:'drawing-main'}};
 assert.deepEqual(validateUpload(request).export,request.export);
 assert.equal(validateUpload({...request,export:{format:'ifccad'}}).export.format,'ifccad');
 for(const value of [{format:'exe',drawing:'x'},{format:'dxf',drawing:''},{format:'dxf',drawing:12}])assert.throws(()=>validateUpload({...request,export:value}));
});
