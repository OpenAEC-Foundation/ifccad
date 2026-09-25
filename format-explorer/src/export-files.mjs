import {createFileClient} from './browser-client.mjs';
import {renderExportReport} from './reports.mjs';
import {translateTree} from './i18n.mjs';
import {cadVersions,defaultCadVersion} from './cad-formats.mjs';

export function drawingChoices(fixture){return (fixture?.ifcx?.data||[]).filter(n=>n.type==='openaec:Drawing').map(n=>({id:n.path,label:n.attributes?.name||n.path}));}
export function downloadName(name,format){return (String(name).replace(/\.(dxf|dwg|ifccad|zip)$/i,'').replace(/[^\p{L}\p{N}._ -]/gu,'_').slice(0,110)||'drawing')+'-ifccad.'+(format==='ifccad'?'zip':format);}
export function initializeExporting(){
 const $=id=>document.getElementById(id),dialog=$('export-dialog'),client=createFileClient();
 $('export-version').replaceChildren(...cadVersions.map(([code,year])=>{const option=document.createElement('option');option.value=code;option.textContent=year;return option;}));$('export-version').value=defaultCadVersion;
 let source=null,controller,generation=0,objectUrl=null,available=false;
 function discard(){if(objectUrl)URL.revokeObjectURL(objectUrl);objectUrl=null;$('export-download').hidden=true;$('export-report').replaceChildren();}
 function busy(value){$('export-run').disabled=value||!available||!source;$('export-drawing').disabled=$('export-format').disabled=$('export-version').disabled=value;$('export-cancel').hidden=!value;}
 function cancel(){generation++;controller?.abort();busy(false);}
 function status(text){$('export-status').textContent=text;translateTree(dialog);}
 function selectionStatus(){status($('export-format').value==='ifccad'?'Klik op ‘Export maken’ om het volledige IFCCAD-pakket voor te bereiden.':'Kies een tekening en een bestandsformaat.');}
 function describe(){const whole=$('export-format').value==='ifccad';$('export-drawing-label').hidden=whole;$('export-version-label').hidden=whole;
  $('export-source-note').textContent=whole?'Het volledige IFCCAD-pakket wordt gedownload als ZIP. Pak dit uit om de pakketmap opnieuw te openen.':source?.kind==='cad'?'De export gebruikt de native IFCCAD-inhoud. Eerder gemeld verlies bij het openen wordt niet hersteld.':'Eén geselecteerde tekening wordt geëxporteerd; andere tekeningen blijven buiten deze export.';translateTree(dialog);
 }
 $('export-open').onclick=async()=>{
  if(!source)return;dialog.showModal();const current=++generation;available=false;busy(false);status('Bestandslezer controleren…');
  try{const cap=await client.capabilities(source.processing);if(current!==generation)return;available=cap.available;
   status(!available?'De bestandslezer is tijdelijk niet beschikbaar. Probeer het later opnieuw.':cap.processing==='server'?'De bronbestanden worden voor deze export tijdelijk naar de OpenAEC-server gestuurd.':'De export wordt in deze browser verwerkt.');busy(false);
  }catch{if(current===generation)status('De bestandslezer is niet bereikbaar. Probeer het later opnieuw of start de lokale ontwikkelserver.');}
 };
 $('export-close').onclick=()=>{cancel();dialog.close();};dialog.addEventListener('cancel',cancel);
 $('export-cancel').onclick=()=>{cancel();status('Geannuleerd');};
 for(const id of ['export-drawing','export-format','export-version'])$(id).onchange=()=>{discard();describe();selectionStatus();};
 $('export-run').onclick=async()=>{
  if(!source||!available)return;
  const current=++generation;controller?.abort();controller=new AbortController();discard();busy(true);
  const selectedSource=source,format=$('export-format').value,drawing=$('export-drawing').value,version=$('export-version').value;
  status('Export voorbereiden…');
  try{
   const result=await client.open({...selectedSource,export:format==='ifccad'?{format}:{format,drawing,version}},{signal:controller.signal,onProgress:phase=>{
    if(current===generation)status(({reading:'Bestand inlezen…',converting:'Omzetten naar IFCCAD…',validating:'Pakket controleren…',exporting:'Omzetten naar CAD…',writing:'CAD-bestand schrijven…',checking:'Exportbestand teruglezen…',packaging:'Pakket inpakken…'})[phase]||phase);
   }});
   if(current!==generation)return;
   renderExportReport($('export-report'),result);
   const file=result.export?.download;
   if(!result.failure&&file){
    if(file.format!==format)throw Error('Unexpected download format');
    const binary=atob(file.base64);if(binary.length!==file.byteLength)throw Error('Incomplete download');
    objectUrl=URL.createObjectURL(new Blob([Uint8Array.from(binary,c=>c.charCodeAt(0))],{type:format==='ifccad'?'application/zip':format==='dxf'?'application/dxf':'application/acad'}));
    const link=$('export-download');link.href=objectUrl;link.download=downloadName(selectedSource.name,format);link.textContent=format==='ifccad'?'Download IFCCAD (ZIP)':'Download '+format.toUpperCase();link.hidden=false;
    status('Export gereed. Bekijk het rapport en download het bestand.');
   }else status('Export mislukt. Er is geen download beschikbaar.');
  }catch(e){if(current===generation)status(e.name==='AbortError'?'Geannuleerd':e.message);}
  finally{if(current===generation)busy(false);}
 };
 return {setSource(request,fixture){
  cancel();discard();source=request;const choices=drawingChoices(fixture);
  $('export-drawing').replaceChildren(...choices.map(d=>{const o=document.createElement('option');o.value=d.id;o.textContent=d.label;o.dataset.noI18n='';return o;}));
  $('export-open').disabled=!source||!choices.length;
  describe();
  selectionStatus();
 }};
}
