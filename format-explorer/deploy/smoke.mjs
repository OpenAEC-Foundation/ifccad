import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {setTimeout as delay} from 'node:timers/promises';

// Exercise the real HTTP service and Rust worker with public repository fixtures.
const [base='http://127.0.0.1:4183', revision, mode='full'] = process.argv.slice(2);
const origin='https://ifccad-explorer.open-aec.com';
async function request(path, options={}) {
  const response=await fetch(base+path,{...options,signal:AbortSignal.timeout(15000),headers:{Host:new URL(origin).host,Origin:origin,...options.headers}});
  assert.equal(response.ok,true,`${path}: HTTP ${response.status}`);
  return response;
}
const version=await (await request('/version.json')).json();
assert.equal(version.revision,revision,'Live revision must match the tested commit');
const capabilities=await (await request('/api/capabilities')).json();
assert.equal(capabilities.available,true);
assert.equal(capabilities.processing,'server');
const html=await (await request('/')).text();
assert.match(html,/id="export-format"/);
assert.ok(html.indexOf('id="export-format"')<html.indexOf('id="export-drawing"'));
if(mode==='full') {
  const examples=JSON.parse(await readFile(new URL('../dist/examples.json',import.meta.url),'utf8'));
  const example=examples.find(value=>value.name==='unrepresented-packed');
  const source={kind:'package',name:'Deployment smoke test',files:example.exportFiles};
  async function job(payload) {
    const {id,token}=await (await request('/api/jobs',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)})).json();
    const headers={Authorization:'Bearer '+token};
    try {
      const deadline=Date.now()+125000;
      while(Date.now()<deadline) {
        const result=await (await request('/api/jobs/'+id,{headers})).json();
        assert.notEqual(result.status,'failed',result.error);
        if(result.status==='complete') {
          assert.ok(!result.result.failure,JSON.stringify(result.result.failure));
          assert.equal(result.result.validation.strictAvailable,true);
          return result.result;
        }
        await delay(350);
      }
      throw Error('Processing did not complete');
    } finally { await request('/api/jobs/'+id,{method:'DELETE',headers}); }
  }
  await job(source);
  for(const format of ['dxf','dwg','ifccad']) {
    const result=await job({...source,export:{format,drawing:'drawing-main'}});
    const download=result.export?.download;
    assert.equal(download?.format,format);
    const bytes=Buffer.from(download.base64,'base64');
    assert.equal(bytes.length,download.byteLength);
    assert.ok(bytes.length>100);
    if(format==='ifccad')assert.equal(bytes.subarray(0,2).toString(),'PK');
    else await job({kind:'cad',name:'Deployment readback',files:[{path:'readback.'+format,base64:download.base64}]});
    console.log(`Verified ${format} export${format==='ifccad'?'':' and CAD reopening'}`);
  }
}
console.log(`Explorer verified at ${revision}`);
