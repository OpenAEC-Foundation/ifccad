import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {access} from 'node:fs/promises';
import {limits} from './upload-paths.mjs';
export const executable=process.env.IFCCAD_VIEWER_BIN||fileURLToPath(new URL('../../target/debug/ifccad-viewer'+(process.platform==='win32'?'.exe':''),import.meta.url));
export async function workerAvailable(){try{await access(executable);return true;}catch{return false;}}
export function runWorker({kind,input,output,signal,onProgress,cap=limits}){
 return new Promise((resolve,reject)=>{
  const child=spawn(executable,[kind,input,...(kind==='cad'?[output]:[])],{shell:false,windowsHide:true,stdio:['ignore','pipe','pipe']});
  let size=0,pending='',result,stderr='',failure;
  const stop=()=>{failure=Error('Job cancelled');child.kill();};signal?.addEventListener('abort',stop,{once:true});if(signal?.aborted)stop();
  const timer=setTimeout(()=>{failure=Error('Processing time limit exceeded');child.kill();},cap.timeoutMs);
  child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');
  child.stdout.on('data',chunk=>{
   size+=Buffer.byteLength(chunk);if(size>cap.outputBytes){failure=Error('Viewer output limit exceeded');child.kill();return;}
   pending+=chunk;let end;
   while((end=pending.indexOf('\n'))!==-1){const line=pending.slice(0,end).trim();pending=pending.slice(end+1);if(!line)continue;
    try{const event=JSON.parse(line);if(event.protocolVersion!==1)throw Error('Unsupported worker protocol');if(event.type==='progress')onProgress?.(event.phase);else if(event.type==='result')result=event.result;else throw Error('Invalid worker event');}
    catch(e){failure=e;child.kill();}
   }
  });
  child.stderr.on('data',chunk=>{stderr=(stderr+chunk).slice(-8192);});
  child.on('error',e=>{failure=e;});
  child.on('close',code=>{clearTimeout(timer);signal?.removeEventListener('abort',stop);if(failure)reject(failure);else if(code!==0||!result)reject(Error('Local reader failed'+(stderr?' (see local server log)':'')));else resolve(result);});
 });
}
