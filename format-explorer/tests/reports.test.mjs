import test from 'node:test';
import assert from 'node:assert/strict';
import {summarizeEntities} from '../src/reports.mjs';
import {createJobClient} from '../src/job-client.mjs';
test('entity summary counts partial once regardless of number of reasons',()=>{
 const result=summarizeEntities([{kind:'LINE',disposition:'emitted',reasons:[]},{kind:'LINE',disposition:'partial',reasons:['a','b']},{kind:'CIRCLE',disposition:'skipped',reasons:['x']}]);
 assert.deepEqual(result[0],{kind:'LINE',read:2,emitted:2,partial:1,skipped:0,unclassified:0});
});
test('client always cancels server job when aborted during upload',async()=>{
 const controller=new AbortController(),calls=[];
 const client=createJobClient({fetchImpl:async(url,options)=>{calls.push(options?.method);if(options?.method==='POST')controller.abort();return {ok:true,text:async()=>'{"id":"test"}'};}});
 await assert.rejects(client.open({}, {signal:controller.signal}),{name:'AbortError'});assert.deepEqual(calls,['POST','DELETE']);
});
