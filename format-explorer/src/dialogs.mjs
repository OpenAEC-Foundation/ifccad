export function initializeDialogs(){
 for(const name of ['settings','about']){const dialog=document.getElementById(name+'-dialog'),trigger=document.getElementById(name+'-open');let focus;
  trigger.onclick=()=>{focus=document.activeElement;dialog.showModal();};dialog.querySelector('[data-close-dialog]').onclick=()=>dialog.close();dialog.addEventListener('close',()=>focus?.focus());
 }
}
