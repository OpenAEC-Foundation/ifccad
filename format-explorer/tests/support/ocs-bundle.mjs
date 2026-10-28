import {readFile,writeFile} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';

export async function recordBundle(source){
 const pins=JSON.parse(await readFile(new URL('../../ocs-source.json',import.meta.url),'utf8'));
 const files={};
 for(const name of ['index.html','app.wasm','worker_pkg/ocs_web_worker_bg.wasm'])files[name]=createHash('sha256').update(await readFile(path.join(source,name))).digest('hex');
 await writeFile(path.join(source,'IFCCAD-BUILD.json'),JSON.stringify({pins,files}));
}
