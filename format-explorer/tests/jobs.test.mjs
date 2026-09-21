import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,readdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {createJobManager} from '../scripts/jobs.mjs';
import {limits} from '../scripts/upload-paths.mjs';
const request={kind:'package',name:'demo',files:[{path:'package.ifcx.json',base64:'e30='}]};
test('jobs clean staging and cancelled results cannot succeed',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'viewer-manager-test-'));
 const manager=createJobManager({tempRoot:root,worker:async({signal})=>{await new Promise(resolve=>signal.addEventListener('abort',resolve,{once:true}));return {source:{name:'late'}};}});
 const {id}=manager.create(request);assert.throws(()=>manager.create(request),/Another/);
 // Wait for staging to reach the fake worker.
 await new Promise(resolve=>setTimeout(resolve,30));manager.cancel(id);await manager.close();
 assert.deepEqual(await readdir(root),[]);await rm(root,{recursive:true});
});
test('size limit is enforced on actual decoded bytes',()=>{
 const manager=createJobManager({cap:{...limits,bytes:1}});assert.throws(()=>manager.create(request),/size limit/);return manager.close();
});

test('completed jobs release the active slot before their result is published',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'viewer-finish-test-'));const manager=createJobManager({tempRoot:root,worker:async()=>({source:{name:'x'}})});
 try{
  const {id}=manager.create(request);while(manager.get(id).status==='running')await new Promise(r=>setTimeout(r,5));
  assert.deepEqual(await readdir(root),[]);assert.doesNotThrow(()=>manager.create(request));
 }finally{await manager.close();await rm(root,{recursive:true});}
});

import {createServer} from 'node:http';
import {createJobHandler} from '../scripts/jobs.mjs';
import {runWorker,workerAvailable} from '../scripts/worker.mjs';
test('HTTP mutation rejects foreign origins before staging',async()=>{
 const api=createJobHandler({available:async()=>true});const server=createServer((req,res)=>api.handle(req,res));await new Promise(r=>server.listen(0,'127.0.0.1',r));const base='http://127.0.0.1:'+server.address().port;
 try{
  const response=await fetch(base+'/api/jobs',{method:'POST',headers:{Origin:'https://untrusted.example','Content-Type':'application/json'},body:JSON.stringify(request)});assert.equal(response.status,403);
  const caps=await(await fetch(base+'/api/capabilities')).json();assert.equal(caps.available,true);
 }finally{await api.manager.close();await new Promise(r=>server.close(r));}
});
test('worker timeout terminates the reader',async t=>{
 if(!await workerAvailable()){t.skip('Build the Rust companion to test process termination');return;}
 await assert.rejects(runWorker({kind:'package',input:'.',signal:new AbortController().signal,cap:{...limits,timeoutMs:1}}),/time limit/);
});

test('internal staging paths are removed from reports without changing source documents',async()=>{
 let inputPath;const manager=createJobManager({worker:async({input})=>{inputPath=input;return {source:{name:'x'},reader:{messages:[input]},presentation:{documents:[{text:input}]},failure:null};}});
 try{const {id}=manager.create(request);while(manager.get(id).status==='running')await new Promise(r=>setTimeout(r,5));const result=manager.get(id).result;assert.equal(result.reader.messages[0],'[temporary package]'+path.sep+'input');assert.equal(result.presentation.documents[0].text,inputPath);}finally{await manager.close();}
});
