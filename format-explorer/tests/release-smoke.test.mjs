import test from 'node:test';
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {build} from '../scripts/build.mjs';
import {productionServer} from '../scripts/production.mjs';

test('public HTTPS smoke runs from a clean checkout without local dist',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-public-smoke-'));
 let service;
 try {
  await mkdir(path.join(root,'processor'));await mkdir(path.join(root,'checkout/deploy'),{recursive:true});await mkdir(path.join(root,'checkout/src'));
  await writeFile(path.join(root,'processor/browser.js'),'export function open_drawing(){} export function convert_cad_to_drawing(){} export function open_ifccad(){} export function convert_cad_to_ifccad(){} export function export_ifccad(){}');
  await writeFile(path.join(root,'processor/browser_bg.wasm'),'static MIME fixture');
  await build({outputRoot:path.join(root,'website'),wasmRoot:path.join(root,'processor'),ocsRoot:path.join(root,'missing-ocs')});
  await writeFile(path.join(root,'website/version.json'),JSON.stringify({revision:'public-smoke'}));
  await writeFile(path.join(root,'checkout/deploy/smoke.mjs'),await readFile(new URL('../deploy/smoke.mjs',import.meta.url)));
  await writeFile(path.join(root,'checkout/src/examples.mjs'),await readFile(new URL('../src/examples.mjs',import.meta.url)));
  service=productionServer({root:path.join(root,'website'),publicOrigin:'https://ifccad-explorer.open-aec.com'});
  await new Promise(resolve=>service.server.listen(0,'127.0.0.1',resolve));
  await promisify(execFile)(process.execPath,[path.join(root,'checkout/deploy/smoke.mjs'),'http://127.0.0.1:'+service.server.address().port,'public-smoke','public'],{cwd:path.join(root,'checkout'),timeout:30000});
 }finally {if(service){service.server.closeAllConnections();await service.close();}await rm(root,{recursive:true,force:true});}
});

for(const mode of ['static','basic'])test(`packaged release ${mode} smoke uses shipped examples without src`,async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'explorer-release-smoke-'));
 let service;
 try{
  await mkdir(path.join(root,'processor'));await mkdir(path.join(root,'deploy'));
  await writeFile(path.join(root,'processor/browser.js'),'export function open_drawing(){} export function convert_cad_to_drawing(){} export function open_ifccad(){} export function convert_cad_to_ifccad(){} export function export_ifccad(){}');
  await writeFile(path.join(root,'processor/browser_bg.wasm'),'static MIME fixture');
  await build({outputRoot:path.join(root,'dist'),wasmRoot:path.join(root,'processor'),ocsRoot:path.join(root,'missing-ocs')});
  await writeFile(path.join(root,'dist/version.json'),JSON.stringify({revision:'curated-smoke'}));
  await writeFile(path.join(root,'deploy/smoke.mjs'),await readFile(new URL('../deploy/smoke.mjs',import.meta.url)));
  service=productionServer({root:path.join(root,'dist'),publicOrigin:'https://ifccad-explorer.open-aec.com'});
  await new Promise(resolve=>service.server.listen(0,'127.0.0.1',resolve));
  await promisify(execFile)(process.execPath,[path.join(root,'deploy/smoke.mjs'),'http://127.0.0.1:'+service.server.address().port,'curated-smoke',mode],{timeout:30000});
 }finally{if(service){service.server.closeAllConnections();await service.close();}await rm(root,{recursive:true,force:true});}
});
