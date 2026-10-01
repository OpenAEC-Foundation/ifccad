import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {request as httpRequest} from 'node:http';
import {productionServer} from '../scripts/production.mjs';

test('public website serves static files and exposes no processing API',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'ocdraw-static-test-'));
 await writeFile(path.join(root,'index.html'),'<title>Inspector</title>');
 const service=productionServer({root,publicOrigin:'https://ifccad-explorer.open-aec.com'});
 await new Promise(resolve=>service.server.listen(0,'127.0.0.1',resolve));
 const base='http://127.0.0.1:'+service.server.address().port;
 const request=(url,method='GET')=>new Promise((resolve,reject)=>{
  const req=httpRequest(base+url,{method,headers:{Host:'ifccad-explorer.open-aec.com'}},res=>{res.resume();res.on('end',()=>resolve(res.statusCode));});
  req.on('error',reject);req.end();
 });
 try{
  assert.equal((await fetch(base+'/')).status,403);
  assert.equal(await request('/'),200);
  assert.equal(await request('/%2e%2e%2foutside.json'),404);
  assert.equal(await request('/api/capabilities'),404);
  assert.equal(await request('/api/jobs'),404);
  assert.equal(await request('/api/jobs','POST'),405);
  assert.equal(await request('/scripts/jobs.mjs'),404);
 }finally{await service.close();service.server.closeAllConnections();await rm(root,{recursive:true,force:true});}
});
