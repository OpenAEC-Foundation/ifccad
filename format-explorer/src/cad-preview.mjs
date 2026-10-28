import {encodeBase64} from './browser-client.mjs';
import {createOcsSession} from './ocs-messages.mjs';
import {supportsCadVersion,defaultCadVersion} from './cad-formats.mjs';

function drawingFormat(){return 'ifccad';}
function drawingLabel(){return 'IFCCAD';}

export function createCadPreviewController({openSession,onUpdate=()=>{},onCadSelection=()=>{}}){
 const state={source:null,valid:false,visible:false,busy:false,viewerBusy:false,phase:'',format:'dxf',version:defaultCadVersion,conversionOptions:undefined,result:null,download:null,error:'',viewerError:'',viewerReady:false,selectionKey:null,selectionLayout:null,selectionStatus:'',selectionError:'',selectionRequested:false};
 let selectionVersion=0,selectionQueue=Promise.resolve(),appliedSelection=null;
 let sourceRevision=0,generation=0,epoch=0,controller=null,preparation=null,viewPromise=null,session=null,sessionPromise=null,originalOpened=false,displayedKey=null,preparedKey=null;
 const notify=()=>{session?.setVisible?.(state.visible&&!state.busy&&!state.viewerBusy&&!!state.download&&displayedKey===preparedKey);onUpdate({...state});};
 const outputKey=()=>String(sourceRevision);
 function invalidate(){generation++;selectionVersion++;appliedSelection=null;controller?.abort();controller=null;preparation=null;viewPromise=null;state.busy=false;state.viewerBusy=false;}
 function resetSession(){session?.close();session=null;sessionPromise=null;originalOpened=false;displayedKey=null;state.viewerReady=false;}
 function clear(){invalidate();sourceRevision++;originalOpened=false;displayedKey=null;session?.setVisible?.(false);preparedKey=null;Object.assign(state,{opening:null,source:null,valid:false,visible:false,phase:'',result:null,download:null,error:'',viewerError:'',selectionKey:null,selectionLayout:null,selectionStatus:'',selectionError:'',selectionRequested:false});notify();}
 function receiveCadSelection(event){
  if(!state.visible||!state.valid||state.busy||state.viewerBusy||!state.download||displayedKey!==preparedKey||!Array.isArray(event.handles)||event.handles.length!==1)return;
  const handle=event.handles[0];if(typeof handle!=='string'||!/^([0-9a-f]{1,16})$/i.test(handle))return;
  const canonical=handle.toUpperCase().replace(/^0+(?=[0-9A-F])/,'');
  const map=state.opening?.preview?.viewerSelection;
  if(map?.format!==drawingFormat(state.source))return;
  const matches=map.entities?.filter(entry=>entry.handle.toUpperCase()===canonical&&entry.layout===event.layout)||[];
  if(matches.length!==1)return;
  const key=matches[0].path;
  if(key===state.selectionKey){appliedSelection=JSON.stringify([generation,selectionVersion,key,null]);if(state.selectionStatus!=='selected'){state.selectionStatus='selected';state.selectionError='';notify();}return;}
  selectionVersion++;state.selectionRequested=true;state.selectionKey=key;state.selectionLayout=null;state.selectionStatus='selected';state.selectionError='';
  appliedSelection=JSON.stringify([generation,selectionVersion,key,null]);notify();onCadSelection(key);
 }
 function syncSelection(current,active){
  if(!state.selectionRequested||!active||!state.download||displayedKey!==preparedKey)return Promise.resolve();
  const version=selectionVersion,key=state.selectionKey,requestedLayout=state.selectionLayout,signature=JSON.stringify([current,version,key,requestedLayout]);if(signature===appliedSelection)return Promise.resolve();
  const task=selectionQueue.then(async()=>{
   if(current!==generation||version!==selectionVersion)return;
   const map=state.opening?.preview?.viewerSelection,target=map?.format===drawingFormat(state.source)?map.entities?.find(entity=>entity.path===key):undefined;
   try{
    if(typeof active.selectGenerated!=='function')throw Error('CAD selection is unavailable in this viewer');
    await active.selectGenerated(target?[target.handle]:[],target?.layout??requestedLayout??undefined);
    if(current===generation&&version===selectionVersion){appliedSelection=signature;state.selectionStatus=requestedLayout!==null?'selected':key?(target?'selected':'unavailable'):'';state.selectionError='';notify();}
   }catch(error){if(current===generation&&version===selectionVersion){state.selectionStatus='unavailable';state.selectionError=error.message;notify();}}
  });selectionQueue=task.catch(()=>{});return task;
 }
 async function selectRequest(key,layout=null){
  state.selectionRequested=true;state.selectionKey=key;state.selectionLayout=layout;state.selectionStatus=key||layout!==null?'pending':'';state.selectionError='';selectionVersion++;notify();
  if(!state.visible||!state.valid)return;
  if(session&&state.download&&displayedKey===preparedKey&&preparedKey===outputKey()&&!state.viewerError){await syncSelection(generation,session);return;}
  await showDrawing();await syncSelection(generation,session);
 }
 async function ensureSession(current){
  if(!sessionPromise){const started=epoch;sessionPromise=Promise.resolve().then(openSession).then(value=>{if(started!==epoch){value.close();return null;}session=value;value.subscribeSelection?.(event=>{if(started===epoch&&session===value)receiveCadSelection(event);});return value;});}
  const active=await sessionPromise;if(current!==generation||!active)return null;
  state.viewerReady=true;state.viewerError='';notify();
  if(state.source.kind==='cad'&&!originalOpened){const file=state.source.files[0];await (active.replaceOriginal??active.openOriginal)(file.base64??encodeBase64(file.bytes),file.path);if(current!==generation)return null;originalOpened=true;}
  return active;
 }
 function prepare(){
  if(!state.source||!state.valid)return Promise.resolve();
  const key=outputKey();if(preparedKey===key&&state.download)return Promise.resolve(state.result);
  if(preparation?.key===key)return preparation.promise;
  invalidate();const current=generation;
  controller=new AbortController();const signal=controller.signal;
  state.busy=true;state.phase='preparing';state.result=null;state.download=null;state.error='';notify();
  const promise=Promise.resolve().then(async()=>{
   try{
    const result=state.opening;
    if(current!==generation||signal.aborted)return;
    if(result?.failure)throw Error(result.failure.message);
    const file=result?.nativeFile;
    if(!result?.validation?.strictAvailable||!file)throw Error('Geen gevalideerd IFCX-bestand beschikbaar.');
    if(file.format!=='ifccad'||!file.fileName?.endsWith('.ifcx')||atob(file.base64).length!==file.byteLength)throw Error('Het IFCX-bestand komt niet overeen met de gevalideerde bron.');
    state.download=file;preparedKey=key;
    return result;
   }catch(error){if(current===generation&&!signal.aborted){state.error=error.message;state.result={...state.result,failure:state.result?.failure||{message:error.message}};}}
   finally{if(current===generation){preparation=null;controller=null;state.busy=false;state.phase=state.error?'failed':'ready';notify();}}
  });
  preparation={key,promise};return promise;
 }
 function showDrawing(){
  if(!state.visible||!state.source)return Promise.resolve();
  const prepared=prepare(),current=generation;if(viewPromise)return viewPromise;
  state.viewerBusy=true;notify();
  viewPromise=(async()=>{
   let active;
   try{active=await ensureSession(current);}catch(error){if(current===generation){state.viewerError=error.message;resetSession();notify();}}
   await prepared;if(current!==generation)return;
   if(!state.valid){state.error=`Conversie naar ${drawingLabel(state.source)} is niet beschikbaar. Alleen het oorspronkelijke CAD-bestand kan worden bekeken.`;return;}
   if(active&&state.download&&displayedKey!==preparedKey){try{await active.replaceGenerated(state.download.base64,state.download.fileName);if(current===generation)displayedKey=preparedKey;}catch(error){if(current===generation)state.viewerError=error.message;}}
   if(current===generation&&!state.viewerError)await syncSelection(current,active);
  })().finally(()=>{if(current===generation){viewPromise=null;state.viewerBusy=false;notify();}});
  return viewPromise;
 }
 return {state,clear,prepare,selectElement:key=>selectRequest(key),selectLayout:layout=>selectRequest(null,layout),refresh:notify,cancel(){invalidate();preparedKey=null;state.result=null;state.download=null;state.phase='cancelled';notify();},updateSource(source,valid,opening){invalidate();sourceRevision++;preparedKey=null;displayedKey=null;state.opening=opening;if(drawingFormat(source)!==drawingFormat(state.source))Object.assign(state,{selectionKey:null,selectionLayout:null,selectionRequested:false,selectionStatus:"",selectionError:""});state.source=source;state.valid=valid;state.conversionOptions=source.exportConversionOptions??source.conversionOptions;state.result=null;state.download=null;state.phase='';notify();},setSource(source,valid,opening){clear();state.opening=opening;state.source=source;state.valid=valid;state.conversionOptions=source?.exportConversionOptions??source?.conversionOptions;state.format=/\.dwg$/i.test(source?.name)?'dwg':'dxf';state.version=defaultCadVersion;notify();},show(){state.visible=true;notify();return showDrawing();},hide(){state.visible=false;notify();},retryViewer(){invalidate();epoch++;resetSession();state.viewerError='';return showDrawing();},select(values){
  if(values.format!==undefined){if(!['dxf','dwg'].includes(values.format))throw Error('Unknown CAD format');state.format=values.format;}
  if(values.version!==undefined){if(!supportsCadVersion(values.version))throw Error('Unknown CAD version');state.version=values.version;}
  if(values.conversionOptions!==undefined)state.conversionOptions=values.conversionOptions;
  return state.visible?showDrawing():prepare();
 }};
}

