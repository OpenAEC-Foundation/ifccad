import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,readdir,access,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {examples} from '../src/examples.mjs';
import {build} from '../scripts/build.mjs';

test('the curated set includes the entity catalogue, house and XREF source formats',async()=>{
 assert.equal(examples.length,3);
 for(const format of ['IFCCAD']){
  const selected=examples.filter(example=>example.format===format);
  assert.deepEqual(selected.map(example=>example.name),['Entity catalogue','House floor plan','XREF source formats']);
  const files=(await readdir(new URL('../../examples/'+format.toLowerCase()+'/',import.meta.url))).filter(file=>/\.(ifcx|json)$/.test(file)).sort();
  for(const example of selected){
   assert.match(example.path,/\.ifccad$/);
   assert.ok(files.includes(path.basename(example.sourcePath)));
  }
 }
});

test('site assembly removes retired example files while keeping the curated set',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-curated-'));
 try{
  await mkdir(path.join(root,'examples/ifccad'),{recursive:true});
  await writeFile(path.join(root,'examples/ifccad/retired.ifcx'),'obsolete');
  await writeFile(path.join(root,'examples/ifccad/text.ifcx'),'old regression asset');
  await writeFile(path.join(root,'examples/ifccad/attributes.ifcx'),'focused regression drawing');
  await writeFile(path.join(root,'examples/ifccad/splines.ifcx'),'focused regression drawing');
  await writeFile(path.join(root,'examples/ifccad/entities.ifcx'),'old catalogue name');
  await writeFile(path.join(root,'examples/ifccad/house.ifcx'),'old house name');
  await writeFile(path.join(root,'examples/hello-line-patterns.ifcx'),'obsolete');
  await build({outputRoot:root,ocsRoot:path.join(root,'absent-ocs'),wasmRoot:path.join(root,'absent-wasm')});
  await assert.rejects(access(path.join(root,'examples/ifccad/retired.ifcx')),{code:'ENOENT'});
  await assert.rejects(access(path.join(root,'examples/ifccad/text.ifcx')),{code:'ENOENT'});
  await assert.rejects(access(path.join(root,'examples/ifccad/attributes.ifcx')),{code:'ENOENT'});
  await assert.rejects(access(path.join(root,'examples/ifccad/splines.ifcx')),{code:'ENOENT'});
  await assert.rejects(access(path.join(root,'examples/hello-line-patterns.ifcx')),{code:'ENOENT'});
  assert.deepEqual((await readdir(path.join(root,'examples/ifccad'))).sort(),['entities.ifccad','house.ifccad','xref-detail.ifccad','xref-source.dwg','xref-source.dxf','xref-source.ifccad','xrefs.ifccad']);
  for(const example of examples){
   assert.deepEqual(await readFile(path.join(root,example.path)),await readFile(new URL('../../'+example.sourcePath,import.meta.url)));
  }
 }finally{await rm(root,{recursive:true,force:true});}
});
