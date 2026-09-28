import {createJobClient} from './job-client.mjs';

export const browserLimits=Object.freeze({files:1000,bytes:64*1024*1024});

export function encodeBase64(input){
 const bytes=input instanceof Uint8Array?input:new Uint8Array(input);
 let binary='';for(let index=0;index<bytes.length;index+=16384)binary+=String.fromCharCode(...bytes.subarray(index,index+16384));
 return btoa(binary);
}
export function decodeBase64(value){return Uint8Array.from(atob(value),character=>character.charCodeAt(0));}
function fileBytes(file){return file.bytes instanceof ArrayBuffer?new Uint8Array(file.bytes):file.bytes instanceof Uint8Array?file.bytes:decodeBase64(file.base64);}

export function createFileClient({workerFactory=()=>new Worker('./browser-worker.mjs',{type:'module'}),serverClient=createJobClient()}={}){
 return {
  capabilities(processing='browser'){
   return processing==='server'?serverClient.capabilities():Promise.resolve({available:true,processing:'browser',limits:browserLimits});
  },
  open(request,{signal,onProgress}={}){
   if(request.processing==='server'){
    const files=request.files.map(file=>({path:file.path,base64:file.base64??encodeBase64(fileBytes(file))}));
    const {processing,...rest}=request;
    return serverClient.open({...rest,files},{signal,onProgress});
   }
   if(signal?.aborted)return Promise.reject(new DOMException('Cancelled','AbortError'));
   return new Promise((resolve,reject)=>{
    let worker,finished=false;
    const finish=(error,result)=>{if(finished)return;finished=true;signal?.removeEventListener('abort',abort);worker?.terminate();error?reject(error):resolve(result);};
    const abort=()=>finish(new DOMException('Cancelled','AbortError'));
    try{
     worker=workerFactory();
     worker.onmessage=event=>{const message=event.data;if(message.type==='progress')onProgress?.(message.phase);else if(message.type==='result')finish(null,message.result);else if(message.type==='error')finish(Error(message.message));};
     worker.onerror=event=>finish(Error(event.message||'Browser processor failed'));
     signal?.addEventListener('abort',abort,{once:true});
     const files=request.files.map(file=>({path:file.path,bytes:fileBytes(file).slice().buffer}));
     const {processing,...rest}=request;
     worker.postMessage({request:{...rest,files}},files.map(file=>file.bytes));
    }catch(error){finish(error);}
   });
  }
 };
}
