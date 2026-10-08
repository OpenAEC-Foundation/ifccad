import {parsePresentationJson} from './presentation-json.mjs';
import {encodeBase64,decodeBase64} from './browser-client.mjs';
import {supportsCadVersion,defaultCadVersion} from './cad-formats.mjs';
import {normalizeTolerance} from './conversion-settings.mjs';
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
 const options=request.conversionOptions||request.preserveSplines===true?{tolerance:normalizeTolerance(request.conversionOptions?.tolerance??{mode:"default"}),...(request.preserveSplines===true?{preserveSplines:true}:{})}:undefined;
 const exportOptions=request.exportConversionOptions?{tolerance:normalizeTolerance(request.exportConversionOptions.tolerance??{mode:'default'})}:options;
 const invoke=(name,args,config=options)=>{
  const capabilities=typeof wasm.conversion_capabilities==='function'?parsePresentationJson(wasm.conversion_capabilities()):{};
  if(typeof wasm.conversion_capabilities==='function'&&(config?.tolerance.mode??'default')==='default'&&capabilities[drawingFormat]?.coordinateToleranceDefault!==true)throw Error('Coordinate default tolerance is unavailable in this processor');
  if(config?.tolerance.coordinateFallback!==undefined&&capabilities[drawingFormat]?.coordinateToleranceFallback!==true)throw Error('Coordinate tolerance fallback is unavailable in this processor');
  if(name==='convert_cad_to_ifccad'&&request.preserveSplines===true&&(capabilities.ifccad?.splinePreservation!==true||typeof wasm.convert_cad_to_ifccad_with_options!=='function'))throw Error('IFCCAD spline preservation is unavailable in this processor');
  if(config){if(typeof wasm[name+'_with_options']==='function')return wasm[name+'_with_options'](...args,JSON.stringify(config));if(config.tolerance.mode!=='default')throw Error('This converter build does not support adjustable tolerance options');}
  if(name==='convert_cad_to_drawing'&&request.preserveSplines===true){if(typeof wasm.convert_cad_to_drawing_with_preservation!=='function')throw Error('Spline preservation is unavailable in this processor');return wasm.convert_cad_to_drawing_with_preservation(...args,true);}
  return wasm[name](...args);
 };
 const drawingFormat=request.kind==='ifccad'?'ifccad':request.kind==='cad'?(request.drawingFormat??'ocdraw'):'ocdraw';
 if(!['ocdraw','ifccad'].includes(drawingFormat))throw Error('Invalid drawing format');
 if(operation&&(![drawingFormat,'dxf','dwg'].includes(operation.format)||(['dxf','dwg'].includes(operation.format)&&!supportsCadVersion(operation.version??defaultCadVersion))))throw Error('Invalid export selection');
 onProgress(request.kind==='cad'?'converting':'validating');
 const opening=parsePresentationJson(request.kind==='cad'
  ?(drawingFormat==='ifccad'?invoke('convert_cad_to_ifccad',[request.name,/\.dwg$/i.test(file.path)?'dwg':'dxf',file.bytes,new Date().toISOString()]):invoke('convert_cad_to_drawing',[request.name,/\.dwg$/i.test(file.path)?'dwg':'dxf',file.bytes]))
  :(drawingFormat==='ifccad'?wasm.open_ifccad(request.name,file.bytes):wasm.open_drawing(request.name,file.bytes)));
 if(request.kind!=='cad')opening.nativeSourceText=new TextDecoder().decode(file.bytes);
 else if(opening.export?.download?.base64)opening.nativeSourceText=new TextDecoder().decode(decodeBase64(opening.export.download.base64));
 opening.conversionCapabilities=typeof wasm.conversion_capabilities==='function'?parsePresentationJson(wasm.conversion_capabilities()):{ocdraw:{adjustableTolerance:typeof wasm.export_drawing_with_options==='function'},ifccad:{adjustableTolerance:false}};
 if(!operation||opening.failure||!opening.validation?.strictAvailable)return opening;
 if(operation.format===drawingFormat){
  if(request.kind!=='cad')opening.export={format:drawingFormat,download:{format:drawingFormat,fileName:file.path.split('/').at(-1),byteLength:file.bytes.length,base64:encodeBase64(file.bytes)}};
  return opening;
 }
 const drawing=request.kind==='cad'?decodeBase64(opening.export.download.base64):file.bytes;
 onProgress('exporting');
 const result=parsePresentationJson(drawingFormat==='ifccad'?invoke('export_ifccad',[request.name,drawing,operation.format,operation.version??defaultCadVersion],exportOptions):invoke('export_drawing',[request.name,drawing,operation.format,operation.version??defaultCadVersion],exportOptions));
 result.nativeSourceText=opening.nativeSourceText;
 result.conversionCapabilities=opening.conversionCapabilities;
 if(!result.presentation)result.presentation=opening.presentation;
 if(!result.validation)result.validation=opening.validation;
 result.source=opening.source;
 if(request.kind==='cad'){result.source=opening.source;result.reader=opening.reader;result.conversion={...opening.conversion,restoration:result.conversion};}
 return result;
}

let wasmPromise;
async function loadWasm(){wasmPromise??=import('./wasm/browser.js').then(async module=>{await module.default(new URL('./wasm/browser_bg.wasm',import.meta.url));return module;});return wasmPromise;}
if(typeof self!=='undefined'&&typeof self.postMessage==='function')self.onmessage=async event=>{try{self.postMessage({type:'progress',phase:'preparing'});const result=processBrowserRequest(event.data.request,await loadWasm(),phase=>self.postMessage({type:'progress',phase}));self.postMessage({type:'result',result});}catch(e){self.postMessage({type:'error',message:e.message||String(e)});}};
