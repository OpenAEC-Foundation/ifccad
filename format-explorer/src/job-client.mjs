import {parsePresentationJson} from './bundle.mjs';
export function createJobClient({fetchImpl=fetch,delay=ms=>new Promise(r=>setTimeout(r,ms))}={}){
 async function json(url,options){const response=await fetchImpl(url,options);const value=parsePresentationJson(await response.text());if(!response.ok)throw Error(value.error||'Request failed');return value;}
 return {capabilities:()=>json('./api/capabilities'),async open(request,{signal,onProgress}={}){
  let id,token;
  try{
   // Do not abort the create response: once its ID arrives we can always cancel the server job.
   ({id,token}=await json('./api/jobs',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(request)}));
   while(!signal?.aborted){const job=await json('./api/jobs/'+id,{signal,headers:{Authorization:'Bearer '+token}});onProgress?.(job.phase);
    if(job.status==='complete')return job.result;if(job.status==='failed')throw Error(job.error);if(job.status==='cancelled')throw new DOMException('Cancelled','AbortError');await delay(350);
   }
   throw new DOMException('Cancelled','AbortError');
  }finally{if(id)await fetchImpl('./api/jobs/'+id,{method:'DELETE',headers:{Authorization:'Bearer '+token}}).catch(()=>{});}
 }};
}
