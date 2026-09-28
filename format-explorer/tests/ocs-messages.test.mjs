import test from 'node:test';
import assert from 'node:assert/strict';
import {isCurrentOcsMessage,createDocumentSession,createOcsControl,createOcsSession} from '../src/ocs-messages.mjs';

test('only the active same-origin frame and token may answer',()=>{
 const frame={contentWindow:{}},token='current',origin='https://explorer.example';
 const good={origin,source:frame.contentWindow,data:{channel:'ifccad-ocs',token,status:'opened'}};
 assert.equal(isCurrentOcsMessage(good,frame,token,origin),true);
 for(const bad of [{...good,origin:'https://other.example'},{...good,source:{}},{...good,data:{...good.data,token:'old'}},{...good,data:{...good.data,status:'unknown'}}])assert.equal(isCurrentOcsMessage(bad,frame,token,origin),false);
});

test('parent session ignores stale replies and serializes document requests',async()=>{
 const listeners=new Map(),posted=[];
 const host={addEventListener(name,fn){listeners.set(name,fn);},removeEventListener(name){listeners.delete(name);}};
 const frame={contentWindow:{postMessage(message,origin){posted.push({message,origin});}},addEventListener(name,fn){listeners.set('frame-'+name,fn);},removeEventListener(name){listeners.delete('frame-'+name);}};
 const origin='https://explorer.example',token='current';
 const session=createOcsSession(frame,token,{host,origin,timeoutMs:1000});
 listeners.get('frame-load')();assert.equal(posted[0].message.op,'init');
 listeners.get('message')({origin,source:frame.contentWindow,data:{channel:'ifccad-ocs',token,status:'ready'}});
 const first=session.openOriginal('YQ==','original.dxf');
 const second=session.replaceGenerated('Yg==','generated.dxf');
 await new Promise(resolve=>setTimeout(resolve,0));
 assert.equal(posted.filter(p=>p.message.op==='open').length,1);
 const open=posted.find(p=>p.message.op==='open').message;
 listeners.get('message')({origin,source:frame.contentWindow,data:{channel:'ifccad-ocs',token:'old',id:open.id,status:'opened'}});
 assert.equal(posted.filter(p=>p.message.op==='replace').length,0);
 listeners.get('message')({origin,source:frame.contentWindow,data:{channel:'ifccad-ocs',token,id:open.id,status:'opened'}});
 await first;await new Promise(resolve=>setTimeout(resolve,0));
 const replace=posted.find(p=>p.message.op==='replace').message;assert.ok(replace);
 listeners.get('message')({origin,source:frame.contentWindow,data:{channel:'ifccad-ocs',token,id:replace.id,status:'opened'}});
 await second;session.close();assert.equal(listeners.has('message'),false);
});

test('viewer startup error rejects readiness promptly',async()=>{
 const listeners=new Map(),host={addEventListener(name,fn){listeners.set(name,fn);},removeEventListener(name){listeners.delete(name);}};
 const frame={contentWindow:{postMessage(){}},addEventListener(name,fn){listeners.set('frame-'+name,fn);},removeEventListener(name){listeners.delete('frame-'+name);}};
 const session=createOcsSession(frame,'token',{host,origin:'https://explorer.example',timeoutMs:1000});
 listeners.get('message')({origin:'https://explorer.example',source:frame.contentWindow,data:{channel:'ifccad-ocs',token:'token',status:'error',message:'WASM unavailable'}});
 await assert.rejects(session.ready,/WASM unavailable/);session.close();
});

test('control adapter waits for asynchronous open and retries a busy response',async()=>{
 const requests=[];let opens=0;
 const api={ocs_control_submit(json){const request=JSON.parse(json);requests.push(request);return String(requests.length);},ocs_control_take(ticket){const request=requests[Number(ticket)-1];
  if(request.op==='state')return JSON.stringify({ok:true,document_id:1,revision:0,documents:[{id:1}]});
  if(request.op==='open'){opens++;return JSON.stringify(opens===1?{ok:false,status:'failed',code:'busy',error:'opening'}:{ok:true,status:'accepted',request_id:request.request_id});}
  if(request.op==='operation')return JSON.stringify({ok:true,status:'completed',request_id:request.request_id});
 }};
 const control=createOcsControl(api,{delay:async()=>{},maxPolls:4,maxBusyRetries:2});
 const reply=await control({op:'open',name:'a.dxf',data_base64:'YQ=='});
 assert.equal(reply.status,'completed');assert.equal(opens,2);
 assert.ok(requests.filter(r=>r.op==='open').every(r=>r.request_id));
});

test('one session opens original then generated and replaces only generated',async()=>{
 let current=1,next=2;const docs=new Map([[1,{id:1,title:'Start',start:true}]]),calls=[];
 const control=async request=>{
  calls.push(request);
  if(request.op==='state')return {ok:true,document_id:current,revision:0,documents:[...docs.values()],modal:null};
  if(request.op==='open'){current=next++;docs.set(current,{id:current,title:request.name});return {ok:true,status:'completed'};}
  if(request.op==='activate'){current=request.document_id;return {ok:true,status:'completed'};}
  if(request.op==='action'&&request.name==='close_document'){docs.delete(current);current=[...docs.keys()].at(-1);return {ok:true,status:'completed'};}
  throw Error('Unexpected '+JSON.stringify(request));
 };
 const session=createDocumentSession(control);
 await session.openOriginal('b3JpZw==','original.dxf');
 await session.replaceGenerated('Z2VuMQ==','via-ifccad-2018.dxf');
 await session.replaceGenerated('Z2VuMg==','via-ifccad-2013.dxf');
 assert.deepEqual([...docs.values()].filter(d=>!d.start).map(d=>d.title),['original.dxf','via-ifccad-2013.dxf']);
 assert.deepEqual(calls.filter(c=>c.op==='open').map(c=>c.name),['original.dxf','via-ifccad-2018.dxf','via-ifccad-2013.dxf']);
 assert.equal(calls.filter(c=>c.op==='action'&&c.name==='close_document').length,1);
});

test('switching versions preserves an edited roundtrip tab and opens the next file',async()=>{
 let current=1,next=2;const docs=new Map([[1,{id:1,title:'Start',start:true,dirty:false}]]),calls=[];
 const control=async request=>{
  calls.push(request);
  if(request.op==='state')return {ok:true,document_id:current,revision:0,documents:[...docs.values()],modal:null};
  if(request.op==='open'){current=next++;docs.set(current,{id:current,title:request.name,dirty:false});return {ok:true,status:'completed'};}
  if(request.op==='activate'){current=request.document_id;return {ok:true,status:'completed'};}
  if(request.op==='action'&&request.name==='close_document'){
   if(docs.get(current)?.dirty)return {ok:true,status:'completed'};
   docs.delete(current);current=[...docs.keys()].at(-1);return {ok:true,status:'completed'};
  }
  throw Error('Unexpected '+JSON.stringify(request));
 };
 const session=createDocumentSession(control);
 await session.openOriginal('b3JpZw==','original.dxf');
 await session.replaceGenerated('Z2VuMQ==','via-ifccad-2018.dxf');
 docs.get(session.ids().generatedId).dirty=true;
 await session.replaceGenerated('Z2VuMg==','via-ifccad-2013.dxf');
 assert.deepEqual([...docs.values()].filter(d=>!d.start).map(d=>d.title),['original.dxf','via-ifccad-2018.dxf','via-ifccad-2013.dxf']);
 assert.equal(calls.filter(c=>c.op==='action'&&c.name==='close_document').length,0);
});
