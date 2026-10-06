import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import {decodeBase64} from '../src/browser-client.mjs';

test('spline preservation is visible only for OCDraw CAD input and is explicitly forwarded',async()=>{
 const elements=new Map(),requests=[];
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true,value:'',checked:false});return elements.get(id);};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{document:{getElementById:element},createFileClient:()=>({async open(request){requests.push(request);return {validation:{strictAvailable:true},presentation:{opaqueEntityCount:request.preserveSplines?1:0}};}}),initializeCadPreview:()=>({clear(){},setSource(){}}),AbortController});
 element('file').files=[{name:'spline.dxf',size:1,arrayBuffer:async()=>new ArrayBuffer(1)}];
 element('drawing-format').value='ocdraw';await element('file').onchange();
 assert.equal(element('preservation-control').hidden,false);
 element('preserve-splines').checked=true;
 await element('open').onclick();assert.equal(requests[0].preserveSplines,true);
 assert.match(element('status').textContent,/brongegevens/);
 element('drawing-format').value='ifccad';await element('drawing-format').onchange();
 assert.equal(element('preservation-control').hidden,true);assert.equal(requests[1].preserveSplines,false);
});

test('a file read failure is visible and cannot export the previously opened drawing',async()=>{
 const elements=new Map();
 const element=id=>{if(!elements.has(id))elements.set(id,{disabled:false,textContent:'',hidden:true});return elements.get(id);};
 let calls=0;
 const client={async open(){calls++;return {validation:{strictAvailable:true}};}};
 const code=(await readFile(new URL('../src/app.mjs',import.meta.url),'utf8')).replace(/^import .*;\r?\n/gm,'');
 vm.runInNewContext(code,{document:{getElementById:element},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){}}),AbortController});
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
 vm.runInNewContext(code,{document:{getElementById:element,createElement(){const link={click(){this.clicked=true;}};links.push(link);return link;}},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){}}),AbortController,decodeBase64,Blob,URL:{createObjectURL(blob){blobs.push(blob);return 'blob:test';},revokeObjectURL(){}},setTimeout(){}});
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
 vm.runInNewContext(code,{document:{getElementById:element},createFileClient:()=>client,initializeCadPreview:()=>({clear(){},setSource(){}}),AbortController});
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
 vm.runInNewContext(code,{document:{getElementById:element},createFileClient:()=>({async open(request){requests.push(request);return {validation:{strictAvailable:true}};}}),initializeCadPreview:()=>({clear(){},setSource(){}}),AbortController});
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
