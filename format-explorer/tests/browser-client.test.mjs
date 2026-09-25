import test from 'node:test';
import assert from 'node:assert/strict';
import {createFileClient} from '../src/browser-client.mjs';

test('browser processing is the default and never calls the upload service',async()=>{
 let worker,uploads=0;
 class FakeWorker {
  constructor(){worker=this;}
  postMessage(message){this.request=message.request;queueMicrotask(()=>this.onmessage({data:{type:'result',result:{ok:true}}}));}
  terminate(){}
 }
 const client=createFileClient({workerFactory:()=>new FakeWorker(),serverClient:{open(){uploads++;}}});
 const result=await client.open({kind:'cad',name:'a.dxf',files:[{path:'a.dxf',bytes:new Uint8Array([1,2,3]).buffer}]});
 assert.deepEqual(result,{ok:true});assert.equal(uploads,0);
 assert.deepEqual([...new Uint8Array(worker.request.files[0].bytes)],[1,2,3]);
});

test('server processing requires an explicit choice and sends encoded bytes',async()=>{
 let upload;
 const client=createFileClient({workerFactory:()=>{throw Error('worker should not start');},serverClient:{open(request){upload=request;return Promise.resolve({ok:true});}}});
 await client.open({kind:'cad',processing:'server',name:'a.dxf',files:[{path:'a.dxf',bytes:new Uint8Array([1,2,3]).buffer}]});
 assert.equal(upload.files[0].base64,'AQID');assert.equal(upload.processing,undefined);
});

test('cancelling a browser job terminates its worker',async()=>{
 let worker;
 class FakeWorker {constructor(){worker=this;}postMessage(){}terminate(){this.terminated=true;}}
 const client=createFileClient({workerFactory:()=>new FakeWorker(),serverClient:{}});
 const controller=new AbortController();
 const pending=client.open({kind:'package',name:'p',files:[]},{signal:controller.signal});
 controller.abort();
 await assert.rejects(pending,error=>error.name==='AbortError');
 assert.equal(worker.terminated,true);
});
