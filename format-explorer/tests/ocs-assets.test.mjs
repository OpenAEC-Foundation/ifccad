import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm,access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {request as httpRequest} from 'node:http';
import {build} from '../scripts/build.mjs';
import {productionServer} from '../scripts/production.mjs';

test('viewer checkout, cache and attribution use the same fixed upstream commit',async()=>{
 const pin=JSON.parse(await readFile(new URL('../ocs-source.json',import.meta.url),'utf8'));
 const workflow=await readFile(new URL('../../.github/workflows/format-explorer.yml',import.meta.url),'utf8');
 assert.match(pin.commit,/^[0-9a-f]{40}$/);
 assert.equal(pin.repository,'https://github.com/HakanSeven12/OpenCADStudio');
 assert.ok(workflow.includes('ref: '+pin.commit));
 assert.ok(workflow.includes('key: ocs-web-'+pin.tag+'-'+pin.commit.slice(0,8)+'-linux-trunk'));
 const artifact='ocs-web-'+pin.tag+'-'+pin.commit.slice(0,8);
 assert.equal(workflow.split('name: '+artifact).length-1,2);
});

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
 assert.match(await readFile(path.join(dist,'ocs/app/ocs-selection.mjs'),'utf8'),/createSelectionObserver/);
 assert.equal(await readFile(path.join(dist,'ocs/app/fonts/viewer.woff2'),'utf8'),'font');
 assert.match(await readFile(path.join(dist,'ocs/app/SOURCE.json'),'utf8'),/2a2d9b55d9a328fb6e2ec4d4adb6b122aa189459/);
 const service=productionServer({root:dist,publicOrigin:'https://ifccad-explorer.open-aec.com'});
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
