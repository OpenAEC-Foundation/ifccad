import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readdir,access,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {examples} from '../src/examples.mjs';
import {build} from '../scripts/build.mjs';

test('four representative drawings per format match the authored example directories',async()=>{
 assert.equal(examples.length,4);
 for(const format of ['IFCCAD']){
  const selected=examples.filter(example=>example.format===format);
  assert.deepEqual(selected.map(example=>example.name),['Drawing overview','Text and multiline text','Hatches','Dimensions']);
  const files=(await readdir(new URL('../../examples/'+format.toLowerCase()+'/',import.meta.url))).filter(file=>/\.(ifcx|json)$/.test(file)).sort();
  assert.deepEqual(files,selected.map(example=>path.basename(example.path)).sort());
 }
});

test('site assembly removes retired example files while keeping the curated set',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-curated-'));
 try{
  await mkdir(path.join(root,'examples/ifccad'),{recursive:true});
  await writeFile(path.join(root,'examples/ifccad/retired.ifcx'),'obsolete');
  await writeFile(path.join(root,'examples/hello-line-patterns.ifcx'),'obsolete');
  await build({outputRoot:root,ocsRoot:path.join(root,'absent-ocs'),wasmRoot:path.join(root,'absent-wasm')});
  await assert.rejects(access(path.join(root,'examples/ifccad/retired.ifcx')),{code:'ENOENT'});
  await assert.rejects(access(path.join(root,'examples/hello-line-patterns.ifcx')),{code:'ENOENT'});
  for(const example of examples)await access(path.join(root,example.path));
 }finally{await rm(root,{recursive:true,force:true});}
});
