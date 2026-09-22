import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {randomUUID,randomBytes,timingSafeEqual} from 'node:crypto';
import {limits,validateUpload} from './upload-paths.mjs';
import {runWorker,workerAvailable} from './worker.mjs';
import {packageZip} from './package-zip.mjs';
export function createJobManager({worker=runWorker,tempRoot=tmpdir(),cap=limits,maxRetainedJobs=3}={}){
 const jobs=new Map();let active=null,closed=false;
 async function execute(job,upload){
  let root;
  try{
   root=await mkdtemp(path.join(tempRoot,'ifccad-viewer-'));const input=path.join(root,'input');await mkdir(input);
   for(const f of upload.files){if(job.controller.signal.aborted)throw Error('Job cancelled');const target=path.resolve(input,f.path);if(!target.startsWith(input+path.sep))throw Error('Unsafe file path');await mkdir(path.dirname(target),{recursive:true});await writeFile(target,f.bytes,{flag:'wx'});}
   const result=await worker({kind:upload.kind,export:upload.export,input:upload.kind==='cad'?path.join(input,upload.files[0].path):input,output:path.join(root,'output'),signal:job.controller.signal,cap,onProgress:phase=>{job.phase=phase;}});
   if(!job.controller.signal.aborted&&upload.export?.format==='ifccad'&&!result.failure&&result.export?.packageReady&&result.validation?.strictAvailable){
    job.phase='packaging';
    const archive=await packageZip(upload.kind==='cad'?path.join(root,'output'):input,{signal:job.controller.signal,cap});
    result.export.fileCount=archive.fileCount;
    result.export.download={format:'ifccad',byteLength:archive.bytes.length,base64:archive.bytes.toString('base64')};
   }
   if(!job.controller.signal.aborted){
    const scrub=value=>typeof value==='string'?value.replaceAll(root,'[temporary package]').replaceAll(root.replaceAll('\\','/'),'[temporary package]'):Array.isArray(value)?value.map(scrub):value&&typeof value==='object'?Object.fromEntries(Object.entries(value).map(([k,v])=>[k,scrub(v)])):value;
    for(const key of ['reader','failure','validation','export'])result[key]=scrub(result[key]);
    result.source.name=upload.name;job.result=result;
   }
  }catch(e){if(!job.controller.signal.aborted){job.error=root?e.message.replaceAll(root,'[temporary package]'):e.message;}}
  finally{
   if(root){try{const resolved=path.resolve(root);if(path.dirname(resolved)!==path.resolve(tempRoot)||!path.basename(resolved).startsWith('ifccad-viewer-'))throw Error('Unsafe cleanup target');await rm(resolved,{recursive:true,force:true,maxRetries:3,retryDelay:100});}catch{job.error='Temporary file cleanup failed';job.result=undefined;}}
   if(active===job.id)active=null;
   if(!job.controller.signal.aborted)job.status=job.error?'failed':'complete';else jobs.delete(job.id);
   job.expires=Date.now()+cap.ttlMs;
  }
 }
 const expire=()=>{for(const [id,j]of jobs)if(j.expires&&j.expires<Date.now())jobs.delete(id);};
 const timer=setInterval(expire,Math.min(cap.ttlMs,30000));timer.unref();
 return {
  get busy(){expire();return Boolean(active)||jobs.size>=maxRetainedJobs;},
  create(request){if(closed)throw Error('Service closed');expire();if(active)throw Object.assign(Error('Another file is being processed'),{status:409});if(jobs.size>=maxRetainedJobs)throw Object.assign(Error('Service busy; please try again shortly'),{status:429});const upload=validateUpload(request,cap);const id=randomUUID(),job={id,status:'running',phase:'reading',controller:new AbortController()};jobs.set(id,job);active=id;job.done=execute(job,upload);return {id};},
  get(id){expire();const j=jobs.get(id);return j?{id,status:j.status,phase:j.phase,result:j.result,error:j.error}:null;},
  cancel(id){const j=jobs.get(id);if(!j)return false;j.controller.abort();j.status='cancelled';j.result=undefined;if(active!==id)jobs.delete(id);return true;},
  async close(){closed=true;clearInterval(timer);for(const j of jobs.values())j.controller.abort();await Promise.allSettled([...jobs.values()].map(j=>j.done));jobs.clear();}
 };
}
export function createJobHandler({manager=createJobManager(),available=workerAvailable,publicOrigin}={}){
 if(publicOrigin){const origin=new URL(publicOrigin);if(origin.protocol!=='https:'||origin.origin!==publicOrigin)throw Error('PUBLIC_ORIGIN must be an exact HTTPS origin');}
 const access=new Map();let uploading=false;
 const send=(res,status,data)=>{res.writeHead(status,{'Content-Type':'application/json','Cache-Control':'no-store','X-Content-Type-Options':'nosniff'});res.end(JSON.stringify(data));};
 return {manager,async handle(req,res){
  if(!req.url.startsWith('/api/'))return false;
  try{
   const expected=publicOrigin?new URL(publicOrigin).host:'127.0.0.1:'+req.socket.localPort;
   if(req.headers.host!==expected&&(publicOrigin||req.headers.host!=='localhost:'+req.socket.localPort))throw Object.assign(Error('Invalid host'),{status:403});
   const origin=publicOrigin||'http://'+req.headers.host;
   if((req.headers.origin&&req.headers.origin!==origin)||(!['GET','HEAD'].includes(req.method)&&req.headers.origin!==origin))throw Object.assign(Error('Invalid origin'),{status:403});
   if(req.method==='GET'&&req.url==='/api/capabilities'){send(res,200,{available:await available(),processing:publicOrigin?'server':'local',limits,formats:['package','dxf','dwg']});return true;}
   if(req.method==='POST'&&req.url==='/api/jobs'){
    if(!await available())throw Object.assign(Error('Build the local reader: cargo build -p ifccad-viewer'),{status:503});
    if(uploading||manager.busy)throw Object.assign(Error('Service busy; please try again shortly'),{status:429});
    if(!/^application\/json(?:;|$)/i.test(req.headers['content-type']||''))throw Object.assign(Error('Expected application/json'),{status:415});
    if(Number(req.headers['content-length'])>limits.requestBytes)throw Object.assign(Error('Upload size limit exceeded'),{status:413});
    uploading=true;
    try{
     let length=0;const chunks=[];for await(const chunk of req){length+=chunk.length;if(length>limits.requestBytes)throw Object.assign(Error('Upload size limit exceeded'),{status:413});chunks.push(chunk);}
     const value=JSON.parse(Buffer.concat(chunks).toString('utf8')),created=manager.create(value),token=randomBytes(32).toString('hex');
     for(const [id,a]of access)if(a.expires<Date.now())access.delete(id);
     access.set(created.id,{token,expires:Date.now()+limits.timeoutMs+limits.ttlMs});send(res,202,{...created,token});return true;
    }finally{uploading=false;}
   }
   const match=req.url.match(/^\/api\/jobs\/([a-f0-9-]+)$/);
   if(match){const a=access.get(match[1]),supplied=(req.headers.authorization||'').replace(/^Bearer /,'');if(!a||a.expires<Date.now()||!/^[a-f0-9]{64}$/.test(supplied)||!timingSafeEqual(Buffer.from(supplied),Buffer.from(a.token)))throw Object.assign(Error('Job access denied'),{status:403});}
   if(match&&req.method==='GET'){const j=manager.get(match[1]);send(res,j?200:404,j||{error:'Job not found'});return true;}
   if(match&&req.method==='DELETE'){const found=manager.cancel(match[1]);access.delete(match[1]);send(res,found?200:404,{cancelled:found});return true;}
   send(res,404,{error:'Not found'});
  }catch(e){send(res,e.status||400,{error:e.message});}
  return true;
 }};
}
