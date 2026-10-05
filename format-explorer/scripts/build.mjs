import {mkdir,copyFile,readdir,access,rm,cp,readFile,writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
export const output=new URL('../dist/',import.meta.url);
export async function build({outputRoot=output,ocsRoot=new URL('../ocs-build/',import.meta.url)}={}){
 const destination=outputRoot instanceof URL?fileURLToPath(outputRoot):path.resolve(outputRoot);await mkdir(destination,{recursive:true});
 for(const name of await readdir(new URL('../src/',import.meta.url)))if(/\.(html|css|mjs|svg|ttf|txt)$/.test(name))await copyFile(new URL('../src/'+name,import.meta.url),path.join(destination,name));
 await mkdir(path.join(destination,'examples'),{recursive:true});
 await copyFile(new URL('../../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json',import.meta.url),path.join(destination,'examples','ordered-scopes.ocdraw.json'));
 await copyFile(new URL('../../examples/ifccad/hello-line-patterns.ifcx',import.meta.url),path.join(destination,'examples','hello-line-patterns.ifcx'));
 await rm(path.join(destination,'job-client.mjs'),{force:true});
 const wasmRoot=new URL('../wasm-build/',import.meta.url),wasmTarget=path.join(destination,'wasm');
 if(!wasmTarget.startsWith(path.resolve(destination)+path.sep))throw Error('Output escapes build directory');
 await rm(wasmTarget,{recursive:true,force:true});
 if(await access(new URL('browser_bg.wasm',wasmRoot)).then(()=>true,()=>false)){await mkdir(wasmTarget,{recursive:true});for(const name of ['browser.js','browser_bg.wasm'])await copyFile(new URL(name,wasmRoot),path.join(wasmTarget,name));await copyFile(new URL('../../LICENSE',import.meta.url),path.join(wasmTarget,'LICENSE'));}
 const bundle=ocsRoot instanceof URL?fileURLToPath(ocsRoot):path.resolve(ocsRoot),target=path.resolve(destination,'ocs','app');
 if(!target.startsWith(path.resolve(destination)+path.sep))throw Error('Viewer output escapes build directory');
 await rm(target,{recursive:true,force:true});
 if(await access(path.join(bundle,'index.html')).then(()=>true,()=>false)){
  await cp(bundle,target,{recursive:true,force:true});
  for(const name of ['ocs-bridge.mjs','ocs-messages.mjs'])await copyFile(new URL('../src/'+name,import.meta.url),path.join(target,name));
  const htmlFile=path.join(target,'index.html'),html=await readFile(htmlFile,'utf8');
  if(!html.includes('</body>'))throw Error('Open CAD Studio HTML has no body end');
  await writeFile(htmlFile,html.replace('</body>','<script type="module" src="./ocs-bridge.mjs"></script></body>'));
  await copyFile(new URL('../ocs-source.json',import.meta.url),path.join(target,'SOURCE.json'));
 }
 console.log('Built IFCCAD & OCDraw Explorer: '+destination);
}
if(process.argv[1]===fileURLToPath(import.meta.url))await build();
