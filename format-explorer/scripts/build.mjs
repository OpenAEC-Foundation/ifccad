import { mkdir, copyFile, writeFile, readFile, readdir, cp, access, rm } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { readExamples } from './fixtures.mjs';
export const output = new URL('../dist/', import.meta.url);
export async function build({outputRoot=output,ocsRoot=new URL('../ocs-build/',import.meta.url)}={}) {
  const destination=outputRoot instanceof URL?fileURLToPath(outputRoot):path.resolve(outputRoot);
  await mkdir(destination,{recursive:true});
  for(const name of await readdir(new URL('../src/',import.meta.url))){if(!/\.(html|css|mjs|svg|ttf|txt)$/.test(name))continue;await copyFile(new URL('../src/'+name,import.meta.url),path.join(destination,name));}
  await writeFile(path.join(destination,'examples.json'),JSON.stringify(await readExamples()));
  const bundle=ocsRoot instanceof URL?fileURLToPath(ocsRoot):path.resolve(ocsRoot);
  const target=path.resolve(destination,'ocs','app');
  if(!target.startsWith(path.resolve(destination)+path.sep))throw Error('Viewer output must stay inside the build directory');
  await rm(target,{recursive:true,force:true});
  if(await access(path.join(bundle,'index.html')).then(()=>true,()=>false)){
    await cp(bundle,target,{recursive:true,force:true});
    await copyFile(new URL('../src/ocs-bridge.mjs',import.meta.url),path.join(target,'ocs-bridge.mjs'));
    await copyFile(new URL('../src/ocs-messages.mjs',import.meta.url),path.join(target,'ocs-messages.mjs'));
    const htmlFile=path.join(target,'index.html'),html=await readFile(htmlFile,'utf8');
    if(!html.includes('</body>'))throw Error('Open CAD Studio HTML has no body end');
    await writeFile(htmlFile,html.replace('</body>','<script type="module" src="./ocs-bridge.mjs"></script></body>'));
    await writeFile(path.join(target,'SOURCE.json'),await readFile(new URL('../ocs-source.json',import.meta.url)));
  }
  console.log('Built independent static viewer: '+destination);
}
if(process.argv[1]===fileURLToPath(import.meta.url))await build();
