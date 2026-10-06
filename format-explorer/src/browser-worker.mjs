import {parsePresentationJson} from './presentation-json.mjs';
import {encodeBase64,decodeBase64} from './browser-client.mjs';
import {supportsCadVersion,defaultCadVersion} from './cad-formats.mjs';
function selectedFile(request){
 if(!request||!['cad','drawing','ifccad'].includes(request.kind)||!Array.isArray(request.files)||request.files.length!==1)throw Error('Select one drawing');
 const file=request.files[0];
 if(typeof file.path!=='string'||!file.path||/[\\:%\x00-\x1f<>"|?*]/.test(file.path)||file.path.startsWith('/')||file.path.split('/').some(p=>!p||p==='.'||p==='..'))throw Error('Unsafe file path');
 if(!(file.bytes instanceof ArrayBuffer))throw Error('File bytes are missing');
 if(file.bytes.byteLength>64*1024*1024)throw Error('File size limit exceeded');
 if(request.kind==='cad'&&!/\.(dxf|dwg)$/i.test(file.path))throw Error('Select one DXF or DWG file');
 if(request.kind==='drawing'&&!/\.ocdraw(?:\.json)?$/i.test(file.path))throw Error('Select an OCDraw file');
 if(request.kind==='ifccad'&&!/\.ifcx(?:\.json)?$/i.test(file.path))throw Error('Select an IFCCAD file');
 return {...file,bytes:new Uint8Array(file.bytes)};
}
export function processBrowserRequest(request,wasm,onProgress=()=>{}){
 const file=selectedFile(request),operation=request.export;
 const drawingFormat=request.kind==='ifccad'?'ifccad':request.kind==='cad'?(request.drawingFormat??'ocdraw'):'ocdraw';
 if(!['ocdraw','ifccad'].includes(drawingFormat))throw Error('Invalid drawing format');
 if(operation&&(![drawingFormat,'dxf','dwg'].includes(operation.format)||(['dxf','dwg'].includes(operation.format)&&!supportsCadVersion(operation.version??defaultCadVersion))))throw Error('Invalid export selection');
 onProgress(request.kind==='cad'?'converting':'validating');
 const opening=parsePresentationJson(request.kind==='cad'
  ?(drawingFormat==='ifccad'?wasm.convert_cad_to_ifccad(request.name,/\.dwg$/i.test(file.path)?'dwg':'dxf',file.bytes,new Date().toISOString()):request.preserveSplines===true?wasm.convert_cad_to_drawing_with_preservation(request.name,/\.dwg$/i.test(file.path)?'dwg':'dxf',file.bytes,true):wasm.convert_cad_to_drawing(request.name,/\.dwg$/i.test(file.path)?'dwg':'dxf',file.bytes))
  :(drawingFormat==='ifccad'?wasm.open_ifccad(request.name,file.bytes):wasm.open_drawing(request.name,file.bytes)));
 if(!operation||opening.failure||!opening.validation?.strictAvailable)return opening;
 if(operation.format===drawingFormat){
  if(request.kind!=='cad')opening.export={format:drawingFormat,download:{format:drawingFormat,fileName:file.path.split('/').at(-1),byteLength:file.bytes.length,base64:encodeBase64(file.bytes)}};
  return opening;
 }
 const drawing=request.kind==='cad'?decodeBase64(opening.export.download.base64):file.bytes;
 onProgress('exporting');
 const result=parsePresentationJson(drawingFormat==='ifccad'?wasm.export_ifccad(request.name,drawing,operation.format,operation.version??defaultCadVersion):wasm.export_drawing(request.name,drawing,operation.format,operation.version??defaultCadVersion));
 if(request.kind==='cad'){result.source=opening.source;result.reader=opening.reader;result.conversion={...opening.conversion,restoration:result.conversion};}
 return result;
}

let wasmPromise;
async function loadWasm(){wasmPromise??=import('./wasm/browser.js').then(async module=>{await module.default(new URL('./wasm/browser_bg.wasm',import.meta.url));return module;});return wasmPromise;}
if(typeof self!=='undefined'&&typeof self.postMessage==='function')self.onmessage=async event=>{try{self.postMessage({type:'progress',phase:'preparing'});const result=processBrowserRequest(event.data.request,await loadWasm(),phase=>self.postMessage({type:'progress',phase}));self.postMessage({type:'result',result});}catch(e){self.postMessage({type:'error',message:e.message||String(e)});}};
