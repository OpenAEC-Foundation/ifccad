import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import {decodeBase64} from '../src/browser-client.mjs';
import {translate} from '../src/i18n.mjs';
import {createCadPreviewController} from '../src/cad-preview.mjs';
const initializeWorkspace=()=>({t:(key,v)=>translate('nl',key,v),show(){},clear(){},selectTab(){},setStatus(){},setLoading(){},clearLoading(){}});
const conversionOptions=()=>({tolerance:{mode:'default'}});
const examples=[];
test('opening starts background roundtrip without waiting for it or opening Drawing',async()=>{
 const elements=new Map(),calls=[];let finish;
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};
 const preview=createCadPreviewController({openExport:request=>{calls.push(request);return new Promise(resolve=>{finish=resolve;});},openSession:()=>{throw Error('viewer must stay closed');}});
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(){return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>preview,AbortController});
 element('preview-format').value='dwg';element('preview-version').value='AC1027';element('file').files=[{name:'new.ifcx',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();
 assert.equal(preview.state.busy,true);assert.equal(element('open').disabled,false);assert.equal(preview.state.visible,false);assert.equal(calls[0].export.format,'dwg');assert.equal(calls[0].export.version,'AC1027');
 const pending=preview.prepare();finish({validation:{strictAvailable:true},export:{requestedVersion:'AC1027',download:{format:'dwg',base64:'AQI=',byteLength:2}}});await pending;assert.ok(preview.state.download);
});
test('opening and converting show actual progress and cancellation also stops pending reads',async()=>{
 const elements=new Map(),progress=[],requests=[];let resolveRead;
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};
 const workspace={...initializeWorkspace(),setLoading(value){progress.push(value);},clearLoading(){progress.push(null);}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace:()=>workspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(r,{onProgress}){requests.push(r);onProgress('converting');return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){},select(){return Promise.resolve();},cancel(){}}),AbortController});
 element('drawing-format').value='ocdraw';element('file').files=[{name:'slow.dxf',size:1,arrayBuffer:()=>new Promise(r=>{resolveRead=r;})}];
 const pending=element('open').onclick();assert.equal(progress.at(-1)?.phase,'loading');assert.equal(progress.at(-1)?.name,'slow.dxf');assert.equal(element('cancel').disabled,false);
 element('loading-cancel').onclick();assert.equal(progress.at(-1),null);assert.equal(element('open').disabled,false);resolveRead(new ArrayBuffer(1));await pending;assert.equal(requests.length,0);
 element('file').files=[{name:'drawing.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();
 assert.ok(progress.some(p=>p?.action==='convertingTo'&&p?.format==='OCDraw'));assert.ok(progress.some(p=>p?.phase==='converting'));assert.equal(progress.at(-1),null);
});
test('opening another file preserves the active workspace tab',async()=>{
 const elements=new Map(),switched=[];const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};
 const workspace={...initializeWorkspace(),state:{tab:'conversion'},selectTab(name){switched.push(name);this.state.tab=name;}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace:()=>workspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(){return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){},select(){return Promise.resolve();},cancel(){}}),AbortController});
 element('file').files=[{name:'new.ifcx',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();
 assert.deepEqual(switched,[]);assert.equal(workspace.state.tab,'conversion');
});

test('spline preservation is visible only for OCDraw CAD input and is explicitly forwarded',async()=>{
 const elements=new Map(),requests=[];
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true,value:'',checked:false});return elements.get(id);};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(request){requests.push(request);return {validation:{strictAvailable:true},presentation:{opaqueEntityCount:request.preserveSplines?1:0}};}}),initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 element('file').files=[{name:'spline.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 element('drawing-format').value='ocdraw';await element('file').onchange();
 assert.equal(element('preservation-control').hidden,false);
 element('preserve-splines').checked=true;
 await element('open').onclick();assert.equal(requests[0].preserveSplines,true);
 assert.match(element('status').textContent,/brongegevens/);
 element('drawing-format').value='ifccad';await element('drawing-format').onchange();
 assert.equal(element('preservation-control').hidden,true);await element('apply-settings').onclick();assert.equal(requests[1].preserveSplines,false);
});

