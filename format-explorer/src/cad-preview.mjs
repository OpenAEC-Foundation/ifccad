import {createFileClient,encodeBase64} from './browser-client.mjs';
import {createOcsSession} from './ocs-messages.mjs';
import {supportsCadVersion,defaultCadVersion} from './cad-formats.mjs';

function drawingFormat(source){return source?.kind==='ifccad'||source?.drawingFormat==='ifccad'?'ifccad':'ocdraw';}
function drawingLabel(source){return drawingFormat(source)==='ifccad'?'IFCCAD':'OCDraw';}

export function createCadPreviewController({openExport,openSession,onUpdate=()=>{}}){
 const state={source:null,valid:false,visible:false,busy:false,format:'dxf',version:defaultCadVersion,result:null,download:null,error:'',viewerError:'',viewerReady:false};
 let generation=0,epoch=0,controller=null,session=null,sessionPromise=null,originalOpened=false,displayedKey=null;
 const cache=new Map(),notify=()=>onUpdate({...state});
 function invalidate(){generation++;controller?.abort();controller=null;state.busy=false;}
 function resetSession(){session?.close();session=null;sessionPromise=null;originalOpened=false;displayedKey=null;state.viewerReady=false;}
 function clear(){invalidate();epoch++;resetSession();cache.clear();Object.assign(state,{source:null,valid:false,visible:false,result:null,download:null,error:'',viewerError:''});notify();}
 async function ensureSession(current){
  if(!sessionPromise){const started=epoch;sessionPromise=Promise.resolve().then(openSession).then(value=>{if(started!==epoch){value.close();return null;}session=value;return value;});}
  const active=await sessionPromise;if(current!==generation||!active)return null;
  state.viewerReady=true;state.viewerError='';notify();
  if(state.source.kind==='cad'&&!originalOpened){const file=state.source.files[0];await active.openOriginal(file.base64??encodeBase64(file.bytes),file.path);if(current!==generation)return null;originalOpened=true;}
  return active;
 }
 async function run(){
  if(!state.visible||!state.source)return;
  invalidate();const current=generation;controller=new AbortController();const signal=controller.signal;
  state.busy=true;state.result=null;state.download=null;state.error='';notify();
  let active=null;
  try{active=await ensureSession(current);}catch(error){if(current===generation){state.viewerError=error.message;state.viewerReady=false;resetSession();notify();}}
  if(current!==generation)return;
  if(!state.valid){state.error=`Conversie naar ${drawingLabel(state.source)} is niet beschikbaar. Alleen het oorspronkelijke CAD-bestand kan worden bekeken.`;state.busy=false;notify();return;}
  const key=JSON.stringify([state.format,state.version]);
  try{
   const result=cache.get(key)??await openExport({...state.source,export:{format:state.format,version:state.version}},{signal});
   if(current!==generation||signal.aborted)return;
   state.result=result;
   if(result.failure)throw Error(result.failure.message);
   const file=result.export?.download;
   if(!result.validation?.strictAvailable||!file)throw Error('Geen gevalideerd CAD-bestand beschikbaar.');
   if(file.format!==state.format||result.export.requestedVersion!==state.version||atob(file.base64).length!==file.byteLength)throw Error('Het CAD-bestand komt niet overeen met de gekozen uitvoer.');
   state.download=file;
   if(file.base64.length<=24*1024*1024){cache.set(key,result);while(cache.size>2)cache.delete(cache.keys().next().value);}
   if(active&&displayedKey!==key){await active.replaceGenerated(file.base64,`via-${drawingLabel(state.source).toLowerCase()}-${state.version}.${state.format}`);if(current!==generation)return;displayedKey=key;}
  }catch(error){if(current===generation&&!signal.aborted)state.error=error.message;}
  finally{if(current===generation){state.busy=false;notify();}}
 }
 return {state,clear,setSource(source,valid){clear();state.source=source;state.valid=valid;state.format=/\.dwg$/i.test(source?.name)?'dwg':'dxf';state.version=defaultCadVersion;notify();},show(){state.visible=true;notify();return run();},hide(){invalidate();state.visible=false;notify();},retryViewer(){invalidate();epoch++;resetSession();state.viewerError='';return run();},select(values){
  if(values.format!==undefined){if(!['dxf','dwg'].includes(values.format))throw Error('Unknown CAD format');state.format=values.format;}
  if(values.version!==undefined){if(!supportsCadVersion(values.version))throw Error('Unknown CAD version');state.version=values.version;}
  return run();
 }};
}

