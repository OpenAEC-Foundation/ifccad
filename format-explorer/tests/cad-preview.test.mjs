import test from 'node:test';
import assert from 'node:assert/strict';
import {createCadPreviewController} from '../src/cad-preview.mjs';
const source={kind:'cad',name:'original.dwg',files:[{path:'original.dwg',bytes:new Uint8Array([1,2]).buffer}]};
const result={failure:null,validation:{strictAvailable:true},export:{requestedVersion:'AC1032',download:{format:'dwg',base64:'AQI=',byteLength:2}}};

test('CAD selection reverses the qualified map without export, feedback or stale-file updates',async()=>{
 for(const format of ['ifccad','ocdraw']){
  const keys=format==='ifccad'?['/cad/d1/e1','/cad/d1/e2']:['entity:1','entity:2'];let listener,exports=0,selects=0;const revealed=[];
  const output={...result,export:{...result.export,download:{...result.export.download,format:'dxf'},viewerSelection:{format,entities:[{path:keys[0],handle:'AB',layout:'Model'},{path:keys[1],handle:'BB',layout:'Model'}]}}};
  const controller=createCadPreviewController({openExport:async()=>{exports++;return output;},openSession:async()=>({subscribeSelection(fn){listener=fn;return()=>{};},setVisible(){},async replaceGenerated(){},async selectGenerated(){selects++;},close(){}}),onCadSelection:key=>revealed.push(key)});
  controller.setSource({kind:format==='ifccad'?'ifccad':'drawing',name:'source',files:source.files},true);await controller.show();
  assert.equal(typeof listener,'function');listener({handles:['AB'],layout:'Model'});assert.deepEqual(revealed,[keys[0]]);await controller.show();assert.equal(selects,0);assert.equal(exports,1);
  listener({handles:['AB'],layout:'Model'});assert.equal(revealed.length,1);
  await controller.selectElement(keys[1]);listener({handles:['BB'],layout:'Model'});assert.equal(revealed.length,1);assert.equal(selects,1);
  for(const event of [{handles:['FF'],layout:'Model'},{handles:['AB'],layout:'Paper'},{handles:['AB','BB'],layout:'Model'},{handles:[],layout:'Model'}])listener(event);assert.equal(revealed.length,1);
  controller.hide();listener({handles:['AB'],layout:'Model'});assert.equal(revealed.length,1);await controller.show();controller.clear();listener({handles:['AB'],layout:'Model'});assert.equal(revealed.length,1);
 }
});

test('OCDraw selection uses its qualified mapping without reloading or preparing the drawing',async()=>{
 const selected=[],updates=[];let exports=0,replacements=0;
 const output={...result,export:{...result.export,download:{...result.export.download,format:'dxf'},viewerSelection:{format:'ocdraw',entities:[{path:'entity:9007199254740993',handle:'CAFE',layout:'Model'}]}}};
 const controller=createCadPreviewController({openExport:async()=>{exports++;return output;},openSession:async()=>({async replaceGenerated(){replacements++;},async selectGenerated(handles,layout){selected.push([handles,layout]);},close(){}}),onUpdate:state=>updates.push(state)});
 controller.setSource({kind:'drawing',name:'input.ocdraw.json',files:source.files},true);await controller.show();updates.length=0;
 await controller.selectElement('entity:9007199254740993');assert.deepEqual(selected.at(-1),[['CAFE'],'Model']);assert.equal(controller.state.selectionStatus,'selected');
 await controller.selectElement(null);assert.deepEqual(selected.at(-1),[[],undefined]);assert.equal(controller.state.selectionStatus,'');
 assert.equal(exports,1);assert.equal(replacements,1);assert.equal(updates.some(state=>state.busy||state.viewerBusy),false);
});

