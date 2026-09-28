import {ocsChannel,createDocumentSession,createOcsControl} from './ocs-messages.mjs';

async function waitForOcs(host){
 for(let attempt=0;attempt<2400;attempt++){
  if(host.wasmBindings?.ocs_control_submit&&host.wasmBindings?.ocs_control_take)return host.wasmBindings;
  await new Promise(resolve=>setTimeout(resolve,50));
 }
 throw Error('Open CAD Studio did not initialize');
}

export function connectOcsFrame(host,{waitForApi=waitForOcs,sessionFactory=api=>createDocumentSession(createOcsControl(api))}={}){
 let token=null,session=null;
 const reply=message=>host.parent.postMessage({channel:ocsChannel,token,...message},host.location.origin);
 async function receive(event){
  if(event.origin!==host.location.origin||event.source!==host.parent)return;
  const message=event.data;
  if(message?.channel!==ocsChannel)return;
  if(message.op==='init'){
   if(token!==null)return;
   token=message.token;
   try{
    const api=await waitForApi(host),control=createOcsControl(api),state=await control({op:'state'});
    if(state.modal){const dismissed=await control({op:'action',name:'close_modal',document_id:state.document_id});if(!dismissed.ok)throw Error(dismissed.error||'Startup dialog could not be closed');}
    session=sessionFactory(api);reply({status:'ready'});
   }catch(error){reply({status:'error',message:error.message});}
   return;
  }
  if(!session||message.token!==token||!['open','replace'].includes(message.op)||!['original','generated'].includes(message.role))return;
  try{
   const documentId=message.op==='open'&&message.role==='original'
    ?await session.openOriginal(message.base64,message.name)
    :message.op==='replace'&&message.role==='generated'
     ?await session.replaceGenerated(message.base64,message.name)
     :null;
   if(documentId===null)return;
   reply({id:message.id,role:message.role,status:'opened',documentId});
  }catch(error){reply({id:message.id,role:message.role,status:'error',message:error.message});}
 }
 host.addEventListener('message',receive);
 return {disconnect(){host.removeEventListener('message',receive);}};
}

if(typeof window!=='undefined'&&window.parent!==window)connectOcsFrame(window);
