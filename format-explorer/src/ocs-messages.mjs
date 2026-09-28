export const ocsChannel='ifccad-ocs';
export function createOcsSession(iframe,token,{host=window,origin=location.origin,timeoutMs=120000}={}){
 let closed=false,serial=0,queue=Promise.resolve(),readyResolve,readyReject;
 const pending=new Map();
 const ready=new Promise((resolve,reject)=>{readyResolve=resolve;readyReject=reject;});
 ready.catch(()=>{});
 const readyTimer=setTimeout(()=>readyReject(Error('Open CAD Studio did not start in time')),timeoutMs);
 const onLoad=()=>{if(!closed)iframe.contentWindow.postMessage({channel:ocsChannel,token,op:'init'},origin);};
 const onMessage=event=>{
  if(!isCurrentOcsMessage(event,iframe,token,origin)||closed)return;
  const message=event.data;
  if(message.status==='ready'){clearTimeout(readyTimer);readyResolve();return;}
  if(message.status==='error'&&!message.id){clearTimeout(readyTimer);readyReject(Error(message.message||'Open CAD Studio could not start'));return;}
  const item=pending.get(message.id);if(!item)return;
  pending.delete(message.id);clearTimeout(item.timer);
  if(message.status==='error')item.reject(Error(message.message||'Open CAD Studio could not open the drawing'));
  else item.resolve(message);
 };
 iframe.addEventListener('load',onLoad);host.addEventListener('message',onMessage);
 function send(op,role,base64,name){
  const run=async()=>{
   await ready;
   if(closed)throw Error('Viewer session closed');
   const id=String(++serial);
   return new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>{pending.delete(id);reject(Error('Open CAD Studio did not answer in time'));},timeoutMs);
    pending.set(id,{resolve,reject,timer});
    iframe.contentWindow.postMessage({channel:ocsChannel,token,id,op,role,base64,name},origin);
   });
  };
  const next=queue.then(run);queue=next.catch(()=>{});return next;
 }
 return {
  ready,
  openOriginal:(base64,name)=>send('open','original',base64,name),
  replaceGenerated:(base64,name)=>send('replace','generated',base64,name),
  close(){closed=true;clearTimeout(readyTimer);iframe.removeEventListener('load',onLoad);host.removeEventListener('message',onMessage);readyReject(Error('Viewer session closed'));for(const item of pending.values()){clearTimeout(item.timer);item.reject(Error('Viewer session closed'));}pending.clear();}
 };
}
export function createOcsControl(api,{delay=ms=>new Promise(resolve=>setTimeout(resolve,ms)),maxPolls=600,maxBusyRetries=20}={}){
 let serial=0;
 async function raw(request){
  const ticket=api.ocs_control_submit(JSON.stringify(request));
  for(let i=0;i<maxPolls;i++){
   const reply=api.ocs_control_take(ticket);
   if(reply)return JSON.parse(reply);
   await delay(50);
  }
  throw Error('Open CAD Studio did not answer in time');
 }
 return async function control(request){
  if(['state','operation'].includes(request.op))return raw(request);
  for(let attempt=0;attempt<=maxBusyRetries;attempt++){
   const state=await raw({op:'state'});
   if(!state.ok)throw Error(state.error||'Open CAD Studio state unavailable');
   const requestId='ifccad-'+Date.now()+'-'+(++serial);
   const payload={...request,request_id:requestId,revision:state.revision};
   if(request.op!=='open'&&payload.document_id===undefined)payload.document_id=state.document_id;
   let reply=await raw(payload);
   if(reply.code==='busy'&&attempt<maxBusyRetries){await delay(100);continue;}
   for(let i=0;['accepted','running'].includes(reply.status)&&i<maxPolls;i++){
    await delay(50);
    reply=await raw({op:'operation',request_id:requestId});
   }
   if(['accepted','running'].includes(reply.status))throw Error('Open CAD Studio operation timed out');
   return reply;
  }
  throw Error('Open CAD Studio remained busy');
 };
}
export function isCurrentOcsMessage(event,iframe,token,origin){
 const message=event?.data;
 return event?.origin===origin&&event?.source===iframe?.contentWindow&&message?.channel===ocsChannel&&message?.token===token&&['ready','opened','error'].includes(message.status);
}
export function createDocumentSession(control){
 let originalId=null,generatedId=null,queue=Promise.resolve();
 const schedule=operation=>{const next=queue.then(operation);queue=next.catch(()=>{});return next;};
 async function open(base64,name){
  if(!base64||!name)throw Error('CAD bytes and name are required');
  const reply=await control({op:'open',name,data_base64:base64});
  if(!reply.ok)throw Error(reply.error||'Open CAD Studio could not open the drawing');
  const state=await control({op:'state'});
  if(!state.ok||!state.documents?.some(document=>document.id===state.document_id))throw Error('Open CAD Studio did not report the opened document');
  return state.document_id;
 }
 async function closeGenerated(){
  if(generatedId===null)return;
  let state=await control({op:'state'});
  const generated=state.documents?.find(document=>document.id===generatedId);
  if(!generated){generatedId=null;return;}
  // Open CAD Studio guards closing an edited document with a save/discard
  // dialog. Keep that tab, including the user's edits, and open the next
  // roundtrip alongside it instead of interrupting the control operation.
  if(generated.dirty){generatedId=null;return;}
  if(state.document_id!==generatedId){const reply=await control({op:'activate',document_id:generatedId});if(!reply.ok)throw Error(reply.error||'Cannot activate generated drawing');}
  state=await control({op:'state'});
  const reply=await control({op:'action',name:'close_document',document_id:generatedId,revision:state.revision});
  if(!reply.ok)throw Error(reply.error||'Cannot close generated drawing');
  state=await control({op:'state'});
  if(state.documents?.some(document=>document.id===generatedId))throw Error('Generated drawing remained open');
  generatedId=null;
 }
 return {
  openOriginal:(base64,name)=>schedule(async()=>{if(originalId!==null)throw Error('Original drawing already open');originalId=await open(base64,name);return originalId;}),
  replaceGenerated:(base64,name)=>schedule(async()=>{await closeGenerated();generatedId=await open(base64,name);return generatedId;}),
  ids:()=>({originalId,generatedId})
 };
}