test('selection in an already displayed drawing never reactivates drawing preparation',async()=>{
 let finishSelection,exports=0,opens=0,replacements=0;const updates=[];
 const output={...result,export:{...result.export,download:{...result.export.download,format:'dxf'},viewerSelection:{format:'ifccad',entities:[{path:'/cad/d1/e1',handle:'A',layout:'Model'}]}}};
 const controller=createCadPreviewController({openExport:async()=>{exports++;return output;},openSession:async()=>{opens++;return {async replaceGenerated(){replacements++;},selectGenerated:()=>new Promise(resolve=>{finishSelection=resolve;}),close(){}};},onUpdate:state=>updates.push(state)});
 controller.setSource({kind:'ifccad',name:'input.ifcx',files:source.files},true);await controller.show();updates.length=0;
 const selecting=controller.selectElement('/cad/d1/e1');while(!finishSelection)await new Promise(resolve=>setImmediate(resolve));
 assert.equal(updates.some(state=>state.busy||state.viewerBusy||!state.viewerReady),false);
 assert.equal(controller.state.selectionStatus,'pending');finishSelection();await selecting;
 assert.equal(controller.state.selectionStatus,'selected');assert.equal(exports,1);assert.equal(opens,1);assert.equal(replacements,1);
 assert.equal(updates.some(state=>state.busy||state.viewerBusy||!state.viewerReady),false);
});

test('native tree selection waits for prepared CAD and uses its exact mapping without re-exporting',async()=>{
 let finish,exports=0;const selections=[];
 const session={async replaceGenerated(){},async selectGenerated(handles,layout){selections.push([handles,layout]);},close(){}};
 const controller=createCadPreviewController({openExport:()=>{exports++;return new Promise(resolve=>{finish=resolve;});},openSession:async()=>session});
 controller.setSource({kind:'ifccad',name:'input.ifcx',files:source.files},true);
 assert.equal(typeof controller.selectElement,'function');const showing=controller.show();const selecting=controller.selectElement('/cad/d1/e9007199254740993');
 while(!finish)await new Promise(resolve=>setImmediate(resolve));finish({...result,export:{...result.export,download:{...result.export.download,format:'dxf'},viewerSelection:{format:'ifccad',entities:[{path:'/cad/d1/e9007199254740993',handle:'CAFE',layout:'Model'}]}}});await Promise.all([showing,selecting]);
 assert.deepEqual(selections.at(-1),[['CAFE'],'Model']);assert.equal(exports,1);assert.equal(controller.state.selectionStatus,'selected');
 await controller.selectElement('/foreign/unmapped');assert.deepEqual(selections.at(-1),[[],undefined]);assert.equal(controller.state.selectionStatus,'unavailable');assert.equal(exports,1);
});

