import {mkdir,readdir,access,rm,readFile,writeFile,lstat} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {examples} from '../src/examples.mjs';
export const output=new URL('../dist/',import.meta.url);
const localPath=value=>value instanceof URL?fileURLToPath(value):path.resolve(value);
async function existing(file){try{return await lstat(file);}catch(error){if(error.code==='ENOENT')return null;throw error;}}
async function directory(folder){const old=await existing(folder);if(old?.isSymbolicLink())throw Error('Build destination is a symbolic link');if(old&&!old.isDirectory())await rm(folder);await mkdir(folder,{recursive:true});}
async function writeChanged(file,body){
 const bytes=Buffer.isBuffer(body)?body:Buffer.from(body),old=await existing(file);
 if(old?.isSymbolicLink())throw Error('Build destination is a symbolic link');
 if(old?.isFile()&&(await readFile(file)).equals(bytes))return;
 if(old?.isDirectory())await rm(file,{recursive:true,force:true});
 await directory(path.dirname(file));await writeFile(file,bytes);
}
async function copyChanged(source,target){await writeChanged(target,await readFile(source));}
/** Synchronize owned output files without rewriting unchanged bundle contents. */
async function bundle(source,target,overrides={}){
 await directory(target);const wanted=new Set(Object.keys(overrides));
 if(source)for(const entry of await readdir(source,{withFileTypes:true})){
  wanted.add(entry.name);if(Object.hasOwn(overrides,entry.name))continue;
  const from=path.join(source,entry.name),to=path.join(target,entry.name);
  if(entry.isDirectory())await bundle(from,to);
  else if(entry.isFile())await copyChanged(from,to);
  else throw Error('Viewer bundle contains an unsupported file type');
 }
 for(const [name,bytes]of Object.entries(overrides))await writeChanged(path.join(target,name),bytes);
 for(const entry of await readdir(target,{withFileTypes:true}))if(!wanted.has(entry.name))await rm(path.join(target,entry.name),{recursive:true,force:true});
}
export async function build({outputRoot=output,ocsRoot=new URL('../ocs-build/',import.meta.url),wasmRoot=new URL('../wasm-build/',import.meta.url)}={}){
 const destination=localPath(outputRoot);
 await directory(destination);
 for(const name of await readdir(new URL('../src/',import.meta.url)))if(/\.(html|css|mjs|svg|ttf|txt)$/.test(name))await copyChanged(new URL('../src/'+name,import.meta.url),path.join(destination,name));
 await mkdir(path.join(destination,'examples'),{recursive:true});
 await copyChanged(new URL('../../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json',import.meta.url),path.join(destination,'examples','ordered-scopes.ocdraw.json'));
 await copyChanged(new URL('../../examples/ifccad/hello-line-patterns.ifcx',import.meta.url),path.join(destination,'examples','hello-line-patterns.ifcx'));
 for(const example of examples){const target=path.resolve(destination,example.path);if(!target.startsWith(path.resolve(destination)+path.sep))throw Error('Example output escapes build directory');await mkdir(path.dirname(target),{recursive:true});await copyChanged(new URL('../../'+example.path,import.meta.url),target);}
 await rm(path.join(destination,'job-client.mjs'),{force:true});
 const wasmSource=localPath(wasmRoot),wasmTarget=path.join(destination,'wasm');
 if(!wasmTarget.startsWith(path.resolve(destination)+path.sep))throw Error('Output escapes build directory');
 if(await access(path.join(wasmSource,'browser_bg.wasm')).then(()=>true,()=>false))await bundle(null,wasmTarget,{
  'browser.js':await readFile(path.join(wasmSource,'browser.js')),
  'browser_bg.wasm':await readFile(path.join(wasmSource,'browser_bg.wasm')),
  LICENSE:await readFile(new URL('../../LICENSE',import.meta.url)),
 });else await rm(wasmTarget,{recursive:true,force:true});
 const source=localPath(ocsRoot),target=path.resolve(destination,'ocs','app');
 if(!target.startsWith(path.resolve(destination)+path.sep))throw Error('Viewer output escapes build directory');
 if(await access(path.join(source,'index.html')).then(()=>true,()=>false)){
  const html=await readFile(path.join(source,'index.html'),'utf8');
  if(!html.includes('</body>'))throw Error('Open CAD Studio HTML has no body end');
  await bundle(source,target,{
   'index.html':Buffer.from(html.replace('</body>','<script type="module" src="./ocs-bridge.mjs"></script></body>')),
   'ocs-bridge.mjs':await readFile(new URL('../src/ocs-bridge.mjs',import.meta.url)),
   'ocs-messages.mjs':await readFile(new URL('../src/ocs-messages.mjs',import.meta.url)),
   'ocs-selection.mjs':await readFile(new URL('../src/ocs-selection.mjs',import.meta.url)),
   'SOURCE.json':await readFile(new URL('../ocs-source.json',import.meta.url)),
  });
 }else await rm(target,{recursive:true,force:true});
 console.log('Built CAD Format Explorer: '+destination);
}
if(process.argv[1]===fileURLToPath(import.meta.url))await build();