export function initializeCadPreview(){
 const $=id=>document.getElementById(id),client=createFileClient(),mount=$('preview-frame');
 async function openSession(){
  const response=await fetch('./ocs/app/index.html',{method:'HEAD'});
  if(!response.ok)throw Error('Open CAD Studio is niet beschikbaar in deze build. Je kunt het CAD-bestand wel downloaden.');
  const iframe=document.createElement('iframe');iframe.title='Open CAD Studio';iframe.referrerPolicy='no-referrer';iframe.src='./ocs/app/index.html';
  const session=createOcsSession(iframe,crypto.randomUUID()),close=session.close;
  session.close=()=>{close();iframe.remove();};mount.replaceChildren(iframe);
  try{await session.ready;return session;}catch(error){session.close();throw error;}
 }
 function update(state){
  $('preview-open').disabled=state.busy||!state.source||(!state.valid&&state.source.kind!=='cad');
  $('cad-preview').hidden=!state.visible;$('preview-format').value=state.format;$('preview-version').value=state.version;
  $('cad-controls').hidden=!state.source;$('cad-download').disabled=state.busy||!state.valid;
  $('preview-format').disabled=state.busy;$('preview-version').disabled=state.busy;$('preview-fullscreen').disabled=!state.viewerReady;
  $('preview-status').textContent=state.busy?'Tekening wordt voorbereid…':state.error||`Resultaat via ${drawingLabel(state.source)} gereed. Raadpleeg de conversiemeldingen voor informatieverlies.`;
  $('preview-source-info').textContent=state.source?.kind==='cad'?`Open CAD Studio toont het oorspronkelijke CAD-bestand en het resultaat via ${drawingLabel(state.source)} als afzonderlijke documenten.`:`Open CAD Studio toont het DXF/DWG-resultaat via ${drawingLabel(state.source)}.`;
  $('preview-viewer-status').textContent=state.viewerError||(!state.viewerReady?'Open CAD Studio wordt voorbereid…':'');
  $('preview-retry').hidden=!state.viewerError;
  $('preview-report').textContent=state.result?JSON.stringify({conversion:state.result.conversion,export:state.result.export?{diagnostics:state.result.export.diagnostics,fileCheck:state.result.export.fileCheck}:undefined,failure:state.result.failure},null,2):'';
 }
 const preview=createCadPreviewController({openExport:(request,options)=>client.open(request,options),openSession,onUpdate:update});
 $('preview-open').onclick=()=>preview.show();$('preview-hide').onclick=async()=>{if(document.fullscreenElement)await document.exitFullscreen();preview.hide();};$('preview-retry').onclick=()=>preview.retryViewer();
 for(const id of ['preview-format','preview-version'])$(id).onchange=()=>preview.select({format:$('preview-format').value,version:$('preview-version').value});
 const surface=$('preview-surface');
 $('preview-fullscreen').onclick=async()=>{try{if(document.fullscreenElement===surface)await document.exitFullscreen();else await surface.requestFullscreen();}catch{$('preview-viewer-status').textContent='Volledig scherm is niet beschikbaar. Vergroot het browservenster of het browserpaneel.';}};
 document.addEventListener('fullscreenchange',()=>{$('preview-fullscreen').textContent=document.fullscreenElement===surface?'Volledig scherm verlaten':'Volledig scherm';});
 return preview;
}
