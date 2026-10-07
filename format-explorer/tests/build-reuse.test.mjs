import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm,stat,utimes,access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {build} from '../scripts/build.mjs';

test('unchanged viewer and converter assets retain their files while changed and removed assets are synchronized',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-reuse-')),ocs=path.join(root,'ocs'),wasm=path.join(root,'wasm'),dist=path.join(root,'dist');
 try{
  await mkdir(ocs);await mkdir(wasm);
  for(const [name,body]of [['index.html','<body>viewer</body>'],['app.wasm','viewer-one'],['old.js','obsolete']])await writeFile(path.join(ocs,name),body);
  await writeFile(path.join(wasm,'browser_bg.wasm'),'runtime-one');await writeFile(path.join(wasm,'browser.js'),'module-one');
  await build({outputRoot:dist,ocsRoot:ocs,wasmRoot:wasm});
  const checked=['ocs/app/app.wasm','ocs/app/index.html','ocs/app/ocs-bridge.mjs','wasm/browser_bg.wasm'].map(name=>path.join(dist,name)),stamp=new Date('2001-01-01T00:00:00Z');
  for(const file of checked)await utimes(file,stamp,stamp);
  await build({outputRoot:dist,ocsRoot:ocs,wasmRoot:wasm});
  for(const file of checked)assert.equal((await stat(file)).mtimeMs,stamp.getTime(),file);
  await writeFile(path.join(wasm,'browser_bg.wasm'),'runtime-two');await writeFile(path.join(ocs,'app.wasm'),'viewer-two');await rm(path.join(ocs,'old.js'));
  await build({outputRoot:dist,ocsRoot:ocs,wasmRoot:wasm});
  assert.equal(await readFile(path.join(dist,'wasm/browser_bg.wasm'),'utf8'),'runtime-two');assert.equal(await readFile(path.join(dist,'ocs/app/app.wasm'),'utf8'),'viewer-two');
  assert.equal((await stat(path.join(dist,'ocs/app/index.html'))).mtimeMs,stamp.getTime());assert.equal((await stat(path.join(dist,'ocs/app/ocs-bridge.mjs'))).mtimeMs,stamp.getTime());
  assert.equal((await readFile(path.join(dist,'ocs/app/index.html'),'utf8')).match(/ocs-bridge\.mjs/g).length,1);
  await assert.rejects(access(path.join(dist,'ocs/app/old.js')),{code:'ENOENT'});
  await build({outputRoot:dist,ocsRoot:path.join(root,'missing'),wasmRoot:path.join(root,'missing')});
  await assert.rejects(access(path.join(dist,'ocs/app/index.html')),{code:'ENOENT'});await assert.rejects(access(path.join(dist,'wasm/browser_bg.wasm')),{code:'ENOENT'});
 }finally{await rm(root,{recursive:true,force:true});}
});
test('Rust check cache includes the repository examples instead of the retired explorer examples folder',async()=>{
 const workflow=await readFile(new URL('../../.github/workflows/format-explorer.yml',import.meta.url),'utf8');
 const key=workflow.split(/\r?\n/).find(line=>line.includes('key: ocdraw-rust-checks-'));assert.match(key,/'examples\/\*\*'/);assert.doesNotMatch(key,/format-explorer\/examples/);
});
