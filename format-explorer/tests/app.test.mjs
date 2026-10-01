import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
import {decodeBase64} from '../src/browser-client.mjs';

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
