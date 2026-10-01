import test from 'node:test';
import assert from 'node:assert/strict';
import {createFileClient} from '../src/browser-client.mjs';

test('browser processing is the default and never calls the upload service',async()=>{
 let worker;
 class FakeWorker {
  constructor(){worker=this;}
  postMessage(message){this.request=message.request;queueMicrotask(()=>this.onmessage({data:{type:'result',result:{ok:true}}}));}
  terminate(){}
 }
 const client=createFileClient({workerFactory:()=>new FakeWorker()});
 const result=await client.open({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes:new Uint8Array([1,2,3]).buffer}]});
 assert.deepEqual(result,{ok:true});
 assert.deepEqual([...new Uint8Array(worker.request.files[0].bytes)],[1,2,3]);
});

test('the browser client has no upload dependency or server branch',async()=>{
 const {readFile}=await import('node:fs/promises');
 const source=await readFile(new URL('../src/browser-client.mjs',import.meta.url),'utf8');
 assert.doesNotMatch(source,/job-client|serverClient|processing==='server'/);
});

test('cancelling a browser job terminates its worker',async()=>{
 let worker;
 class FakeWorker {constructor(){worker=this;}postMessage(){}terminate(){this.terminated=true;}}
 const client=createFileClient({workerFactory:()=>new FakeWorker()});
 const controller=new AbortController();
 const pending=client.open({kind:'drawing',name:'p',files:[]},{signal:controller.signal});
 controller.abort();
 await assert.rejects(pending,error=>error.name==='AbortError');
 assert.equal(worker.terminated,true);
});
