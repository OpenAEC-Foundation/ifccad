import {createFileClient,decodeBase64} from './browser-client.mjs';
import {initializeCadPreview} from './cad-preview.mjs';
const $=id=>document.getElementById(id),client=createFileClient(),preview=initializeCadPreview();let source,controller;
function show(result){
 $('drawing').hidden=false;$('name').textContent=result.source?.name||source.name;
 $('report').textContent=JSON.stringify({reader:result.reader,conversion:result.conversion,validation:result.validation,failure:result.failure,export:result.export?{format:result.export.format,diagnostics:result.export.diagnostics}:undefined},null,2);
 $('records').textContent=JSON.stringify(result.presentation,null,2)||'Geen gevalideerde tekeninginformatie.';
 $('export').disabled=!result.validation?.strictAvailable||!!result.failure;$('cad-download').disabled=$('export').disabled;
 $('status').textContent=result.failure?.message|| (result.validation?.strictAvailable?'Tekening gecontroleerd.':'De tekening kon niet worden gevalideerd.');
}
async function run(operation){
 if(!source)return;controller=new AbortController();$('open').disabled=true;$('cancel').disabled=false;$('export').disabled=true;$('cad-download').disabled=true;
 try{const result=await client.open({...source,...(operation?{export:operation}:{})},{signal:controller.signal,onProgress:phase=>$('status').textContent=phase});show(result);if(!operation)preview.setSource({...source},!!result.validation?.strictAvailable&&!result.failure);
  if(operation&&result.export?.download&&!result.failure){const file=result.export.download,url=URL.createObjectURL(new Blob([decodeBase64(file.base64)])),link=document.createElement('a');link.href=url;link.download=file.fileName||source.name+'.'+operation.format;link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
 }catch(e){$('status').textContent=e.name==='AbortError'?'Geannuleerd.':e.message;if(!operation&&e.name!=='AbortError')preview.setSource({...source},false);}finally{controller=null;$('open').disabled=false;$('cancel').disabled=true;}
}
$('open').onclick=async()=>{
 source=undefined;preview.clear();$('export').disabled=true;$('cad-download').disabled=true;$('drawing').hidden=true;
 const file=$('file').files[0];if(!file){$('status').textContent='Kies eerst een bestand.';return;}if(file.size>64*1024*1024){$('status').textContent='Dit bestand is groter dan 64 MiB.';return;}
 $('open').disabled=true;$('status').textContent='Bestand lezen…';
 try{source={kind:/\.(dxf|dwg)$/i.test(file.name)?'cad':'drawing',name:file.name,files:[{path:file.name,bytes:await file.arrayBuffer()}]};}
 catch(e){$('status').textContent='Het bestand kon niet worden gelezen. Sluit het eventueel in het andere programma en kies het opnieuw. '+e.message;return;}
 finally{$('open').disabled=false;}
 await run();
};
$('export').onclick=()=>run({format:'ocdraw'});$('cad-download').onclick=()=>run({format:$('preview-format').value,version:$('preview-version').value});$('cancel').onclick=()=>controller?.abort();
