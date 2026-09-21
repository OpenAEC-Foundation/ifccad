import {createJobClient} from './job-client.mjs';
import {t,translateTree} from './i18n.mjs';
export function initializeOpening({onResult}){
 const $=id=>document.getElementById(id),client=createJobClient();let cap,controller,generation=0;
 const status=$('open-status'),dialog=$('open-dialog');
 $('open-files').onclick=async()=>{dialog.showModal();document.getElementById("open-cancel").hidden=true;status.textContent='Bestandslezer controleren…';translateTree(dialog);try{cap=await client.capabilities();status.textContent=cap.available?(cap.processing==='server'?'Je bestanden worden via HTTPS naar de OpenAEC-server gestuurd voor verwerking. Uploads worden na verwerking verwijderd; resultaten verlopen binnen vijf minuten. Maximaal 64 MiB en 1000 bestanden. Eén bestand of pakket tegelijk.':'Bestanden blijven op deze computer. Maximaal 64 MiB en 1000 bestanden.'):(cap.processing==='server'?'De bestandslezer is tijdelijk niet beschikbaar. Probeer het later opnieuw.':'Start de lokale lezer met cargo build -p ifccad-viewer.');}catch{cap=null;status.textContent='De bestandslezer is niet bereikbaar. Probeer het later opnieuw of start de lokale ontwikkelserver.';}$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;translateTree(dialog);};
 $('open-close').onclick=()=>{generation++;controller?.abort();dialog.close();};$('choose-package').onclick=()=>$('package-input').click();$('choose-cad').onclick=()=>$('cad-input').click();
 $('open-cancel').onclick=()=>{generation++;controller?.abort();status.textContent='Geannuleerd';$('open-cancel').hidden=true;$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;translateTree(dialog);};
 async function selected(files,kind){
  if(!files.length)return;const current=++generation;controller?.abort();controller=new AbortController();const signal=controller.signal;
  $('open-cancel').hidden=false;$('choose-package').disabled=$('choose-cad').disabled=true;
  try{
   if(files.length>cap.limits.files||files.reduce((sum,f)=>sum+f.size,0)>cap.limits.bytes)throw Error(t('Bestandslimiet overschreden'));
   status.textContent='Bestanden voorbereiden…';translateTree(dialog);
   const items=[];let name=files[0].name;
   for(const f of files){if(signal.aborted)throw new DOMException('Cancelled','AbortError');const bytes=new Uint8Array(await f.arrayBuffer());let binary='';for(let i=0;i<bytes.length;i+=16384)binary+=String.fromCharCode(...bytes.subarray(i,i+16384));const relative=kind==='package'?f.webkitRelativePath:f.name;if(kind==='package')name=relative.split('/')[0];items.push({path:kind==='package'?relative.split('/').slice(1).join('/'):relative,base64:btoa(binary)});}
   const result=await client.open({kind,name,files:items},{signal,onProgress:phase=>{status.textContent=({reading:'Bestand inlezen…',converting:'Omzetten naar IFCCAD…',validating:'Pakket controleren…',preparing:'Graph voorbereiden…'})[phase]||phase;translateTree(dialog);}});
   if(current!==generation)return;onResult(result);dialog.close();
  }catch(e){if(current===generation){status.textContent=e.name==='AbortError'?'Geannuleerd':e.message;translateTree(dialog);}}
  finally{if(current===generation){$('open-cancel').hidden=true;$('choose-package').disabled=$('choose-cad').disabled=!cap?.available;}}
 }
 for(const [id,kind]of [['package-input','package'],['cad-input','cad']])$(id).onchange=e=>{selected([...e.target.files],kind);e.target.value='';};
 dialog.addEventListener('cancel',()=>{generation++;controller?.abort();$('open-cancel').hidden=true;});
 return {cancel(){generation++;controller?.abort();}};
}
