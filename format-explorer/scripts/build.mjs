import { mkdir, copyFile, writeFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { readExamples } from './fixtures.mjs';
export const output = new URL('../dist/', import.meta.url);
export async function build() {
  await mkdir(output,{recursive:true});
  for(const name of await readdir(new URL('../src/',import.meta.url))){if(!/\.(html|css|mjs|svg|ttf|txt)$/.test(name))continue;await copyFile(new URL('../src/'+name,import.meta.url),new URL(name,output));}
  await writeFile(new URL('examples.json',output),JSON.stringify(await readExamples()));
  console.log('Built independent static viewer: '+fileURLToPath(output));
}
if(process.argv[1]===fileURLToPath(import.meta.url))await build();