test('a file read failure is visible and cannot export the previously opened drawing',async()=>{
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true});return elements.get(id);};
 let calls=0;
 const client={async open(){calls++;return {validation:{strictAvailable:true}};}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 element('file').files=[{name:'first.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 await element('open').onclick();
 assert.equal(element('export').disabled,false);
 element('file').files=[{name:'locked.dxf',size:1,arrayBuffer:async()=>{throw new Error('NotReadableError');}}];
 await element('open').onclick();
 assert.match(element('status').textContent,/bestand.*niet.*gelezen/i);
 assert.equal(element('export').disabled,true);
 assert.equal(element('open').disabled,false);
 assert.equal(element('drawing').hidden,true);
 await element('export').onclick();
 assert.equal(calls,1);
});

test('CAD download uses the shared preview choice while OCDraw download is independent',async()=>{
 const elements=new Map();const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true,value:''});return elements.get(id);};
 const requests=[],links=[],blobs=[];const client={async open(request){requests.push(request);return {validation:{strictAvailable:true},...(request.export?{export:{download:{base64:'AQI=',fileName:'drawing.'+request.export.format}}}:{})};}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element,createElement(){const link={click(){this.clicked=true;}};links.push(link);return link;}},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController,decodeBase64,Blob,URL:{createObjectURL(blob){blobs.push(blob);return 'blob:test';},revokeObjectURL(){}},setTimeout(){}});
 element('file').files=[{name:'drawing.ocdraw.json',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 await element('open').onclick();
 element('preview-format').value='dwg';element('preview-version').value='AC1027';
 await element('cad-download').onclick();
 await element('export').onclick();
 assert.deepEqual(JSON.parse(JSON.stringify(requests[1].export)),{format:'dwg',version:'AC1027'});
 assert.deepEqual(JSON.parse(JSON.stringify(requests[2].export)),{format:'ocdraw'});
 assert.deepEqual(links.map(link=>[link.download,link.clicked]),[['drawing.dwg',true],['drawing.ocdraw',true]]);
 for(const blob of blobs)assert.deepEqual([...new Uint8Array(await blob.arrayBuffer())],[1,2]);
});

test('the current drawing explorer opens IFCCAD and downloads its native IFCX format',async()=>{
 const elements=new Map(),requests=[];
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true,value:''});return elements.get(id);};
 const client={async open(request){requests.push(request);return {validation:{strictAvailable:true},presentation:{format:'ifccad'}};}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 element('file').files=[{name:'hello.ifcx',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 await element('open').onclick();await element('export').onclick();
 assert.equal(requests[0].kind,'ifccad');assert.equal(requests[1].export.format,'ifccad');
 assert.match(element('export').textContent,/IFCCAD/);assert.match(element('content-title').textContent,/IFCCAD/);
 element('file').files=[{name:'returned.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();assert.equal(requests[2].drawingFormat,'ifccad');
});

test('only a selected DWG or DXF offers conversion with its actual format name',async()=>{
 const elements=new Map(),requests=[];
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(request){requests.push(request);return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 assert.equal(element('drawing-format-control').hidden,true);
 for(const [name,label] of [['drawing.DWG','DWG omzetten naar'],['drawing.dxf','DXF omzetten naar'],['hello.ifcx',null],['hello.ifcx.json',null],['drawing.ocdraw',null],['drawing.ocdraw.json',null]]){
  element('file').files=[{name,size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
  await element('file').onchange();
  assert.equal(element('drawing-format-control').hidden,label===null,name);
  if(label)assert.equal(element('drawing-format-label').textContent,label);
  assert.equal(element('drawing-format').disabled,label===null);
  assert.equal(element('export').disabled,true);
  await element('open').onclick();
  assert.equal(element('drawing-format-control').hidden,label===null,name);
 }
 element('file').files=[];await element('file').onchange();
 assert.equal(element('drawing-format-control').hidden,true);
 assert.equal(element('export').disabled,true);
 assert.equal(requests.length,6);
});
test('opening a native file does not overwrite an explicitly chosen CAD route',async()=>{
 const elements=new Map(),requests=[];const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(r){requests.push(r);return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 element('drawing-format').value='ocdraw';element('file').files=[{name:'native.ifcx',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();
 element('file').files=[{name:'source.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();assert.equal(requests[1].drawingFormat,'ocdraw');
});
test('a late file read cannot replace a newer source',async()=>{
 const elements=new Map(),requests=[];const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:false,value:''});return elements.get(id);};let resolveRead;
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{initializeWorkspace,conversionOptions,examples,document:{getElementById:element},createFileClient:()=>({async open(r){requests.push(r);return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){},updateSource(){},cancel(){},select(){return Promise.resolve();}}),AbortController});
 element('file').files=[{name:'first.ifcx',size:1,arrayBuffer:()=>new Promise(r=>{resolveRead=r;})}];const pending=element('open').onclick();
 element('file').files=[{name:'second.ifcx',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];await element('open').onclick();resolveRead(new ArrayBuffer(1));await pending;
 assert.deepEqual(requests.map(r=>r.name),['second.ifcx']);
});
