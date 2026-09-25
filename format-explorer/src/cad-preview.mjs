import {drawingChoices} from './export-files.mjs';
import {downloadName} from './export-files.mjs';
import {cadVersions,defaultCadVersion,supportsCadVersion} from './cad-formats.mjs';
import {createJobClient} from './job-client.mjs';
import {createOcsSession} from './ocs-messages.mjs';
import {renderExportReport} from './reports.mjs';
import {translateTree} from './i18n.mjs';

export function createCadPreviewController({openExport,openSession,onUpdate=()=>{}}){
 const state={source:null,drawings:[],drawing:'',format:'dxf',version:defaultCadVersion,original:null,visible:false,busy:false,result:null,error:'',viewerError:'',viewerReady:false,download:null};
 let generation=0,sourceEpoch=0,controller=null,session=null,sessionPromise=null,originalOpened=false,displayedKey=null;
 const cache=new Map();
 const notify=()=>onUpdate({...state});
 function invalidate(){generation++;controller?.abort();controller=null;state.busy=false;state.result=null;state.download=null;state.error='';notify();}
 function clear(){invalidate();sourceEpoch++;session?.close?.();session=null;sessionPromise=null;originalOpened=false;displayedKey=null;cache.clear();state.source=null;state.drawings=[];state.drawing='';state.original=null;state.visible=false;state.viewerError='';state.viewerReady=false;notify();}
 function setSource(source,fixture,original=null){
  clear();if(!source)return;
  state.source=source;state.drawings=drawingChoices(fixture);state.drawing=state.drawings[0]?.id||'';
  state.format=source.kind==='cad'&&/\.dwg$/i.test(original?.name||source.name)?'dwg':'dxf';
  state.version=defaultCadVersion;state.original=original;notify();
 }
 async function ensureSession(current){
  if(!sessionPromise){const epoch=sourceEpoch;sessionPromise=Promise.resolve().then(openSession).then(value=>{if(epoch!==sourceEpoch){value.close?.();return null;}session=value;return value;});}
  const active=await sessionPromise;
  if(current!==generation||!active)return null;
  state.viewerReady=true;state.viewerError='';notify();
  if(state.original&&!originalOpened){await active.openOriginal(state.original.base64,state.original.name);originalOpened=true;}
  return active;
 }
 async function run(){
  if(!state.visible||!state.source||!state.drawing)return;
  invalidate();const current=generation;controller=new AbortController();const signal=controller.signal;
  state.busy=true;notify();
  let active=null;
  try{active=await ensureSession(current);}catch(error){if(current===generation){state.viewerError=error.message;state.viewerReady=false;sessionPromise=null;notify();}}
  if(current!==generation)return;
  const key=JSON.stringify([state.drawing,state.format,state.version]);
  try{
   const request={...state.source,export:{format:state.format,drawing:state.drawing,version:state.version}};
   const result=cache.has(key)?cache.get(key):await openExport(request,{signal});
   if(current!==generation||signal.aborted)return;
   state.result=result;
   const file=result.export?.download;
   if(result.failure||!file){state.error=result.failure?.message||'No CAD download was produced';return;}
   if(file.format!==state.format||result.export?.effectiveVersion!==state.version)throw Error('Export format or CAD version does not match the selection');
   if(atob(file.base64).length!==file.byteLength)throw Error('Incomplete CAD download');
   state.download=file;
   if(file.base64.length<=24*1024*1024){cache.delete(key);cache.set(key,result);while(cache.size>2)cache.delete(cache.keys().next().value);}
   if(active&&displayedKey!==key){
    const year=cadVersions.find(([code])=>code===state.version)?.[1]||state.version;
    await active.replaceGenerated(file.base64,`via-ifccad-${year}.${state.format}`);
    if(current!==generation)return;
    displayedKey=key;
   }
   state.error='';
  }catch(error){if(current===generation&&!signal.aborted)state.error=error.message;}
  finally{if(current===generation){state.busy=false;notify();}}
 }
 return {state,setSource,clear,show(){state.visible=true;notify();return run();},hide(){state.visible=false;notify();},retryViewer(){session?.close?.();session=null;sessionPromise=null;originalOpened=false;displayedKey=null;state.viewerError='';state.viewerReady=false;return run();},select(values){
  if(values.drawing!==undefined){if(!state.drawings.some(d=>d.id===values.drawing))throw Error('Unknown drawing');state.drawing=values.drawing;}
  if(values.format!==undefined){if(!['dxf','dwg'].includes(values.format))throw Error('Unknown CAD format');state.format=values.format;}
  if(values.version!==undefined){if(!supportsCadVersion(values.version))throw Error('Unknown CAD version');state.version=values.version;}
  return run();
 }};
}