test('a new file discards a pending native selection and rapid selection keeps the last item',async()=>{
 let finish;const selected=[];
 const output={...result,export:{...result.export,download:{...result.export.download,format:'dxf'},viewerSelection:{format:'ifccad',entities:[{path:'/cad/d1/e1',handle:'A',layout:'Model'},{path:'/cad/d1/e2',handle:'B',layout:'Model'}]}}};
 const controller=createCadPreviewController({openExport:()=>new Promise(resolve=>{finish=resolve;}),openSession:async()=>({async replaceGenerated(){},async selectGenerated(handles){selected.push(handles);},close(){}})});
 controller.setSource({kind:'ifccad',name:'old.ifcx',files:source.files},true);const viewing=controller.show();const selection=controller.selectElement('/cad/d1/e1');while(!finish)await new Promise(resolve=>setImmediate(resolve));controller.setSource({kind:'ifccad',name:'new.ifcx',files:source.files},true);finish(output);await Promise.all([viewing,selection]);assert.deepEqual(selected,[]);assert.equal(controller.state.selectionKey,null);
 const showing=controller.show();await new Promise(resolve=>setImmediate(resolve));const a=controller.selectElement('/cad/d1/e1'),b=controller.selectElement('/cad/d1/e2');finish(output);await Promise.all([showing,a,b]);assert.deepEqual(selected.at(-1),['B']);assert.equal(controller.state.selectionStatus,'selected');
});
function setup(openExport=async()=>result){
 const calls=[];const session={async openOriginal(...args){calls.push(['original',...args]);},async replaceGenerated(...args){calls.push(['generated',...args]);},close(){calls.push(['close']);}};
 return {calls,controller:createCadPreviewController({openExport,openSession:async()=>session})};
}
test('changing roundtrip tolerance keeps the original CAD input tolerance intact',async()=>{
 const requests=[];const {controller}=setup(async request=>{requests.push(request);return result;});
 const input={tolerance:{mode:'exact'}},output={tolerance:{mode:'custom',value:1e-9,unit:'drawing'}};
 controller.setSource({...source,conversionOptions:input,exportConversionOptions:output},true);await controller.show();
 assert.deepEqual(requests[0].conversionOptions,input);assert.deepEqual(requests[0].exportConversionOptions,output);
 await controller.select({conversionOptions:{tolerance:{mode:'default'}}});assert.deepEqual(requests[1].conversionOptions,input);assert.deepEqual(requests[1].exportConversionOptions,{tolerance:{mode:'default'}});
});
test('background preparation validates output without starting the CAD viewer',async()=>{
 let exports=0;const {calls,controller}=setup(async()=>{exports++;return result;});controller.setSource(source,true);
 assert.equal(typeof controller.prepare,'function');await controller.prepare();assert.equal(controller.state.visible,false);assert.equal(controller.state.download,result.export.download);assert.deepEqual(calls,[]);
 await controller.show();assert.equal(exports,1);assert.deepEqual(calls.map(c=>c[0]),['original','generated']);
});
test('opening or leaving Drawing shares the pending background export',async()=>{
 let finish,signal,exports=0;const {calls,controller}=setup((r,o)=>{exports++;signal=o.signal;return new Promise(resolve=>{finish=resolve;});});controller.setSource(source,true);
 assert.equal(typeof controller.prepare,'function');const background=controller.prepare();while(!finish)await new Promise(resolve=>setImmediate(resolve));
 const viewing=controller.show();controller.hide();assert.equal(signal.aborted,false);finish(result);await Promise.all([background,viewing]);await controller.show();
 assert.equal(exports,1);assert.equal(calls.filter(c=>c[0]==='generated').length,1);
});
test('replacing a source invalidates its unfinished background result',async()=>{
 let finish,signal;const {controller}=setup((r,o)=>{signal=o.signal;return new Promise(resolve=>{finish=resolve;});});controller.setSource(source,true);
 assert.equal(typeof controller.prepare,'function');const background=controller.prepare();while(!finish)await new Promise(resolve=>setImmediate(resolve));
 controller.setSource({...source,name:'new.dwg'},true);assert.equal(signal.aborted,true);finish(result);await background;assert.equal(controller.state.download,null);assert.equal(controller.state.result,null);
});
test('invalid native input is not exported in the background',async()=>{
 const {calls,controller}=setup(()=>{throw Error('must not export');});controller.setSource(source,false);
 assert.equal(typeof controller.prepare,'function');await controller.prepare();assert.deepEqual(calls,[]);assert.equal(controller.state.result,null);
});
test('explicit cancellation discards late background progress and output',async()=>{
 let finish,signal,progress;const {controller}=setup((r,o)=>{signal=o.signal;progress=o.onProgress;return new Promise(resolve=>{finish=resolve;});});controller.setSource(source,true);const pending=controller.prepare();while(!finish)await new Promise(resolve=>setImmediate(resolve));
 controller.cancel();assert.equal(signal.aborted,true);assert.equal(controller.state.phase,'cancelled');progress('exporting');finish(result);await pending;assert.equal(controller.state.phase,'cancelled');assert.equal(controller.state.result,null);assert.equal(controller.state.download,null);
});
test('new output settings replace a pending hidden job and ignore its late result',async()=>{
 let finish,signal;const requests=[];const {calls,controller}=setup((r,o)=>{requests.push(r);if(requests.length===1){signal=o.signal;return new Promise(resolve=>{finish=resolve;});}return {...result,export:{...result.export,download:{...result.export.download,format:r.export.format}}};});
 controller.setSource(source,true);const old=controller.prepare();while(!finish)await new Promise(resolve=>setImmediate(resolve));await controller.select({format:'dxf',conversionOptions:{tolerance:{mode:'exact'}}});assert.equal(signal.aborted,true);finish(result);await old;
 assert.equal(controller.state.download.format,'dxf');assert.equal(requests[1].exportConversionOptions.tolerance.mode,'exact');assert.deepEqual(calls,[]);
});
test('tolerance changes regenerate output without closing the edited CAD session',async()=>{
 const requests=[];const {calls,controller}=setup(async r=>{requests.push(r);return result;});
 controller.setSource(source,true);await controller.show();
 await controller.select({conversionOptions:{tolerance:{mode:'exact'}}});
 assert.equal(requests.length,2);assert.equal(requests[1].exportConversionOptions.tolerance.mode,'exact');
 assert.equal(calls.filter(c=>c[0]==='original').length,1);assert.equal(calls.some(c=>c[0]==='close'),false);
});
test('original CAD and a production export via OCDraw open as distinct documents',async()=>{
 let request;const {calls,controller}=setup(async value=>{request=value;return result;});
 controller.setSource(source,true);await controller.show();
 assert.equal(request.kind,'cad');assert.deepEqual(request.export,{format:'dwg',version:'AC1032'});
 assert.deepEqual(calls.map(c=>c[0]),['original','generated']);
 assert.match(calls[1][2],/via-ocdraw/);
 await controller.show();assert.equal(calls.length,2);
});
test('failed conversion keeps the original available without generating a substitute',async()=>{
 const {calls,controller}=setup(()=>{throw Error('must not export');});
 controller.setSource(source,false);await controller.show();
 assert.deepEqual(calls.map(c=>c[0]),['original']);assert.match(controller.state.error,/conversie/i);
});
test('an OCDraw source opens only generated CAD',async()=>{
 const {calls,controller}=setup();controller.setSource({...source,kind:'drawing',name:'test.ocdraw.json'},true);
 await controller.select({format:'dwg'});await controller.show();
 assert.deepEqual(calls.map(c=>c[0]),['generated']);
});
test('an IFCCAD source opens generated CAD with the IFCCAD roundtrip label',async()=>{
 const {calls,controller}=setup();controller.setSource({...source,kind:'ifccad',name:'hello.ifcx'},true);
 await controller.select({format:'dwg'});await controller.show();
 assert.deepEqual(calls.map(c=>c[0]),['generated']);assert.match(calls[0][2],/via-ifccad/);
});
test('changing source cancels export and ignores late results',async()=>{
 let finish,signal;const {calls,controller}=setup((r,o)=>{signal=o.signal;return new Promise(resolve=>{finish=resolve;});});
 controller.setSource(source,true);const pending=controller.show();
 while(!finish)await new Promise(resolve=>setImmediate(resolve));
 controller.clear();assert.equal(signal.aborted,true);finish(result);await pending;
 assert.deepEqual(calls.map(c=>c[0]),['original','close']);assert.equal(controller.state.download,null);
});
test('export failure is reported without opening generated bytes',async()=>{
 const {calls,controller}=setup(async()=>({failure:{message:'Cannot export'},validation:{strictAvailable:true}}));
 controller.setSource(source,true);await controller.show();assert.equal(controller.state.error,'Cannot export');assert.equal(calls.length,1);
});
test('missing viewer still allows export diagnostics and retry',async()=>{
 let attempts=0;const calls=[];
 const controller=createCadPreviewController({openExport:async()=>result,openSession:async()=>{if(++attempts===1)throw Error('Missing viewer');return {openOriginal:async()=>{},replaceGenerated:async()=>calls.push('generated'),close(){}};}});
 controller.setSource(source,true);await controller.show();assert.equal(controller.state.viewerError,'Missing viewer');assert.ok(controller.state.download);
 await controller.retryViewer();assert.deepEqual(calls,['generated']);assert.equal(controller.state.viewerError,'');
});
test('unvalidated, incomplete or mismatched output never reaches the viewer',async()=>{
 for(const output of [
  {...result,validation:{strictAvailable:false}},
  {...result,export:{...result.export,requestedVersion:'AC1027'}},
  {...result,export:{...result.export,download:{...result.export.download,byteLength:3}}},
  {...result,export:{...result.export,download:{...result.export.download,format:'dxf'}}}
 ]){
  const {calls,controller}=setup(async()=>output);controller.setSource(source,true);await controller.show();
  assert.deepEqual(calls.map(c=>c[0]),['original']);assert.ok(controller.state.error);assert.equal(controller.state.download,null);
 }
});
