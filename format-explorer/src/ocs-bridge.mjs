import {ocsChannel,createDocumentSession,createOcsControl} from './ocs-messages.mjs';
import {createSelectionObserver} from './ocs-selection.mjs';

async function waitForOcs(host){
 for(let attempt=0;attempt<2400;attempt++){
  if(host.wasmBindings?.ocs_control_submit&&host.wasmBindings?.ocs_control_take)return host.wasmBindings;
  await new Promise(resolve=>setTimeout(resolve,50));
 }
 throw Error('Open CAD Studio did not initialize');
}

export function connectOcsFrame(host,{waitForApi=waitForOcs,sessionFactory=api=>createDocumentSession(createOcsControl(api))}={}){
 let token=null,session=null,observer=null,disconnected=false;
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
    if(disconnected)return;
    if(state.modal){const dismissed=await control({op:'action',name:'close_modal',document_id:state.document_id});if(!dismissed.ok)throw Error(dismissed.error||'Startup dialog could not be closed');}
    session=sessionFactory(api);
    observer=createSelectionObserver({readState:()=>control({op:'state'}),getGeneratedId:()=>session?.ids?.()?.generatedId,isFocused:()=>!host.document?.hidden&&(host.document?.hasFocus?.()??true),emit:event=>reply({role:'generated',status:'selection-changed',...event})});
    reply({status:'ready'});
   }catch(error){reply({status:'error',message:error.message});}
   return;
  }
  if(!session||message.token!==token||disconnected)return;
  if(message.op==='observe'){observer.setEnabled(message.visible===true);return;}
  if(message.op==='disconnect'){disconnect();return;}
  if(!['open','replace','select'].includes(message.op)||!['original','generated'].includes(message.role))return;
  observer.pause();
  try{
   if(message.op==='select'){
    if(message.role!=='generated')return;
    const documentId=await session.selectGenerated(message.handles,message.layout,message.documentId);
    observer.expect(documentId,message.handles,message.layout);
    reply({id:message.id,role:'generated',status:'selected',documentId});return;
   }
   const documentId=message.op==='open'&&message.role==='original'
    ?await session.openOriginal(message.base64,message.name)
    :message.op==='replace'&&message.role==='generated'
     ?await session.replaceGenerated(message.base64,message.name)
     :null;
   if(documentId===null)return;
   reply({id:message.id,role:message.role,status:'opened',documentId});
  }catch(error){reply({id:message.id,role:message.role,status:'error',message:error.message});}
  finally{observer.resume();}
 }
 const wake=()=>observer?.wake();
 function disconnect(){disconnected=true;observer?.close();host.removeEventListener?.('message',receive);host.removeEventListener?.('focus',wake);host.removeEventListener?.('pointerup',wake);host.removeEventListener?.('keyup',wake);host.removeEventListener?.('pagehide',disconnect);}
 host.addEventListener('message',receive);
 for(const name of ['focus','pointerup','keyup'])host.addEventListener(name,wake);
 host.addEventListener('pagehide',disconnect);
 return {disconnect};
}

if(typeof window!=='undefined'&&window.parent!==window)connectOcsFrame(window);
