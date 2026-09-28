import {defaultCadVersion,supportsCadVersion} from '../src/cad-formats.mjs';
export const limits=Object.freeze({files:1000,bytes:64*1024*1024,requestBytes:90*1024*1024,outputBytes:128*1024*1024,timeoutMs:120000,ttlMs:300000});
export function normalizeUploadPath(value){
 if(typeof value!=='string'||!value||value.length>1024||/[\\:%\x00-\x1f<>"|?*]/.test(value)||value.startsWith('/')||value.split('/').some(p=>!p||p==='.'||p==='..'||/[. ]$/.test(p)||/^(con|prn|aux|nul|com[0-9]|lpt[0-9])(?:\.|$)/i.test(p)))throw Error('Unsafe file path');
 return value;
}
export function validateUpload(request,cap=limits){
 if(!request||!['package','cad'].includes(request.kind)||!Array.isArray(request.files)||!request.files.length||request.files.length>cap.files)throw Error('Invalid file selection');
 const seen=new Set();let total=0;
 const files=request.files.map(f=>{
  const path=normalizeUploadPath(f.path),key=path.toLowerCase();if(seen.has(key))throw Error('Duplicate file path');seen.add(key);
  if(typeof f.base64!=='string'||f.base64.length%4||/[^A-Za-z0-9+/=]/.test(f.base64)||! /^[A-Za-z0-9+/]*={0,2}$/.test(f.base64))throw Error('Invalid file encoding');
  const bytes=Buffer.from(f.base64,'base64');total+=bytes.length;if(total>cap.bytes)throw Object.assign(Error('File size limit exceeded'),{status:413});
  return {path,bytes};
 });
 if(request.kind==='package'&&!files.some(f=>f.path==='package.ifcx.json'))throw Error('Select a folder containing package.ifcx.json');
 if(request.kind==='cad'&&(files.length!==1||! /\.(dxf|dwg)$/i.test(files[0].path)))throw Error('Select one DXF or DWG file');
 const exp=request.export;
 if(exp!==undefined&&(!exp||!['dxf','dwg','ifccad'].includes(exp.format)||(exp.format!=='ifccad'&&(typeof exp.drawing!=='string'||!exp.drawing||exp.drawing.length>1024||/[\x00-\x1f]/.test(exp.drawing)))))throw Error('Invalid export selection');
 if(exp&&(exp.format==='ifccad'?exp.version!==undefined:!supportsCadVersion(exp.version??defaultCadVersion)))throw Error('Invalid CAD version');
 return {kind:request.kind,name:String(request.name||files[0].path).slice(0,256),files,...(exp?{export:exp.format==='ifccad'?{format:'ifccad',drawing:''}:{format:exp.format,drawing:exp.drawing,version:exp.version??defaultCadVersion}}:{})};
}