export function initializeCadPreview({translate,onState=()=>{},onCadSelection=()=>{},externalSettings=false}={}){
 const $=id=>document.getElementById(id),mount=$('preview-frame');
 const text=(key,fallback)=>translate?translate(key):fallback;
 async function openSession(){
  const response=await fetch('./ocs/app/index.html',{method:'HEAD'});
  if(!response.ok)throw Error(text('viewUnavailable','Open CAD Studio is niet beschikbaar in deze build. Je kunt het CAD-bestand wel downloaden.'));
  const iframe=document.createElement('iframe');iframe.title='Open CAD Studio';iframe.referrerPolicy='no-referrer';iframe.src='./ocs/app/index.html';
  const session=createOcsSession(iframe,crypto.randomUUID()),close=session.close;
  session.close=()=>{close();iframe.remove();};mount.replaceChildren(iframe);
  try{await session.ready;return session;}catch(error){session.close();throw error;}
 }
 function update(state){
  if(!externalSettings){$('preview-open').disabled=state.busy||!state.source||(!state.valid&&state.source.kind!=='cad');$('preview-format').value=state.format;$('preview-version').value=state.version;$('cad-controls').hidden=!state.source;$('cad-download').disabled=state.busy||!state.valid;}
  $('cad-preview').hidden=!state.visible;
  $('drawing-empty').hidden=state.visible;$('drawing-empty-message').textContent=text(state.phase==='cancelled'?'applyToPreview':state.source?'noCadPreview':'chooseCadPreview','Open een bestand om de tekening te bekijken.');
  $('preview-format').disabled=state.busy;$('preview-version').disabled=state.busy;
  const message=state.error||state.viewerError||(state.visible&&(state.busy||state.viewerBusy||!state.viewerReady)?text('viewLoading','Tekening wordt voorbereid…'):'');
  $('preview-status').textContent=message;$('preview-notice').hidden=!message;
  $('preview-name').textContent=state.source?.name||text('drawing','Tekening');$('preview-name').title=state.source?.name||'';
  $('preview-retry').hidden=!state.viewerError;
  $('cad-selection-status').hidden=!['pending','unavailable'].includes(state.selectionStatus);$('cad-selection-status').textContent=text(state.selectionStatus==='pending'?(state.selectionLayout!==null?'cadLayoutPending':'cadSelectionPending'):(state.selectionLayout!==null?'cadLayoutUnavailable':'cadSelectionUnavailable'),'');$('cad-selection-status').title=state.selectionError;
  onState({phase:state.phase,format:state.format,version:state.version,error:state.error});
 }
 const preview=createCadPreviewController({openSession,onUpdate:update,onCadSelection});
 $('preview-open').onclick=()=>preview.show();$('preview-retry').onclick=()=>preview.retryViewer();
 $('cad-roundtrip-cancel').onclick=()=>preview.cancel();
 for(const id of ['preview-format','preview-version'])$(id).onchange=()=>preview.select({format:$('preview-format').value,version:$('preview-version').value});
 return preview;
}
