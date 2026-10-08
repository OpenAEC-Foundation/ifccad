/** Read-only observation for the pinned OCS state API (which has no event export). */
export function createSelectionObserver({readState,getGeneratedId,emit,isFocused=()=>true,schedule=setTimeout,cancel=clearTimeout,interval=300}){
 let enabled=false,closed=false,held=0,version=0,timer=null,inFlight=false,last=null,lastLayout;
 const canonical=handles=>handles.map(handle=>handle.toUpperCase().replace(/^0+(?=[0-9A-F])/,'')).sort();
 const signature=(documentId,layout,handles)=>JSON.stringify([documentId,layout,canonical(handles)]);
 function clearTimer(){if(timer!==null){cancel(timer);timer=null;}}
 function next(delay=interval){if(enabled&&!closed&&!held&&!inFlight&&timer===null)timer=schedule(async()=>{timer=null;await sample();},delay);}
 async function sample(){
  const current=version,documentId=getGeneratedId();
  if(!enabled||closed||held||inFlight)return;
  if(documentId==null||!isFocused()){next();return;}
  inFlight=true;
  try{
   const state=await readState();
   if(current!==version||documentId!==getGeneratedId()||!enabled||closed||held||!isFocused())return;
   if(!state.ok||state.modal)return;
   if(state.document_id!==documentId){last=null;return;}
   if(!Array.isArray(state.selection))return;
   if(state.selection.length!==1){last=signature(documentId,state.layout,[]);lastLayout=state.layout;return;}
   if(state.selection.some(handle=>typeof handle!=='string'||!/^([0-9a-f]{1,16})$/i.test(handle)||BigInt('0x'+handle)===0n))return;
   const handles=canonical(state.selection),key=signature(documentId,state.layout,handles);lastLayout=state.layout;
   if(key!==last){last=key;emit({documentId,layout:state.layout,handles});}
  }catch{/* A transient state read must not interrupt editing or document loading. */}
  finally{inFlight=false;next();}
 }
 return {
  setEnabled(value){enabled=!!value;version++;clearTimer();next(0);},
  expect(documentId,handles,layout){version++;last=signature(documentId,layout??lastLayout,handles);if(layout!==undefined)lastLayout=layout;},
  pause(){held++;version++;clearTimer();},resume(){held=Math.max(0,held-1);next(0);},
  wake(){version++;last=null;clearTimer();next(0);},
  close(){closed=true;enabled=false;version++;clearTimer();}
 };
}
