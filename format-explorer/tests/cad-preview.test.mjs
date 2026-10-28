import test from 'node:test';
import assert from 'node:assert/strict';
import {createCadPreviewController} from '../src/cad-preview.mjs';
const file={format:'ifccad',fileName:'drawing.ifcx',base64:'e30=',byteLength:2};
const source={kind:'cad',name:'original.dwg',files:[{path:'original.dwg',bytes:new Uint8Array([1,2]).buffer}]};
const opening={validation:{strictAvailable:true},nativeFile:file,preview:{viewerSelection:{format:'ifccad',entities:[{path:'/cad/d1/e1',handle:'A',layout:'Model'}]}}};

test('preview opens the inspected native bytes without CAD serialization and ignores output settings',async()=>{
 const calls=[];let exports=0;
 const controller=createCadPreviewController({openExport:async()=>{exports++;throw Error('No CAD export');},openSession:async()=>({openOriginal:async(...args)=>calls.push(['original',...args]),replaceGenerated:async(...args)=>calls.push(['native',...args]),close(){}})});
 controller.setSource(source,true,opening);await controller.show();
 assert.equal(exports,0);assert.deepEqual(calls,[['original','AQI=','original.dwg'],['native','e30=','drawing.ifcx']]);
 await controller.select({format:'dxf',version:'AC1024'});assert.equal(calls.length,2);
});

test('native selection uses import mappings independently of CAD output',async()=>{
 const selected=[],revealed=[];let observe;
 const controller=createCadPreviewController({onCadSelection:key=>revealed.push(key),openSession:async()=>({replaceGenerated:async()=>{},selectGenerated:async(...args)=>selected.push(args),subscribeSelection(fn){observe=fn;},close(){}})});
 controller.setSource({...source,kind:'ifccad'},true,opening);await controller.show();
 await controller.selectElement('/cad/d1/e1');assert.deepEqual(selected,[ [['A'],'Model'] ]);
 observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,[]);
 await controller.selectLayout('Paper');assert.deepEqual(selected.at(-1),[[],'Paper']);
 observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,['/cad/d1/e1']);
});

test('an invalid or absent native snapshot never starts an export or opens substitute bytes',async()=>{
 let exports=0;const calls=[];
 const controller=createCadPreviewController({openExport:async()=>exports++,openSession:async()=>({openOriginal:async()=>calls.push('original'),replaceGenerated:async()=>calls.push('native'),close(){}})});
 controller.setSource(source,true,{...opening,nativeFile:{...file,byteLength:3}});await controller.show();
 assert.equal(exports,0);assert.deepEqual(calls,['original']);assert.ok(controller.state.error);
});

test('cancellation during viewer startup cannot publish a late native document',async()=>{
 let finish;const calls=[];
 const controller=createCadPreviewController({openSession:()=>new Promise(resolve=>{finish=resolve;})});
 controller.setSource(source,true,opening);const pending=controller.show();
 while(!finish)await new Promise(resolve=>setImmediate(resolve));controller.cancel();
 finish({openOriginal:async()=>calls.push('original'),replaceGenerated:async()=>calls.push('native'),close(){}});await pending;
 assert.deepEqual(calls,[]);assert.equal(controller.state.download,null);assert.equal(controller.state.phase,'cancelled');
});

test('replacing a source retains the Studio session and requests safe original replacement',async()=>{
 const calls=[];let sessions=0;
 const controller=createCadPreviewController({openSession:async()=>{sessions++;return {replaceOriginal:async(...args)=>calls.push(['original',...args]),replaceGenerated:async(...args)=>calls.push(['native',...args]),close(){calls.push(['close']);}};}});
 controller.setSource(source,true,opening);await controller.show();controller.clear();
 controller.setSource({...source,name:'second.dwg',files:[{path:'second.dwg',bytes:source.files[0].bytes}]},true,opening);await controller.show();
 assert.equal(sessions,1);assert.deepEqual(calls.map(row=>row[0]),['original','native','original','native']);
});

test('unknown, multiple, hidden and stale CAD selections do not change native inspection',async()=>{
 let observe;const revealed=[];
 const controller=createCadPreviewController({onCadSelection:key=>revealed.push(key),openSession:async()=>({replaceGenerated:async()=>{},subscribeSelection(fn){observe=fn;},close(){}})});
 controller.setSource({...source,kind:'ifccad'},true,opening);await controller.show();
 for(const event of [{handles:[],layout:'Model'},{handles:['A','B'],layout:'Model'},{handles:['A'],layout:'Paper'},{handles:['F'],layout:'Model'}])observe(event);
 assert.deepEqual(revealed,[]);controller.hide();observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,[]);
 await controller.show();controller.clear();observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,[]);
});

test('later native selection wins without reloading the prepared IFCX',async()=>{
 let finish;const calls=[];
 const controller=createCadPreviewController({openSession:async()=>({replaceGenerated:async()=>calls.push('open'),selectGenerated:async(handles,layout)=>{calls.push([handles,layout]);if(layout==='Paper')await new Promise(resolve=>{finish=resolve;});},close(){}})});
 controller.setSource({...source,kind:'ifccad'},true,opening);await controller.show();
 const first=controller.selectLayout('Paper');while(!finish)await new Promise(resolve=>setImmediate(resolve));const second=controller.selectElement('/cad/d1/e1');finish();await Promise.all([first,second]);
 assert.deepEqual(calls,['open',[[],'Paper'],[['A'],'Model']]);assert.equal(controller.state.selectionStatus,'selected');
});

test('updating a validated snapshot keeps selection observation alive and does not reopen original CAD',async()=>{
 let observe;const revealed=[],calls=[];
 const controller=createCadPreviewController({onCadSelection:key=>revealed.push(key),openSession:async()=>({openOriginal:async()=>calls.push('original'),replaceGenerated:async()=>calls.push('native'),subscribeSelection(fn){observe=fn;},close(){}})});
 controller.setSource(source,true,opening);await controller.show();controller.updateSource(source,true,opening);await controller.show();
 observe({handles:['A'],layout:'Model'});assert.deepEqual(revealed,['/cad/d1/e1']);assert.deepEqual(calls,['original','native','native']);
});
