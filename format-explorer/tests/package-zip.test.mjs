import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {crc32,zipFiles,packageZip} from '../scripts/package-zip.mjs';
import {createJobManager} from '../scripts/jobs.mjs';

test('ZIP CRC and path/size boundaries',()=>{
 assert.equal(crc32(Buffer.from('123456789')),0xcbf43926);
 assert.equal(crc32(Buffer.alloc(0)),0);
 for(const file of ['../outside','/absolute','x\\y'])assert.throws(()=>zipFiles([{path:file,bytes:Buffer.from('a')}]));
 assert.throws(()=>zipFiles([{path:'x',bytes:Buffer.alloc(100)}],{files:1,bytes:100}),/limit/);
 assert.throws(()=>zipFiles([{path:'x',bytes:Buffer.alloc(0)},{path:'X',bytes:Buffer.alloc(0)}]),/Duplicate/);
});
test('standard Python ZIP reader verifies CRCs, Unicode names and exact payloads',t=>{
 const probe=spawnSync('python',['--version']);if(probe.error||probe.status!==0){t.skip('Python ZIP interoperability check requires Python');return;}
 const files=[{path:'package.ifcx.json',bytes:Buffer.from('{"data":[]}\r\n')},{path:'resources/één.ifcdr.json',bytes:Buffer.from('unchanged\n')},{path:'preservation/blob.bin',bytes:Buffer.from([0,255,3,128])}];
 const code='import sys,io,zipfile,json,base64\nz=zipfile.ZipFile(io.BytesIO(sys.stdin.buffer.read()))\nassert z.testzip() is None\nprint(json.dumps({n:base64.b64encode(z.read(n)).decode() for n in z.namelist()}))';
 const read=spawnSync('python',['-c',code],{input:zipFiles(files)});assert.equal(read.status,0,read.stderr.toString());
 assert.deepEqual(JSON.parse(read.stdout),Object.fromEntries(files.map(f=>[f.path,f.bytes.toString('base64')])));
});
test('package download contains nested blobs and honours cancellation',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-zip-'));
 try{
  await mkdir(path.join(root,'resources'));await writeFile(path.join(root,'package.ifcx.json'),'{}');await writeFile(path.join(root,'resources','blob.bin'),Buffer.from([1,0,255]));
  assert.equal((await packageZip(root)).fileCount,2);
  const controller=new AbortController();controller.abort();await assert.rejects(packageZip(root,{signal:controller.signal}),{name:'AbortError'});
 }finally{await rm(root,{recursive:true,force:true});}
});
test('only successful strict validation can authorize a package download',async()=>{
 for(const valid of [false,true]){
  const manager=createJobManager({worker:async()=>({source:{},failure:null,validation:{strictAvailable:valid},export:{format:'ifccad',packageReady:valid}})});
  try{
   const {id}=manager.create({kind:'package',files:[{path:'package.ifcx.json',base64:'e30='}],export:{format:'ifccad'}});
   while(manager.get(id).status==='running')await new Promise(r=>setTimeout(r,5));
   const result=manager.get(id).result;assert.equal(Boolean(result.export.download),valid);
   if(valid){assert.equal(result.export.fileCount,1);assert.equal(result.export.download.format,'ifccad');}
  }finally{await manager.close();}
 }
});
