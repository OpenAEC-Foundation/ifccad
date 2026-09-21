import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {request as httpRequest} from 'node:http';
import {createJobManager,createJobHandler} from '../scripts/jobs.mjs';
import {productionServer} from '../scripts/production.mjs';
import {validateUpload} from '../scripts/upload-paths.mjs';

const origin='https://ifccad-explorer.open-aec.com';
const upload={kind:'package',name:'demo',files:[{path:'package.ifcx.json',base64:'e30='}]};
test('public service confines files and authenticates each job independently',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-public-test-'));
 await writeFile(path.join(root,'index.html'),'<title>Explorer</title>');
 const manager=createJobManager({worker:async()=>({source:{name:'demo'}})});
 const api=createJobHandler({manager,available:async()=>true,publicOrigin:origin});
 const service=productionServer({root,api,publicOrigin:origin});
 await new Promise(resolve=>service.server.listen(0,'127.0.0.1',resolve));
 const base='http://127.0.0.1:'+service.server.address().port;
 const request=(url,options={})=>new Promise((resolve,reject)=>{
  const req=httpRequest(base+url,{...options,headers:{Host:new URL(origin).host,...options.headers}},res=>{
   const chunks=[];res.on('data',chunk=>chunks.push(chunk));res.on('end',()=>{const text=Buffer.concat(chunks).toString();resolve({status:res.statusCode,text:async()=>text,json:async()=>JSON.parse(text)});});
  });req.on('error',reject);req.end(options.body);
 });
 const create=()=>request('/api/jobs',{method:'POST',headers:{Origin:origin,'Content-Type':'application/json'},body:JSON.stringify(upload)});
 try{
  assert.equal((await fetch(base+'/')).status,403);
  const page=await request('/');assert.equal(page.status,200);assert.equal(await page.text(),'<title>Explorer</title>');
  assert.equal((await request('/%2e%2e%2foutside.json')).status,404);
  assert.equal((await request('/scripts/jobs.mjs')).status,404);
  assert.equal((await request('/api/jobs',{method:'POST',headers:{Origin:'https://other.example'}})).status,403);
  assert.equal((await request('/api/jobs',{method:'POST'})).status,403);
  assert.equal((await request('/api/jobs',{method:'POST',headers:{Origin:origin,'Content-Type':'text/plain'}})).status,415);
  assert.equal((await (await request('/api/capabilities')).json()).processing,'server');
  const created=await create();assert.equal(created.status,202);const first=await created.json();
  assert.match(first.token,/^[a-f0-9]{64}$/);
  const jobUrl='/api/jobs/'+first.id;
  assert.equal((await request(jobUrl)).status,403);
  assert.equal((await request(jobUrl,{headers:{Authorization:'Bearer '+'0'.repeat(64)}})).status,403);
  let result;
  for(let i=0;i<100;i++){result=await(await request(jobUrl,{headers:{Authorization:'Bearer '+first.token}})).json();if(result.status==='complete')break;await new Promise(r=>setTimeout(r,5));}
  assert.equal(result.status,'complete');
  const second=await(await create()).json();
  assert.equal((await request(jobUrl,{headers:{Authorization:'Bearer '+second.token}})).status,403);
  assert.equal((await request(jobUrl,{method:'DELETE',headers:{Origin:origin,Authorization:'Bearer '+first.token}})).status,200);
  assert.equal((await request(jobUrl,{headers:{Authorization:'Bearer '+first.token}})).status,403);
 }finally{await service.close();service.server.closeAllConnections();await rm(root,{recursive:true,force:true});}
});

test('retained result capacity is bounded and cancellation releases capacity',async()=>{
 const manager=createJobManager({maxRetainedJobs:1,worker:async()=>({source:{name:'demo'}})});
 try{
  const {id}=manager.create(upload);
  while(manager.get(id).status==='running')await new Promise(r=>setTimeout(r,5));
  assert.equal(manager.busy,true);assert.throws(()=>manager.create(upload),/Service busy/);
  manager.cancel(id);assert.equal(manager.busy,false);assert.doesNotThrow(()=>manager.create(upload));
 }finally{await manager.close();}
});

test('multi-megabyte uploads decode without regular expression stack exhaustion',()=>{
 const data=Buffer.alloc(4*1024*1024,42);
 const validated=validateUpload({kind:'cad',files:[{path:'example.dxf',base64:data.toString('base64')}]});
 assert.deepEqual(validated.files[0].bytes,data);
 for(const base64 of ['a===','====','ab=c','abcd='])assert.throws(()=>validateUpload({kind:'cad',files:[{path:'example.dxf',base64}]}));
});
