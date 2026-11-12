export function initializeDialogs(beforeOpen={}){
 for(const name of ['settings','about','feedback','file-json']){const dialog=document.getElementById(name+'-dialog'),trigger=document.getElementById(name+'-open');let focus;
  trigger.onclick=()=>{focus=document.activeElement;beforeOpen[name]?.();dialog.showModal();};dialog.querySelector('[data-close-dialog]').onclick=()=>dialog.close();dialog.addEventListener('close',()=>focus?.focus());
 }
}
