import assert from 'node:assert/strict';
import {request as httpRequest} from 'node:http';
import {request as httpsRequest} from 'node:https';

// Exercise the static website and its shipped WebAssembly with public fixtures.
const [base='http://127.0.0.1:4183', revision, mode='full'] = process.argv.slice(2);
const origin='https://ifccad-explorer.open-aec.com';
async function request(path, options={}) {
  // Node fetch ignores a custom Host header. The container's loopback check
  // must send the same Host as nginx without weakening production validation.
  const url=new URL(base+path);
  return new Promise((resolve,reject)=>{
    const req=(url.protocol==='https:'?httpsRequest:httpRequest)(url,{method:options.method,signal:AbortSignal.timeout(15000),headers:{Host:new URL(origin).host,Origin:origin,...options.headers}},res=>{
      const chunks=[];
      res.on('data',chunk=>chunks.push(chunk));res.on('error',reject);
      res.on('end',()=>{
        if(res.statusCode<200||res.statusCode>=300)return reject(Error(`${path}: HTTP ${res.statusCode}`));
        const body=Buffer.concat(chunks).toString();
        resolve({headers:res.headers,json:()=>JSON.parse(body),text:()=>body});
      });
    });
    req.on('error',reject);req.end(options.body);
  });
}
const version=await (await request('/version.json')).json();
assert.equal(version.revision,revision,'Live revision must match the tested commit');
const html=await (await request('/')).text();
assert.match(html,/id="preview-format"/);
assert.match(html,/id="cad-download"/);
assert.match(html,/id="export"/);
assert.match(html,/id="preview-fullscreen"/);
assert.match(html,/id="preview-open"/);
assert.match(html,/id="preview-frame"/);
assert.doesNotMatch(html,/id="processing"/);
const processorHead=await request('/wasm/ocdraw_browser_bg.wasm',{method:'HEAD'});
assert.equal(processorHead.headers['content-type'],'application/wasm');
const processorScript=await (await request('/wasm/ocdraw_browser.js')).text();
assert.match(processorScript,/convert_cad_to_drawing/);
assert.match(processorScript,/open_drawing/);
for(const api of ['open_ifcx','convert_cad_to_ifcx','export_ifcx'])assert.ok(processorScript.includes(`export function ${api}(`),`Missing IFCX API: ${api}`);
const ifcxExampleHead=await request('/examples/hello-line-patterns.ifcx',{method:'HEAD'});
assert.match(ifcxExampleHead.headers['content-type'],/^application\/json/);
if(mode==='full') {
  const ocsHtml=await (await request('/ocs/app/index.html')).text();
  assert.match(ocsHtml,/ocs-bridge\.mjs/);
  const viewerWasm=ocsHtml.match(/\/ocs\/app\/([^'"\s]+\.wasm)/);
  assert.ok(viewerWasm,'Open CAD Studio WASM reference is missing');
  const wasmHead=await request('/ocs/app/'+viewerWasm[1],{method:'HEAD'});
  assert.equal(wasmHead.headers['content-type'],'application/wasm');
  assert.equal(wasmHead.headers['x-frame-options'],'SAMEORIGIN');
  const workerHead=await request('/ocs/app/worker_pkg/ocs_web_worker_bg.wasm',{method:'HEAD'});
  assert.equal(workerHead.headers['content-type'],'application/wasm');
  const fixture=await (await request('/examples/ordered-scopes.ocdraw.json')).text();
  const {initSync,open_drawing,convert_cad_to_drawing,export_drawing}=await import('../dist/wasm/ocdraw_browser.js');
  const {readFile}=await import('node:fs/promises');
  const {processBrowserRequest}=await import('../dist/browser-worker.mjs');
  initSync({module:await readFile(new URL('../dist/wasm/ocdraw_browser_bg.wasm',import.meta.url))});
  const wasm={open_drawing,convert_cad_to_drawing,export_drawing};
  const source={kind:'drawing',name:'ordered-scopes',files:[{path:'ordered-scopes.ocdraw.json',bytes:Uint8Array.from(Buffer.from(fixture)).buffer}]};
  const process=request=>{const result=processBrowserRequest(request,wasm);assert.equal(result.failure,null,JSON.stringify(result.failure));assert.equal(result.validation.strictAvailable,true);return result;};
  process(source);
  for(const format of ['dxf','dwg','ocdraw']){
   const result=process({...source,export:{format,version:'AC1027'}}),download=result.export.download;
   assert.equal(download.format,format);
   const bytes=Buffer.from(download.base64,'base64');assert.equal(bytes.length,download.byteLength);
   process({kind:format==='ocdraw'?'drawing':'cad',name:'readback',files:[{path:'readback.'+(format==='ocdraw'?'ocdraw.json':format),bytes:Uint8Array.from(bytes).buffer}]});
   console.log(`Verified browser WASM ${format} export and readback`);
  }
}
console.log(`Explorer verified at ${revision}`);