export function initializeCadPreview(){
 const $=id=>document.getElementById(id),client=createJobClient(),mount=$('preview-frame');
 let objectUrl=null,lastDownload=null;
 $('preview-version').replaceChildren(...cadVersions.map(([code,year])=>{const option=document.createElement('option');option.value=code;option.textContent=year;return option;}));
 async function openSession(){
  const available=await fetch('./ocs/app/index.html',{method:'HEAD'});
  if(!available.ok)throw Error('Open CAD Studio is niet beschikbaar in deze lokale build. Download het CAD-bestand en open het handmatig.');
  const iframe=document.createElement('iframe');iframe.title='Open CAD Studio';iframe.referrerPolicy='no-referrer';iframe.src='./ocs/app/index.html';
  const session=createOcsSession(iframe,crypto.randomUUID());mount.replaceChildren(iframe);
  try{await session.ready;return session;}catch(error){session.close();iframe.remove();throw error;}
 }
 function update(state){
  $('preview-open').disabled=!state.source||!state.drawings.length;
  $('workspace').hidden=state.visible;$('cad-preview').hidden=!state.visible;
  $('preview-open').hidden=state.visible;
  $('preview-drawing').replaceChildren(...state.drawings.map(d=>{const option=document.createElement('option');option.value=d.id;option.textContent=d.label;option.dataset.noI18n='';return option;}));
  $('preview-drawing').value=state.drawing;$('preview-format').value=state.format;$('preview-version').value=state.version;
  $('preview-viewer-status').hidden=state.viewerReady&&!state.viewerError;
  $('preview-viewer-status').textContent=state.viewerError||'Open CAD Studio wordt voorbereid…';
  $('preview-retry').hidden=!state.viewerError;
  $('preview-status').textContent=state.busy?'Roundtrip wordt voorbereid…':state.error||(!state.source?'':state.download?'Roundtrip gereed.':'Kies een tekening en tussenformaat.');
  if(state.result)renderExportReport($('preview-report'),state.result);else $('preview-report').replaceChildren();
  if(lastDownload!==state.download){if(objectUrl)URL.revokeObjectURL(objectUrl);objectUrl=null;lastDownload=state.download;
   if(state.download){const binary=atob(state.download.base64);objectUrl=URL.createObjectURL(new Blob([Uint8Array.from(binary,c=>c.charCodeAt(0))],{type:state.format==='dxf'?'application/dxf':'application/acad'}));}
  }
  const link=$('preview-download');link.hidden=!objectUrl;
  if(objectUrl){link.href=objectUrl;link.download=downloadName(state.source.name,state.format);link.textContent='Download '+state.format.toUpperCase();}
  translateTree($('cad-preview'));translateTree($('preview-open'));
 }
 const controller=createCadPreviewController({openExport:(request,options)=>client.open(request,options),openSession,onUpdate:update});
 $('preview-open').addEventListener('click',()=>controller.show());
 $('preview-close').addEventListener('click',()=>{controller.hide();$('preview-open').focus({preventScroll:true});});
 $('preview-retry').addEventListener('click',()=>controller.retryViewer());
 for(const id of ['preview-drawing','preview-format','preview-version'])$(id).addEventListener('change',()=>controller.select({drawing:$('preview-drawing').value,format:$('preview-format').value,version:$('preview-version').value}));
 return {setSource:controller.setSource,clear:controller.clear,hide:controller.hide,state:controller.state};
}
