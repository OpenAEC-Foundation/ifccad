import {createFileClient} from './browser-client.mjs';
import {t,translateTree} from './i18n.mjs';

export function initializeOpening({onResult}){
 const $=id=>document.getElementById(id),client=createFileClient();
 const dialog=$('open-dialog'),status=$('open-status');
 const methods=[...dialog.querySelectorAll('input[name="open-processing"]')];
 const processing=()=>methods.find(method=>method.checked)?.value||'browser';
 let cap,controller,generation=0;

 async function refreshCapability(){
  const current=++generation;
  status.textContent=processing()==='server'?'Server controleren…':'Bestanden blijven in deze browser. Maximaal 64 MiB en 1000 bestanden.';
  $('choose-package').disabled=$('choose-cad').disabled=true;translateTree(dialog);
  try{
   const next=await client.capabilities(processing());
   if(current!==generation)return;
   cap=next;
   if(processing()==='server')status.textContent=next.available
    ?'Je bestanden worden via HTTPS naar de OpenAEC-server gestuurd voor verwerking. Uploads worden na verwerking verwijderd.'
    :'De bestandslezer op de server is tijdelijk niet beschikbaar.';
  }catch(error){if(current===generation){cap=null;status.textContent=error.message;}}
  if(current===generation){$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;translateTree(dialog);}
 }

 $('open-files').onclick=()=>{methods[0].checked=true;$('open-fallback').open=false;dialog.showModal();$('open-cancel').hidden=true;refreshCapability();};
 for(const method of methods)method.onchange=refreshCapability;
 $('open-close').onclick=()=>{generation++;controller?.abort();dialog.close();};
 $('choose-package').onclick=()=>$('package-input').click();$('choose-cad').onclick=()=>$('cad-input').click();
 $('open-cancel').onclick=()=>{generation++;controller?.abort();status.textContent='Geannuleerd';$('open-cancel').hidden=true;$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;translateTree(dialog);};

 async function selected(files,kind){
  if(!files.length)return;const current=++generation;controller?.abort();controller=new AbortController();const signal=controller.signal;
  $('open-cancel').hidden=false;$('choose-package').disabled=$('choose-cad').disabled=true;methods.forEach(method=>method.disabled=true);
  try{
   if(!cap?.available)throw Error('Bestandsverwerking is niet beschikbaar');
   if(files.length>cap.limits.files||files.reduce((sum,file)=>sum+file.size,0)>cap.limits.bytes)throw Error(t('Bestandslimiet overschreden'));
   status.textContent='Bestanden voorbereiden…';translateTree(dialog);
   const items=[];let name=files[0].name;
   for(const file of files){
    if(signal.aborted)throw new DOMException('Cancelled','AbortError');
    const relative=kind==='package'?file.webkitRelativePath:file.name;
    if(kind==='package')name=relative.split('/')[0];
    items.push({path:kind==='package'?relative.split('/').slice(1).join('/'):relative,bytes:await file.arrayBuffer()});
   }
   const request={kind,name,files:items,processing:processing()};
   const result=await client.open(request,{signal,onProgress:phase=>{status.textContent=({reading:'Bestand inlezen…',converting:'Omzetten naar IFCCAD…',validating:'Pakket controleren…',preparing:'Graph voorbereiden…'})[phase]||phase;translateTree(dialog);}});
   if(current!==generation)return;onResult(result,request);dialog.close();
  }catch(error){if(current===generation){status.textContent=error.name==='AbortError'?'Geannuleerd':error.message;translateTree(dialog);}}
  finally{methods.forEach(method=>method.disabled=false);if(current===generation){$('open-cancel').hidden=true;$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;}}
 }
 for(const [id,kind] of [['package-input','package'],['cad-input','cad']])$(id).onchange=event=>{selected([...event.target.files],kind);event.target.value='';};
 dialog.addEventListener('cancel',()=>{generation++;controller?.abort();$('open-cancel').hidden=true;});
 return {cancel(){generation++;controller?.abort();}};
}
