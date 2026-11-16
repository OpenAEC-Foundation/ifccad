import test from 'node:test';
import assert from 'node:assert/strict';
import {nativeExportRequest} from '../src/native-request.mjs';
import {processBrowserRequest} from '../src/browser-worker.mjs';
import {ifccadEncoding} from '../src/ifccad-encoding.mjs';

test('all exports use the already inspected CAD generation, including exact JSON bytes',()=>{
 let imports=0,received;
 const bytes=new TextEncoder().encode('private CAD bytes').buffer;
 const source={kind:'cad',name:'drawing.dxf',files:[{path:'drawing.dxf',bytes}]};
 const wasm={convert_cad_to_ifccad(){imports++;return JSON.stringify({validation:{strictAvailable:true},export:{download:{base64:'IHt9Cg=='}}});},open_ifccad(name,data){received=data;return JSON.stringify({validation:{strictAvailable:true}});}};
 const opened=processBrowserRequest(source,wasm);
 assert.equal(opened.nativeFile.fileName,'drawing.ifcx');assert.equal(opened.nativeFile.base64,'IHt9Cg==');
 const request=nativeExportRequest(source,opened,{format:'ifccad'});
 const saved=processBrowserRequest(request,wasm);
 assert.equal(imports,1);assert.equal(saved.export.download.base64,'IHt9Cg==');assert.equal(new TextDecoder().decode(received),' {}\n');
});

test('binary preview is labelled as a projection and binary downloads retain the original source request',()=>{
 const source={kind:'ifccad',name:'drawing.ifccad.cbor',files:[{path:'drawing.ifccad.cbor',bytes:new Uint8Array([1,2]).buffer}]};
 const opened=processBrowserRequest(source,{open_ifccad:()=>JSON.stringify({validation:{strictAvailable:true},nativeSourceText:'{}'})});
 assert.equal(opened.nativeFile.representation,'decoded');assert.equal(opened.nativeFile.fileName,'drawing.ifcx');
 const request=nativeExportRequest(source,opened,{format:'ifccad',encoding:'cbor'});
 assert.equal(request.files[0].bytes,source.files[0].bytes);
});

test('invalid snapshots cannot be exported as current native data',()=>{
 assert.throws(()=>nativeExportRequest({}, {validation:{strictAvailable:false}}, {format:'ifccad'}),/validated/);
});

test('an IFCCAD example keeps its filename and bytes when downloaded as JSON',()=>{
 const name='entities.ifccad';
 const source={kind:'ifccad',name,files:[{path:name,bytes:new TextEncoder().encode(' {}\n').buffer}]};
 const wasm={
  open_ifccad:()=>JSON.stringify({validation:{strictAvailable:true}}),
  export_ifccad_binary(exportName,data,encoding){
   assert.equal(exportName,'entities.ifcx');
   assert.equal(new TextDecoder().decode(data),' {}\n');
   assert.equal(encoding,'cbor');
   return JSON.stringify({export:{download:{fileName:'entities.ifccad.cbor'}}});
  },
 };
 assert.equal(ifccadEncoding(name),'ifcx');
 const opened=processBrowserRequest(source,wasm);
 assert.equal(opened.nativeFile.fileName,'entities.ifcx');
 const request=nativeExportRequest(source,opened,{format:'ifccad'});
 const saved=processBrowserRequest(request,wasm).export.download;
 assert.equal(saved.fileName,name);
 assert.equal(saved.base64,'IHt9Cg==');
 assert.equal(nativeExportRequest(source,opened,{format:'dxf'}).name,'entities.ifcx');
 const binary=nativeExportRequest(source,opened,{format:'ifccad',encoding:'cbor'});
 assert.equal(processBrowserRequest(binary,wasm).export.download.fileName,'entities.ifccad.cbor');
});
