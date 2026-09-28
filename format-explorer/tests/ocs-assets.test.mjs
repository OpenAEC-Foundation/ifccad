import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm,access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {request as httpRequest} from 'node:http';
import {build} from '../scripts/build.mjs';
import {productionServer} from '../scripts/production.mjs';

test('pinned Open CAD Studio bundle is copied and framed on the explorer origin',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-ocs-'));
 const source=path.join(root,'source'),dist=path.join(root,'dist');
 await mkdir(path.join(source,'fonts'),{recursive:true});
 await mkdir(path.join(source,'worker_pkg'),{recursive:true});
 for(const [name,body] of [['index.html','<html><body>Open CAD Studio</body></html>'],['app.js','viewer'],['app.wasm','wasm'],['fonts/viewer.woff2','font'],['LICENSE','GPL']])await writeFile(path.join(source,name),body);
 await writeFile(path.join(source,'worker_pkg/ocs_web_worker_bg.wasm'),'worker');
 await build({outputRoot:dist,ocsRoot:source});
 const html=await readFile(path.join(dist,'ocs/app/index.html'),'utf8');
 assert.equal((html.match(/ocs-bridge\.mjs/g)||[]).length,1);
 assert.equal(await readFile(path.join(dist,'ocs/app/fonts/viewer.woff2'),'utf8'),'font');
 assert.match(await readFile(path.join(dist,'ocs/app/SOURCE.json'),'utf8'),/0d023d267bc5b7afeca3b54e98875b0efd4f3926/);
 const service=productionServer({root:dist,publicOrigin:'https://ifccad-explorer.open-aec.com',api:{handle:async()=>false,manager:{close:async()=>{}}}});
 await new Promise(resolve=>service.server.listen(0,'127.0.0.1',resolve));
 const base='http://127.0.0.1:'+service.server.address().port;
 const request=url=>new Promise((resolve,reject)=>{
  const req=httpRequest(base+url,{headers:{Host:'ifccad-explorer.open-aec.com'}},res=>{res.resume();res.on('end',()=>resolve(res));});req.on('error',reject);req.end();
 });
 try{
  const wasm=await request('/ocs/app/app.wasm');assert.equal(wasm.statusCode,200);assert.equal(wasm.headers['content-type'],'application/wasm');assert.equal(wasm.headers['x-frame-options'],'SAMEORIGIN');
  assert.match((await request('/ocs/app/app.js')).headers['content-type'],/javascript/);
  assert.equal((await request('/ocs/app/fonts/viewer.woff2')).headers['content-type'],'font/woff2');
  assert.equal((await request('/ocs/app/worker_pkg/ocs_web_worker_bg.wasm')).headers['content-type'],'application/wasm');
  assert.equal((await request('/')).headers['x-frame-options'],'DENY');
  assert.equal((await request('/ocs/app/%2e%2e%2f%2e%2e%2fsecrets.txt')).statusCode,404);
 }finally{await service.close();await rm(root,{recursive:true,force:true});}
});

test('a build without viewer assets removes an obsolete copied viewer',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-ocs-stale-')),source=path.join(root,'source'),dist=path.join(root,'dist');
 await mkdir(source);await writeFile(path.join(source,'index.html'),'<body></body>');
 try{
  await build({outputRoot:dist,ocsRoot:source});
  await build({outputRoot:dist,ocsRoot:path.join(root,'absent')});
  await assert.rejects(access(path.join(dist,'ocs/app/index.html')),{code:'ENOENT'});
 }finally{await rm(root,{recursive:true,force:true});}
});
