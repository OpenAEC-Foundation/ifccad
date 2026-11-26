import test from 'node:test';
import assert from 'node:assert/strict';
import {connectOcsFrame} from '../src/ocs-bridge.mjs';

test('frame accepts only its same-origin parent and active token',async()=>{
 const sent=[],calls=[];let onMessage;
 const parent={postMessage(message,origin){sent.push({message,origin});}};
 const host={parent,location:{origin:'https://explorer.example'},addEventListener(name,fn){if(name==='message')onMessage=fn;}};
 const api={ocs_control_submit:()=> '1',ocs_control_take:()=>JSON.stringify({ok:true,document_id:1,revision:0,modal:null})};
 connectOcsFrame(host,{waitForApi:async()=>api,sessionFactory:()=>({openOriginal:async(base64,name)=>{calls.push([base64,name]);return 7;},replaceGenerated:async()=>8})});
 onMessage({origin:'https://other.example',source:parent,data:{channel:'ocdraw-ocs',token:'x',op:'init'}});
 assert.equal(sent.length,0);
 onMessage({origin:host.location.origin,source:parent,data:{channel:'ocdraw-ocs',token:'x',op:'init'}});
 await new Promise(resolve=>setTimeout(resolve,0));assert.equal(sent[0].message.status,'ready');
 onMessage({origin:host.location.origin,source:parent,data:{channel:'ocdraw-ocs',token:'old',id:'1',op:'open',role:'original',base64:'YQ==',name:'a.dxf'}});
 onMessage({origin:host.location.origin,source:parent,data:{channel:'ocdraw-ocs',token:'x',id:'2',op:'open',role:'original',base64:'YQ==',name:'a.dxf'}});
 await new Promise(resolve=>setTimeout(resolve,0));
 assert.deepEqual(calls,[['YQ==','a.dxf']]);assert.equal(sent.at(-1).message.documentId,7);assert.equal(sent.at(-1).message.id,'2');
});

test('source choice preserves the scoped host result and a displayable loaded status',async()=>{
 const sent=[],calls=[];let onMessage;
 const parent={postMessage(message){sent.push(message);}};
 const host={parent,location:{origin:'https://explorer.example'},addEventListener(name,fn){if(name==='message')onMessage=fn;}};
 const api={ocs_control_submit:()=> '1',ocs_control_take:()=>JSON.stringify({ok:true,document_id:1,revision:0,modal:null})};
 connectOcsFrame(host,{waitForApi:async()=>api,sessionFactory:()=>({registerExternalSource:async(...args)=>{calls.push(args);return {source_status:'loaded',other_drawing:true,diagnostics:['identity changed']};}})});
 onMessage({origin:host.location.origin,source:parent,data:{channel:'ocdraw-ocs',token:'token',op:'init'}});
 await new Promise(resolve=>setTimeout(resolve,0));
 onMessage({origin:host.location.origin,source:parent,data:{channel:'ocdraw-ocs',token:'token',id:'source',op:'source',role:'generated',sourceId:'9007199254740993',fileName:'ground.dwg',base64:'YQ==',documentId:8}});
 await new Promise(resolve=>setTimeout(resolve,0));
 assert.deepEqual(calls,[['9007199254740993','ground.dwg','YQ==',8]]);
 assert.equal(sent.at(-1).sourceStatus,'Loaded');
 assert.equal(sent.at(-1).otherDrawing,true);
 assert.deepEqual(sent.at(-1).diagnostics,['identity changed']);
});
