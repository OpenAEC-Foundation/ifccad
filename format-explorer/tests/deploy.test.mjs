import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,readlink,symlink,rm} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

for(const previous of [false,true])for(const fail of [false,true])test(`deployment ${fail?'rolls back':'activates'} with ${previous?'managed':'original'} release`,{skip:process.platform==='win32'},async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'ifccad-deploy-test-'));
  const revision='a'.repeat(40),id=revision+'-1-1';
  try {
    await mkdir(path.join(root,'bin'));await mkdir(path.join(root,'incoming'));await mkdir(path.join(root,'payload/dist'),{recursive:true});
    await writeFile(path.join(root,'compose.yml'),'services: {}\n');
    await writeFile(path.join(root,'payload/dist/version.json'),JSON.stringify({revision}));
    await writeFile(path.join(root,'payload/container.mjs'),'');
    await writeFile(path.join(root,'payload/ifccad-viewer'),'reader');
    await writeFile(path.join(root,'bin/docker'),`#!/bin/bash\nprintf '%s\\n' "$*" >> "$IFCCAD_DEPLOY_ROOT/calls"\nif [[ "$*" == *' exec '* || "$1" == exec ]]; then exit ${fail?1:0}; fi\n`,{mode:0o755});
    await writeFile(path.join(root,'bin/curl'),'#!/bin/bash\ncat "$IFCCAD_DEPLOY_ROOT/payload/dist/version.json"\n',{mode:0o755});
    if(previous) {
      await mkdir(path.join(root,'releases/old'),{recursive:true});
      await symlink(path.join(root,'releases/old'),path.join(root,'current'));
      await writeFile(path.join(root,'deployment.yml'),'# Managed by IFCCAD deployment\n');
    }
    assert.equal(spawnSync('tar',['-czf',path.join(root,'incoming',id+'.tar.gz'),'-C',path.join(root,'payload'),'.']).status,0);
    const hash=spawnSync('sha256sum',[path.join(root,'incoming',id+'.tar.gz')],{encoding:'utf8'}).stdout.split(' ')[0];
    const result=spawnSync('bash',[fileURLToPath(new URL('../deploy/release.sh',import.meta.url)),id,hash],{env:{...process.env,IFCCAD_DEPLOY_ROOT:root,PATH:path.join(root,'bin')+':'+process.env.PATH,IFCCAD_HEALTH_ATTEMPTS:'1'},encoding:'utf8'});
    assert.equal(result.status,fail?1:0,result.stdout+result.stderr);
    const calls=await readFile(path.join(root,'calls'),'utf8');
    if(!fail)assert.equal(await readlink(path.join(root,'current')),path.join(root,'releases',id));
    else if(previous)assert.equal(await readlink(path.join(root,'current')),path.join(root,'releases/old'));
    else await assert.rejects(readlink(path.join(root,'current')),{code:'ENOENT'});
    assert.equal((calls.match(/up -d --no-deps --force-recreate explorer/g)||[]).length,fail?2:1);
    if(fail&&!previous)await assert.rejects(readFile(path.join(root,'deployment.yml')),{code:'ENOENT'});
  } finally {await rm(root,{recursive:true,force:true});}
});
