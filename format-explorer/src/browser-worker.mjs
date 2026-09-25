import {encodeBase64} from './browser-client.mjs';
import {validPackagePath,zipBrowserFiles} from './browser-zip.mjs';

function validatedFiles(request){
 if(!['cad','package'].includes(request.kind)||!Array.isArray(request.files)||!request.files.length||request.files.length>1000)throw Error('Invalid file selection');
 let total=0;const seen=new Set();
 for(const file of request.files){
  if(!validPackagePath(file.path))throw Error('Unsafe file path');
  const key=file.path.toLowerCase();if(seen.has(key))throw Error('Duplicate file path');seen.add(key);
  if(!(file.bytes instanceof ArrayBuffer))throw Error('File bytes are missing');
  total+=file.bytes.byteLength;if(total>64*1024*1024)throw Error('File size limit exceeded');
 }
 if(request.kind==='cad'&&(request.files.length!==1||! /\.(dxf|dwg)$/i.test(request.files[0].path)))throw Error('Select one DXF or DWG file');
 if(request.kind==='package'&&!request.files.some(file=>file.path==='package.ifcx.json'))throw Error('Select a folder containing package.ifcx.json');
 return request.files.map(file=>({path:file.path,bytes:new Uint8Array(file.bytes)}));
}

/** The worker's synchronous core; exported so transport decisions can be tested. */
export function processBrowserRequest(request,wasm,onProgress=()=>{}){
 const selected=validatedFiles(request);
 const operation=request.export;
 let files=selected,opening=null;
 if(request.kind==='cad'){
  onProgress('converting');
  const format=/\.dwg$/i.test(selected[0].path)?'dwg':'dxf';
  opening=JSON.parse(wasm.open_cad(request.name,format,selected[0].bytes,new Date().toISOString()));
  if(!operation)return opening;
  if(opening.failure||!opening.validation?.strictAvailable)return opening;
  onProgress('converting');
  files=opening.presentation.documents.map(document=>({path:document.path,bytes:new TextEncoder().encode(document.text)}));
 }
 if(!operation){
  onProgress('validating');
  return JSON.parse(wasm.open_package(request.name,files.map(file=>file.path),files.map(file=>file.bytes)));
 }
 if(!['dxf','dwg','ifccad'].includes(operation.format))throw Error('Invalid export format');
 onProgress('exporting');
 const result=JSON.parse(wasm.export_package(request.name,files.map(file=>file.path),files.map(file=>file.bytes),operation.drawing||'',operation.format,operation.version||'AC1032'));
 if(opening){result.source=opening.source;result.reader=opening.reader;result.conversion=opening.conversion;}
 if(operation.format==='ifccad'&&!result.failure&&result.validation?.strictAvailable&&result.export?.packageReady){
  onProgress('packaging');
  const archive=zipBrowserFiles(files);
  result.export.fileCount=files.length;
  result.export.download={format:'ifccad',byteLength:archive.length,base64:encodeBase64(archive)};
 }
 return result;
}

let wasmPromise;
async function loadWasm(){
 wasmPromise??=import('./wasm/ifccad_browser.js').then(async module=>{await module.default(new URL('./wasm/ifccad_browser_bg.wasm',import.meta.url));return module;});
 return wasmPromise;
}
if(typeof self!=='undefined'&&typeof self.postMessage==='function'){
 self.onmessage=async event=>{
  try{
   self.postMessage({type:'progress',phase:'preparing'});
   const wasm=await loadWasm();
   const result=processBrowserRequest(event.data.request,wasm,phase=>self.postMessage({type:'progress',phase}));
   self.postMessage({type:'result',result});
  }catch(error){self.postMessage({type:'error',message:error.message||String(error)});}
 };
}
